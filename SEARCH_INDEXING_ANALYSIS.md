# Search Indexing Analysis for Performance Optimization

## Current Search Implementation

### Bookmark Search
- **Location**: `src-tauri/src/rag.rs::search_bookmarks()`
- **Method**: Linear scan through all bookmarks
- **Complexity**: O(n) where n = number of bookmarks
- **Filtering**: Case-insensitive substring match on URL and title

### History Search
- **Location**: `src-tauri/src/rag.rs::search_visits()`
- **Method**: Linear scan through all visits
- **Complexity**: O(n) where n = number of visits
- **Filtering**: Case-insensitive substring match on URL and title, deduplication by URL

### Page Search (RAG)
- **Location**: `src-tauri/src/rag.rs::search_pages()`
- **Method**: Linear scan through all pages with embedding scoring
- **Complexity**: O(n) where n = number of indexed pages
- **Filtering**: Keyword matching + embedding cosine similarity

## Performance Considerations

### Current Performance
- **Small datasets** (< 1000 items): Linear scan is acceptable
- **Medium datasets** (1000-10000 items): May start to show latency
- **Large datasets** (> 10000 items): Significant latency expected

### Typical Usage Patterns
- Bookmarks: Usually < 1000 items
- History: Could grow to 10,000+ items over months
- Indexed Pages: Could grow to 10,000+ items

## Indexing Options

### Option 1: Sled Database Indexes
**Pros:**
- Built-in to existing database
- Simple to implement
- No additional dependencies

**Cons:**
- Limited indexing capabilities
- May not support full-text search well

**Implementation:**
```rust
// Add secondary indexes to sled trees
db.bookmarks_tree.open_tree("bookmarks_by_url")?;
db.visits_tree.open_tree("visits_by_url")?;
```

### Option 2: Tantivy (Rust Full-Text Search)
**Pros:**
- Professional-grade full-text search
- BM25 ranking algorithm
- Fast queries even on large datasets
- Supports fuzzy matching

**Cons:**
- Additional dependency
- Requires separate index storage
- More complex implementation

**Implementation:**
```toml
[dependencies]
tantivy = "0.22"
```

```rust
use tantivy::{Index, IndexWriter, Searcher, Term, query::QueryParser};

// Create index
let index = Index::create_in_ram(schema);
let mut writer = index.writer(50_000_000)?;

// Add documents
writer.add_document(doc!{
    "url" => url,
    "title" => title,
    "body" => body_content
});

// Search
let reader = index.reader()?;
let searcher = reader.searcher();
let query_parser = QueryParser::for_index(&index, vec![title, url]);
let query = query_parser.parse_query(search_term)?;
let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
```

### Option 3: Simple Inverted Index
**Pros:**
- Lightweight implementation
- No external dependencies
- Good for substring matching

**Cons:**
- Need to implement from scratch
- Limited ranking capabilities
- May not scale as well as dedicated solutions

**Implementation:**
```rust
use std::collections::{HashMap, HashSet};

struct InvertedIndex {
    index: HashMap<String, HashSet<String>>, // term -> document IDs
}

impl InvertedIndex {
    fn add_document(&mut self, doc_id: String, content: &str) {
        for word in content.split_whitespace() {
            let term = word.to_lowercase();
            self.index.entry(term).or_default().insert(doc_id.clone());
        }
    }
    
    fn search(&self, query: &str) -> Vec<String> {
        // Implement search logic
    }
}
```

## Recommendations

### Short Term (Current Scale)
- **Status**: Current linear scan is acceptable
- **Action**: Monitor performance as data grows
- **Threshold**: Consider indexing when datasets exceed 5,000 items

### Medium Term (If Performance Issues Arise)
- **Option A**: Implement simple inverted index for history search
- **Option B**: Add sled secondary indexes for URL lookups
- **Priority**: History search is most likely to benefit from indexing

### Long Term (Large Scale)
- Consider Tantivy for production-grade full-text search
- Implement for all search types (bookmarks, history, pages)
- Add features like:
  - Fuzzy matching
  - Phrase search
  - Result ranking
  - Search suggestions

## Implementation Priority

1. **Low Priority**: Current implementation is adequate for typical usage
2. **Monitor**: Add performance metrics to track search latency
3. **Evaluate**: Reassess when history grows beyond 5,000 visits
4. **Implement**: Simple inverted index if performance degrades

## Alternative: Client-Side Indexing

Consider moving search indexing to the frontend:
- Use JavaScript libraries like FlexSearch or Lunr.js
- Benefits: Offload work from Rust backend, reduce IPC overhead
- Drawbacks: Increased memory usage in frontend, need to sync with backend

## Conclusion

The current linear scan implementation is appropriate for the expected scale of data. Indexing should be considered only if performance issues arise with larger datasets. Start with a simple inverted index if needed, and consider Tantivy for production-grade full-text search in the future.
