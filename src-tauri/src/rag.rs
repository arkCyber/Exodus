//! Exodus Browser — RAG (retrieval-augmented generation) storage layer.
//! Persists captured page content in a local sled database for offline semantic search.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sled::Tree;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a captured webpage for RAG indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPage {
    pub id: String,
    pub url: String,
    pub title: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub embedding: Option<Vec<f32>>, // Will be populated by exodus-core
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub page: WebPage,
    pub score: f32,
    pub matched_text: String,
}

/// Lightweight browsing history entry (auto-recorded on navigation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visit {
    pub id: String,
    pub url: String,
    pub title: String,
    pub timestamp: DateTime<Utc>,
    pub visit_count: u32,
}

/// User bookmark entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub url: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    /// Empty string = show on the bookmark bar; other values = folder name in the panel only.
    #[serde(default)]
    pub folder: String,
    /// Order on the bookmark bar (lower = further left). Only used when `folder` is empty.
    #[serde(default)]
    pub bar_order: u32,
}

/// Snapshot of open tabs for session restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub tabs: Vec<TabSnapshot>,
    pub active_tab_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Snapshot of a single tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub id: String,
    pub url: String,
    pub title: String,
    pub active: bool,
}

/// RAG database manager using sled embedded database
pub struct RagDatabase {
    pages_tree: Arc<Tree>,
    bookmarks_tree: Arc<Tree>,
    visits_tree: Arc<Tree>,
    session_tree: Arc<Tree>,
}

/// Maximum visit rows kept in the database.
const MAX_VISITS: usize = 500;

impl RagDatabase {
    /// Initialize the RAG database at the given directory (created if missing).
    pub fn new_at(data_dir: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path: PathBuf = data_dir.as_ref().join("exodus_rag_db");
        std::fs::create_dir_all(&db_path)?;
        let db = sled::open(&db_path)?;
        let pages_tree = db.open_tree("pages")?;
        let bookmarks_tree = db.open_tree("bookmarks")?;
        let visits_tree = db.open_tree("visits")?;
        let session_tree = db.open_tree("session")?;
        
        Ok(Self {
            pages_tree: Arc::new(pages_tree),
            bookmarks_tree: Arc::new(bookmarks_tree),
            visits_tree: Arc::new(visits_tree),
            session_tree: Arc::new(session_tree),
        })
    }

