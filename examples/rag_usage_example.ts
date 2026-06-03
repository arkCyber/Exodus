/**
 * Exodus Browser — RAG Usage Example
 * Demonstrates how to use RAG functionality for semantic search and page indexing
 */

import * as ragClient from '../src/lib/ragClient';

/**
 * Example 1: Basic page indexing and keyword search
 */
async function exampleBasicIndexing() {
  console.log('=== Example 1: Basic Page Indexing ===');
  
  // Start RAG service
  const socketPath = await ragClient.startRagService('/tmp/exodus-rag-data');
  console.log('RAG service started at:', socketPath);
  
  // Store a page
  const pageId = await ragClient.storeRagPage(
    'https://example.com/docs/rust',
    'Rust Programming Language Documentation',
    'Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.'
  );
  console.log('Page stored with ID:', pageId);
  
  // Search by keyword
  const results = await ragClient.searchRagPages('rust');
  console.log('Search results:', results);
  
  // Stop service
  await ragClient.stopRagService();
  console.log('RAG service stopped');
}

/**
 * Example 2: Semantic search with embeddings
 */
async function exampleSemanticSearch() {
  console.log('\n=== Example 2: Semantic Search ===');
  
  // Start RAG service
  await ragClient.startRagService('/tmp/exodus-rag-data');
  
  // Store pages with embeddings (requires Allama embeddings API)
  try {
    await ragClient.generateAndStoreRagPage(
      'https://example.com/docs/python',
      'Python Documentation',
      'Python is an interpreted, high-level, general-purpose programming language.'
    );
    
    await ragClient.generateAndStoreRagPage(
      'https://example.com/docs/javascript',
      'JavaScript Documentation',
      'JavaScript is a programming language that is one of the core technologies of the World Wide Web.'
    );
    
    // Semantic search using embedding vector
    // Note: In real usage, you would generate the query embedding using Allama
    const queryEmbedding = [0.1, 0.2, 0.3, 0.4]; // Placeholder embedding
    const semanticResults = await ragClient.searchRagSemantic(queryEmbedding);
    console.log('Semantic search results:', semanticResults);
  } catch (error) {
    console.error('Semantic search requires Allama embeddings API:', error);
  }
  
  await ragClient.stopRagService();
}

/**
 * Example 3: Hybrid search (keyword + semantic)
 */
async function exampleHybridSearch() {
  console.log('\n=== Example 3: Hybrid Search ===');
  
  await ragClient.startRagService('/tmp/exodus-rag-data');
  
  // Store pages
  await ragClient.storeRagPage(
    'https://example.com/blog/web-development',
    'Web Development Guide',
    'Learn modern web development with HTML, CSS, and JavaScript.'
  );
  
  await ragClient.storeRagPage(
    'https://example.com/blog/mobile-development',
    'Mobile Development Guide',
    'Build mobile apps with React Native and Flutter.'
  );
  
  // Hybrid search with keyword and optional embedding
  const hybridResults = await ragClient.searchRagHybrid('development');
  console.log('Hybrid search results:', hybridResults);
  
  await ragClient.stopRagService();
}

/**
 * Example 4: Bookmark management
 */
async function exampleBookmarks() {
  console.log('\n=== Example 4: Bookmark Management ===');
  
  await ragClient.startRagService('/tmp/exodus-rag-data');
  
  // Add bookmarks
  await ragClient.addRagBookmark('https://rust-lang.org', 'Rust Language', 'Programming');
  await ragClient.addRagBookmark('https://python.org', 'Python Language', 'Programming');
  await ragClient.addRagBookmark('https://example.com', 'Example Site', '');
  
  // List bookmarks
  const bookmarks = await ragClient.listRagBookmarks();
  console.log('Bookmarks:', bookmarks);
  
  await ragClient.stopRagService();
}

/**
 * Example 5: Visit history tracking
 */
async function exampleVisitHistory() {
  console.log('\n=== Example 5: Visit History ===');
  
  await ragClient.startRagService('/tmp/exodus-rag-data');
  
  // Record visits
  await ragClient.recordRagVisit('https://example.com/page1', 'Page 1');
  await ragClient.recordRagVisit('https://example.com/page2', 'Page 2');
  await ragClient.recordRagVisit('https://example.com/page1', 'Page 1'); // Duplicate visit
  
  // Search visits
  const visits = await ragClient.searchRagVisits('page');
  console.log('Visit history:', visits);
  
  await ragClient.stopRagService();
}

/**
 * Example 6: Complete RAG workflow
 */
async function exampleCompleteWorkflow() {
  console.log('\n=== Example 6: Complete RAG Workflow ===');
  
  // 1. Start service
  await ragClient.startRagService('/tmp/exodus-rag-data');
  
  // 2. Index browsing history
  const pages = [
    { url: 'https://docs.rs', title: 'Rust Documentation', content: 'Official Rust docs' },
    { url: 'https://developer.mozilla.org', title: 'MDN Web Docs', content: 'Web development documentation' },
    { url: 'https://stackoverflow.com', title: 'Stack Overflow', content: 'Q&A for programmers' },
  ];
  
  for (const page of pages) {
    await ragClient.storeRagPage(page.url, page.title, page.content);
    console.log('Indexed:', page.title);
  }
  
  // 3. Record visits
  for (const page of pages) {
    await ragClient.recordRagVisit(page.url, page.title);
  }
  
  // 4. Add bookmarks
  await ragClient.addRagBookmark(pages[0].url, pages[0].title, 'Documentation');
  await ragClient.addRagBookmark(pages[1].url, pages[1].title, 'Documentation');
  
  // 5. Search
  const searchResults = await ragClient.searchRagPages('documentation');
  console.log('Search results for "documentation":', searchResults);
  
  // 6. List bookmarks
  const bookmarks = await ragClient.listRagBookmarks();
  console.log('Bookmarks in Documentation folder:', bookmarks.filter((b: any) => b.folder === 'Documentation'));
  
  // 7. Stop service
  await ragClient.stopRagService();
  console.log('Workflow complete');
}

/**
 * Run all examples
 */
async function runExamples() {
  try {
    await exampleBasicIndexing();
    await exampleSemanticSearch();
    await exampleHybridSearch();
    await exampleBookmarks();
    await exampleVisitHistory();
    await exampleCompleteWorkflow();
    console.log('\n=== All examples completed successfully ===');
  } catch (error) {
    console.error('Example failed:', error);
  }
}

// Export for use in other files
export {
  exampleBasicIndexing,
  exampleSemanticSearch,
  exampleHybridSearch,
  exampleBookmarks,
  exampleVisitHistory,
  exampleCompleteWorkflow,
  runExamples
};
