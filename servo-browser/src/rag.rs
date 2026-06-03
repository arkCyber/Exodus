use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
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

/// RAG database manager using sled embedded database
pub struct RagDatabase {
    db: Db,
    pages_tree: Arc<Tree>,
}

impl RagDatabase {
    /// Initialize the RAG database
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db = sled::open("exodus_rag_db")?;
        let pages_tree = Arc::new(db.open_tree("pages")?);
        
        Ok(Self {
            db,
            pages_tree,
        })
    }

    /// Store a webpage in the database
    pub async fn store_page(&self, page: WebPage) -> Result<(), Box<dyn std::error::Error>> {
        let key = page.id.as_bytes();
        let value = serde_json::to_vec(&page)?;
        self.pages_tree.insert(key, value)?;
        self.pages_tree.flush_async().await?;
        Ok(())
    }

    /// Retrieve a page by ID
    pub fn get_page(&self, id: &str) -> Result<Option<WebPage>, Box<dyn std::error::Error>> {
        if let Some(value) = self.pages_tree.get(id)? {
            let page: WebPage = serde_json::from_slice(&value)?;
            Ok(Some(page))
        } else {
            Ok(None)
        }
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

    /// Delete a page by ID
    pub fn delete_page(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pages_tree.remove(id)?;
        Ok(())
    }

    /// Clear all pages (for testing/reset)
    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pages_tree.clear()?;
        Ok(())
    }
}

/// Cosine similarity calculation for vector similarity.
/// Computes the cosine of the angle between two vectors.
/// Returns a value between -1.0 and 1.0, where 1.0 means identical direction.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Create a new WebPage instance
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

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        
        let c = vec![0.0, 0.0, 0.0];
        let d = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&c, &d), 0.0);
    }
}