    /// Initialize the RAG database in the current working directory (tests / legacy).
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_at(".")
    }

    /// Store a webpage in the database
    pub async fn store_page(&self, page: WebPage) -> Result<(), Box<dyn std::error::Error>> {
        let key = page.id.as_bytes();
        let value = serde_json::to_vec(&page)?;
        self.pages_tree.insert(key, value)?;
        self.pages_tree.flush_async().await?;
        Ok(())
    }

    /// Search indexed pages by title, URL, or content snippet.
    pub fn search_pages(&self, query: &str) -> Result<Vec<WebPage>, Box<dyn std::error::Error>> {
        let query_lower = query.to_lowercase();
        let pages = self.get_all_pages()?;
        Ok(pages
            .into_iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&query_lower)
                    || p.url.to_lowercase().contains(&query_lower)
                    || p.content.to_lowercase().contains(&query_lower)
            })
            .collect())
    }

    /// Get all pages (for search)
    pub fn get_all_pages(&self) -> Result<Vec<WebPage>, Box<dyn std::error::Error>> {
        let mut pages = Vec::new();
        
        for item in self.pages_tree.iter() {
            let (_, value) = item?;
            let page: WebPage = serde_json::from_slice(&value)?;
            pages.push(page);
        }
        
        Ok(pages)
    }

    /// Clear all RAG pages and visit history.
    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pages_tree.clear()?;
        self.visits_tree.clear()?;
        Ok(())
    }

    /// Remove a single indexed page by id (RAG only; visits unchanged).
    pub fn remove_page_by_id(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.pages_tree.remove(id.as_bytes())? {
            Some(_) => Ok(()),
            None => Err(format!("Indexed page not found: {}", id).into()),
        }
    }

    /// Record or update a visit for browsing history.
    pub fn record_visit(&self, url: String, title: String) -> Result<Visit, Box<dyn std::error::Error>> {
        let mut visits = self.list_visits()?;
        if let Some(existing) = visits.iter_mut().find(|v| v.url == url) {
            existing.title = if title.is_empty() {
                existing.title.clone()
            } else {
                title
            };
            existing.timestamp = Utc::now();
            existing.visit_count = existing.visit_count.saturating_add(1);
            let updated = existing.clone();
            self.visits_tree
                .insert(updated.id.as_bytes(), serde_json::to_vec(&updated)?)?;
            self.prune_visits_if_needed()?;
            return Ok(updated);
        }

        let visit = Visit {
            id: Uuid::new_v4().to_string(),
            url,
            title,
            timestamp: Utc::now(),
            visit_count: 1,
        };
        self.visits_tree
            .insert(visit.id.as_bytes(), serde_json::to_vec(&visit)?)?;
        self.prune_visits_if_needed()?;
        Ok(visit)
    }

    /// List visits sorted by most recent first.
    pub fn list_visits(&self) -> Result<Vec<Visit>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();
        for item in self.visits_tree.iter() {
            let (_, value) = item?;
            items.push(serde_json::from_slice::<Visit>(&value)?);
        }
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(items)
    }

    /// Search visits by title or URL.
    pub fn search_visits(&self, query: &str) -> Result<Vec<Visit>, Box<dyn std::error::Error>> {
        let visits = self.list_visits()?;
        let query_lower = query.to_lowercase();
        
        let filtered: Vec<Visit> = visits
            .into_iter()
            .filter(|v| {
                v.title.to_lowercase().contains(&query_lower) || 
                v.url.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        Ok(filtered)
    }

    /// Clear all browsing history visits.
    pub fn clear_visits(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.visits_tree.clear()?;
        Ok(())
    }

    /// Save session snapshot (open tabs and active tab).
    pub async fn save_session(&self, snapshot: SessionSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        let key = b"current_session";
        let value = serde_json::to_vec(&snapshot)?;
        self.session_tree.insert(key, value)?;
        self.session_tree.flush_async().await?;
        Ok(())
    }

    /// Load session snapshot.
    pub fn load_session(&self) -> Result<Option<SessionSnapshot>, Box<dyn std::error::Error>> {
        let key = b"current_session";
        if let Some(value) = self.session_tree.get(key)? {
            let snapshot = serde_json::from_slice::<SessionSnapshot>(&value)?;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    /// Clear session snapshot.
    pub fn clear_session(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.session_tree.remove(b"current_session")?;
        Ok(())
    }

    /// Drop oldest visits when over capacity.
    fn prune_visits_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut visits = self.list_visits()?;
        if visits.len() <= MAX_VISITS {
            return Ok(());
        }
        visits.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        let to_remove = visits.len() - MAX_VISITS;
        for visit in visits.into_iter().take(to_remove) {
            self.visits_tree.remove(visit.id.as_bytes())?;
        }
        Ok(())
    }

    /// Store or update a page keyed by URL (avoids duplicate RAG entries).
    pub async fn upsert_page_by_url(
        &self,
        url: String,
        title: String,
        content: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let existing = self
            .get_all_pages()?
            .into_iter()
            .find(|p| p.url == url);

        let page = if let Some(mut page) = existing {
            page.title = title;
            page.content = content;
            page.timestamp = Utc::now();
            page
        } else {
            create_webpage(url, title, content)
        };

        let id = page.id.clone();
        self.store_page(page).await?;
        Ok(id)
    }

    /// Attach an embedding vector to the page with the given URL.
    pub async fn set_embedding_for_url(
        &self,
        url: &str,
        embedding: Vec<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pages = self.get_all_pages()?;
        let Some(mut page) = pages.into_iter().find(|p| p.url == url) else {
            return Ok(());
        };
        page.embedding = Some(embedding);
        self.store_page(page).await?;
        Ok(())
    }

    /// List bookmarks sorted by newest first.
    pub fn list_bookmarks(&self) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();
        for item in self.bookmarks_tree.iter() {
            let (_, value) = item?;
            items.push(serde_json::from_slice::<Bookmark>(&value)?);
        }
        items.sort_by(|a, b| {
            let a_bar = a.folder.is_empty();
            let b_bar = b.folder.is_empty();
            match (a_bar, b_bar) {
                (true, true) => a
                    .bar_order
                    .cmp(&b.bar_order)
                    .then_with(|| a.created_at.cmp(&b.created_at)),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => b.created_at.cmp(&a.created_at),
            }
        });
        Ok(items)
    }

    /// Search bookmarks by title or URL.
    pub fn search_bookmarks(&self, query: &str) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        let bookmarks = self.list_bookmarks()?;
        let query_lower = query.to_lowercase();
        
        let filtered: Vec<Bookmark> = bookmarks
            .into_iter()
            .filter(|b| {
                b.title.to_lowercase().contains(&query_lower) || 
                b.url.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        Ok(filtered)
    }

    /// Add a bookmark (replaces same URL if present).
    pub fn add_bookmark(
        &self,
        url: String,
        title: String,
        folder: String,
    ) -> Result<Bookmark, Box<dyn std::error::Error>> {
        tracing::info!("Adding bookmark: url={}, title={}", url, title);
        if url.trim().is_empty() {
            return Err("URL cannot be empty".into());
        }
        let mut bookmarks = self.list_bookmarks()?;
        if let Some(existing) = bookmarks.iter_mut().find(|b| b.url == url) {
            existing.title = title;
            existing.folder = folder;
            existing.created_at = Utc::now();
            let updated = existing.clone();
            let key = updated.id.as_bytes();
            self.bookmarks_tree
                .insert(key, serde_json::to_vec(&updated)?)?;
            return Ok(updated);
        }

        let bar_order = if folder.is_empty() {
            bookmarks
                .iter()
                .filter(|b| b.folder.is_empty())
                .map(|b| b.bar_order)
                .max()
                .unwrap_or(0)
                .saturating_add(1)
        } else {
            0
        };
        let bookmark = Bookmark {
            id: Uuid::new_v4().to_string(),
            url,
            title,
            created_at: Utc::now(),
            folder,
            bar_order,
        };
        self.bookmarks_tree
            .insert(bookmark.id.as_bytes(), serde_json::to_vec(&bookmark)?)?;
        Ok(bookmark)
    }

    /// Remove bookmark by id.
    pub fn remove_bookmark(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.bookmarks_tree.remove(id)?;
        Ok(())
    }

    /// Move a bookmark into a folder (empty string = bookmark bar).
    pub fn update_bookmark_folder(
        &self,
        id: &str,
        folder: String,
    ) -> Result<Bookmark, Box<dyn std::error::Error>> {
        let mut bookmarks = self.list_bookmarks()?;
        
        // Calculate next_order before getting mutable reference
        let next_order = if folder.is_empty() {
            bookmarks
                .iter()
                .filter(|b| b.folder.is_empty() && b.id != id)
                .map(|b| b.bar_order)
                .max()
                .unwrap_or(0)
                .saturating_add(1)
        } else {
            0
        };
        
        let bookmark = bookmarks
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or("Bookmark not found")?;
        bookmark.folder = folder.clone();
        bookmark.bar_order = next_order;
        let updated = bookmark.clone();
        self.bookmarks_tree
            .insert(updated.id.as_bytes(), serde_json::to_vec(&updated)?)?;
        Ok(updated)
    }

    /// Reorder bookmarks on the bar (left-to-right). Ids must be bar bookmarks (`folder` empty).
    pub fn reorder_bookmarks_bar(
        &self,
        ordered_ids: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut bookmarks = self.list_bookmarks()?;
        for (index, id) in ordered_ids.iter().enumerate() {
            let bookmark = bookmarks
                .iter_mut()
                .find(|b| b.id == *id)
                .ok_or_else(|| format!("Bookmark not found: {id}"))?;
            if !bookmark.folder.is_empty() {
                bookmark.folder = String::new();
            }
            bookmark.bar_order = index as u32;
        }
        for id in ordered_ids {
            if let Some(bookmark) = bookmarks.iter().find(|b| b.id == id) {
                self.bookmarks_tree
                    .insert(bookmark.id.as_bytes(), serde_json::to_vec(bookmark)?)?;
            }
        }
        Ok(())
    }
}

/// Score a page against a natural-language query using word overlap.
/// This is a fallback implementation when embeddings are not available.
/// Returns a score between 0.0 and 1.0 based on query word matches in title and content.
pub fn score_page_match(page: &WebPage, query: &str) -> f32 {
    let query_lower = query.to_lowercase();
    let words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .collect();

    if words.is_empty() {
        return 0.0;
    }

    let haystack = format!("{} {}", page.title, page.content).to_lowercase();
    let mut hits = 0usize;
    for word in &words {
        if haystack.contains(word) {
            hits += 1;
        }
    }

    let ratio = hits as f32 / words.len() as f32;
    if ratio == 0.0 {
        return 0.0;
    }

    // Boost title matches
    let title_lower = page.title.to_lowercase();
    let title_boost = if words.iter().any(|w| title_lower.contains(w)) {
        0.15
    } else {
        0.0
    };

    (ratio * 0.85 + title_boost).min(1.0)
}

/// Create a new WebPage with generated ID and timestamp
pub fn create_webpage(url: String, title: String, content: String) -> WebPage {
    WebPage {
        id: Uuid::new_v4().to_string(),
        url,
        title,
        content,
        timestamp: Utc::now(),
        embedding: None, // Will be populated by exodus-core
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> RagDatabase {
        let dir = std::env::temp_dir().join(format!("exodus_rag_test_{}", Uuid::new_v4()));
        RagDatabase::new_at(&dir).expect("temp db")
    }

    #[tokio::test]
    async fn test_set_embedding_for_url() {
        let db = temp_db();
        let url = "https://embed-test.example/page";
        db.upsert_page_by_url(url.into(), "Title".into(), "Content about rust".into())
            .await
            .expect("upsert");
        db.set_embedding_for_url(url, vec![1.0, 0.0, 0.0])
            .await
            .expect("set embedding");
        let page = db
            .get_all_pages()
            .expect("list")
            .into_iter()
            .find(|p| p.url == url)
            .expect("page");
        assert_eq!(page.embedding.as_ref().map(|v| v.len()), Some(3));
    }

    #[test]
    fn test_score_page_match_finds_keywords() {
        let page = WebPage {
            id: "1".into(),
            url: "https://example.com".into(),
            title: "Rust Programming".into(),
            content: "Systems language for performance".into(),
            timestamp: Utc::now(),
            embedding: None,
        };
        assert!(score_page_match(&page, "rust programming") > 0.3);
        assert_eq!(score_page_match(&page, "xyznone"), 0.0);
    }

    #[test]
    fn test_record_visit_dedupes_by_url() {
        let db = temp_db();
        let v1 = db
            .record_visit(
                "https://example.com".into(),
                "Example".into(),
            )
            .expect("visit");
        let v2 = db
            .record_visit(
                "https://example.com".into(),
                "Example updated".into(),
            )
            .expect("visit again");
        assert_eq!(v1.id, v2.id);
        assert_eq!(v2.visit_count, 2);
        assert_eq!(v2.title, "Example updated");
    }

    #[test]
    fn test_bookmark_folder_update() {
        let db = temp_db();
        let bm = db
            .add_bookmark(
                "https://a.com".into(),
                "A".into(),
                String::new(),
            )
            .expect("add");
        let updated = db
            .update_bookmark_folder(&bm.id, "Work".into())
            .expect("folder");
        assert_eq!(updated.folder, "Work");
    }

    #[test]
    fn test_search_visits_filters_title_and_url() {
        let db = temp_db();
        db.record_visit("https://docs.rs".into(), "Rust docs".into())
            .expect("visit");
        db.record_visit("https://example.com".into(), "Example".into())
            .expect("visit");
        let hits = db.search_visits("rust").expect("search");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].url.contains("docs.rs"));
    }

    #[test]
    fn session_snapshot_serializes_camel_case() {
        let snapshot = SessionSnapshot {
            tabs: vec![TabSnapshot {
                id: "t1".into(),
                url: "https://a.test".into(),
                title: "A".into(),
                active: true,
            }],
            active_tab_id: Some("t1".into()),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&snapshot).expect("json");
        assert!(json.contains("\"activeTabId\""));
    }

    #[tokio::test]
    async fn test_session_save_and_load() {
        let db = temp_db();
        let snapshot = SessionSnapshot {
            tabs: vec![TabSnapshot {
                id: "tab-1".into(),
                url: "https://example.com".into(),
                title: "Example".into(),
                active: true,
            }],
            active_tab_id: Some("tab-1".into()),
            timestamp: Utc::now(),
        };
        db.save_session(snapshot).await.expect("save");
        let loaded = db.load_session().expect("load").expect("some");
        assert_eq!(loaded.tabs.len(), 1);
        assert_eq!(loaded.active_tab_id.as_deref(), Some("tab-1"));
        db.clear_session().expect("clear");
        assert!(db.load_session().expect("load").is_none());
    }

    #[test]
    fn test_search_pages_and_bookmarks() {
        let db = temp_db();
        db.add_bookmark(
            "https://rust-lang.org".into(),
            "Rust Lang".into(),
            String::new(),
        )
        .expect("bookmark");
        let hits = db.search_bookmarks("rust").expect("search bookmarks");
        assert_eq!(hits.len(), 1);

        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            db.upsert_page_by_url(
                "https://docs.rs".into(),
                "Rust docs".into(),
                "The Rust programming language".into(),
            )
            .await
            .expect("upsert");
        });
        let pages = db.search_pages("programming").expect("search pages");
        assert_eq!(pages.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_page_by_id() {
        let db = temp_db();
        let id = db
            .upsert_page_by_url(
                "https://remove-me.example".into(),
                "T".into(),
                "body".into(),
            )
            .await
            .expect("upsert");
        assert_eq!(db.get_all_pages().expect("list").len(), 1);
        db.remove_page_by_id(&id).expect("remove");
        assert!(db.get_all_pages().expect("list").is_empty());
    }

    #[test]
    fn test_clear_visits() {
        let db = temp_db();
        db.record_visit("https://b.com".into(), "B".into())
            .expect("visit");
        assert_eq!(db.list_visits().expect("list").len(), 1);
        db.clear_visits().expect("clear");
        assert!(db.list_visits().expect("list").is_empty());
    }

    #[test]
    fn test_import_bookmarks_merges_duplicate_url() {
        let db = temp_db();
        db.add_bookmark(
            "https://example.com".into(),
            "Old title".into(),
            String::new(),
        )
        .expect("seed");
        let json = r#"[{"id":"x","url":"https://example.com","title":"New title","created_at":"2026-01-01T00:00:00Z","folder":"Work"}]"#;
        let parsed: Vec<Bookmark> = serde_json::from_str(json).expect("parse");
        for b in parsed {
            db.add_bookmark(b.url, b.title, b.folder).expect("import");
        }
        let list = db.list_bookmarks().expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "New title");
        assert_eq!(list[0].folder, "Work");
    }

    #[test]
    fn test_bookmarks_export_import_roundtrip() {
        let source = temp_db();
        source
            .add_bookmark(
                "https://example.com".into(),
                "Example".into(),
                String::new(),
            )
            .expect("bookmark a");
        source
            .add_bookmark(
                "https://work.example/doc".into(),
                "Work doc".into(),
                "Work".into(),
            )
            .expect("bookmark b");

        let exported = source.list_bookmarks().expect("export list");
        let json = serde_json::to_string(&exported).expect("serialize export");

        let parsed: Vec<Bookmark> = serde_json::from_str(&json).expect("parse export json");
        assert_eq!(parsed.len(), 2);

        let target = temp_db();
        for bookmark in parsed {
            target
                .add_bookmark(
                    bookmark.url.clone(),
                    bookmark.title.clone(),
                    bookmark.folder.clone(),
                )
                .expect("import bookmark");
        }

        let imported = target.list_bookmarks().expect("list imported");
        assert_eq!(imported.len(), 2);
        assert!(
            imported
                .iter()
                .any(|b| b.url == "https://example.com" && b.title == "Example")
        );
        assert!(
            imported
                .iter()
                .any(|b| b.url == "https://work.example/doc" && b.folder == "Work")
        );
    }
}
