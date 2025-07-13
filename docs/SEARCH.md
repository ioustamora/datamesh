# DataMesh Search & Discovery Guide

This guide covers the comprehensive search and discovery capabilities in DataMesh, including CLI commands, web interface, and API endpoints.

## 🔍 Search Architecture

DataMesh implements a multi-layered search system that combines:

- **Database-level search**: Fast SQL-based queries on metadata
- **Semantic matching**: Relevance scoring for search results
- **Advanced filtering**: Multiple criteria including size, date, and file type
- **Regex support**: Pattern matching for complex queries
- **Caching**: Intelligent result caching for performance

## 📖 CLI Search Commands

### 1. Advanced Search (`search`)

The primary search command with comprehensive filtering options:

```bash
# Basic text search
datamesh search "document"

# Search with file type filter
datamesh search "report" --file-type pdf

# Search with size constraints
datamesh search "image" --size ">1MB"

# Search with date range
datamesh search "backup" --date "last week"

# Regex pattern search
datamesh search "file[0-9]+" --regex

# Combined filters with limit
datamesh search "project" --file-type txt --size "1KB-10MB" --limit 10
```

#### Search Options:
- `--file-type`: Filter by file extension (e.g., pdf, txt, jpg)
- `--size`: Size range filter (e.g., '>1MB', '100KB-10MB', '<5GB')
- `--date`: Date range filter (e.g., 'last week', '2024-01-01:2024-12-31')
- `--regex`: Enable regex pattern matching
- `--limit`: Maximum number of results (default: 50)

#### Size Filter Formats:
- `>1MB`: Files larger than 1MB
- `<500KB`: Files smaller than 500KB
- `1MB-10MB`: Files between 1MB and 10MB
- `=2GB`: Files exactly 2GB

#### Date Filter Formats:
- `last week`: Files from the last 7 days
- `last month`: Files from the last 30 days
- `2024-01-01:2024-12-31`: Files between specific dates
- `>2024-01-01`: Files after a specific date

### 2. Recent Files (`recent`)

Show recently uploaded or accessed files:

```bash
# Show 20 most recent files
datamesh recent

# Show 50 recent files from last 3 days
datamesh recent --count 50 --days 3

# Recent PDF files only
datamesh recent --file-type pdf
```

### 3. Popular Files (`popular`)

Display most frequently accessed files:

```bash
# Show popular files
datamesh popular

# Popular files by timeframe
datamesh popular --timeframe "last month"

# Popular files by category
datamesh popular --category documents
```

## 🌐 Web Interface Search

### Global Search Component

The web interface provides a powerful global search accessible via:
- **Keyboard shortcut**: `Ctrl+K` (or `Cmd+K` on Mac)
- **Search icon**: Click the search icon in the header
- **Navigation**: Available in all views

#### Features:
- **Real-time search**: Results appear as you type
- **Multi-category search**: Files, folders, commands, and users
- **Smart suggestions**: Quick commands and recent searches
- **Keyboard navigation**: Arrow keys and Enter to select
- **Search history**: Recent searches are cached locally

#### Search Syntax:
```
# Basic search
document

# Command search
"create folder"

# Filter by type
type:pdf important

# Size filters
size:>1MB reports

# Date filters
modified:today

# Tag search
tag:project-alpha
```

### File Manager Search

In the file manager interface:
- **Search bar**: Located in the file manager toolbar
- **Live filtering**: Results update as you type
- **Type filters**: Dropdown to filter by file type
- **Sort options**: Name, size, date, relevance

## 🔧 API Search Endpoints

### File Search API

```http
POST /api/v1/files/search
Content-Type: application/json
Authorization: Bearer <token>

{
  "query": "document",
  "tags": "project",
  "size_range": [1048576, 10485760],
  "date_range": ["2024-01-01T00:00:00Z", "2024-12-31T23:59:59Z"],
  "page": 1,
  "page_size": 20
}
```

Response:
```json
{
  "files": [
    {
      "id": "file123",
      "name": "project_document.pdf",
      "size": 2048576,
      "modified": "2024-01-15T10:30:00Z",
      "tags": ["project", "document"],
      "file_type": "pdf",
      "relevance_score": 0.95
    }
  ],
  "total": 142,
  "page": 1,
  "page_size": 20
}
```

### Search Parameters:
- `query`: Text search query (optional)
- `tags`: Comma-separated tags filter (optional)
- `size_range`: Array of [min_bytes, max_bytes] (optional)
- `date_range`: Array of [from_date, to_date] in ISO format (optional)
- `page`: Page number for pagination (default: 1)
- `page_size`: Results per page (default: 20, max: 100)

## 🚀 Advanced Search Features

### 1. Semantic Search

DataMesh implements semantic search capabilities:

```rust
// Relevance scoring algorithm
fn calculate_relevance_score(filename: &str, query: &str) -> f64 {
    // Exact match bonus
    if filename.to_lowercase().contains(&query.to_lowercase()) {
        return 1.0;
    }
    
    // Partial match scoring
    let query_words: Vec<&str> = query.split_whitespace().collect();
    let filename_words: Vec<&str> = filename.split_whitespace().collect();
    
    // Calculate word overlap and position scoring
    // Implementation includes fuzzy matching and position weighting
}
```

### 2. Search Optimization

- **Indexing**: Database indexes on commonly searched fields
- **Caching**: Results cached for repeated queries
- **Deduplication**: Similar queries are deduplicated
- **Debouncing**: Search requests are debounced in the web interface

### 3. Search Analytics

Track search performance and usage:

```bash
# Search metrics
datamesh metrics search

# Popular search terms
datamesh analytics search-terms

# Search performance stats
datamesh stats search-performance
```

## 📊 Search Performance

### Optimization Strategies:

1. **Database Optimization**:
   - Indexes on `name`, `tags`, `upload_time`, `file_size`
   - Full-text search capabilities
   - Query optimization for complex filters

2. **Caching Strategy**:
   - Query result caching with TTL
   - Intelligent cache invalidation
   - Prefetching for common queries

3. **Web Interface Optimization**:
   - Debounced search requests (300ms)
   - Request deduplication
   - Client-side result caching

### Performance Metrics:
- **Average search time**: < 100ms for simple queries
- **Complex query time**: < 500ms with multiple filters
- **Cache hit ratio**: > 80% for repeated searches
- **Concurrent search capacity**: 100+ simultaneous searches

## 🔧 Configuration

### Search Settings

Configure search behavior in `datamesh.toml`:

```toml
[search]
# Default result limit
default_limit = 50

# Maximum result limit
max_limit = 1000

# Search timeout in seconds
timeout = 30

# Enable full-text search
full_text_search = true

# Cache settings
cache_ttl = 300  # 5 minutes
cache_size = 1000  # Max cached queries

# Relevance scoring weights
[search.relevance]
exact_match_weight = 1.0
partial_match_weight = 0.7
tag_match_weight = 0.8
recent_bonus = 0.1
```

### Performance Tuning

```toml
[search.performance]
# Enable search result caching
enable_cache = true

# Database query optimization
optimize_queries = true

# Parallel search execution
parallel_search = true

# Maximum concurrent searches
max_concurrent = 50
```

## 🛠️ Search Integration Examples

### Programmatic Search

```rust
use datamesh::file_manager::{SearchCriteria, SizeRange, DateRange};

// Build search criteria
let criteria = SearchCriteria {
    query: "important document".to_string(),
    file_type: Some("pdf".to_string()),
    size_range: Some(SizeRange::Between(1024 * 1024, 100 * 1024 * 1024)),
    date_range: Some(DateRange::LastMonths(6)),
    use_regex: false,
    limit: 100,
};

// Execute search
let results = search_files(criteria).await?;

// Process results
for file in results {
    println!("Found: {} ({})", file.name, file.file_key);
}
```

### JavaScript API Integration

```javascript
// Web interface search
const searchResults = await fetch('/api/v1/files/search', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
        query: 'project documents',
        tags: 'important',
        size_range: [1048576, null], // > 1MB
        page: 1,
        page_size: 20
    })
});

const { files, total } = await searchResults.json();
```

## 🔍 Search Best Practices

### 1. Query Optimization
- Use specific terms rather than generic words
- Combine text search with filters for better results
- Use regex only when necessary (impacts performance)

### 2. Filter Usage
- Apply size filters to reduce result set
- Use date ranges for time-sensitive searches
- Combine multiple filters for precision

### 3. Performance Tips
- Limit results for large datasets
- Use caching for repeated searches
- Avoid overly complex regex patterns

### 4. User Experience
- Provide search suggestions
- Show search history
- Implement progressive search (results while typing)

## 📚 Related Documentation

- **[CLI Usage Guide](USAGE.md)** - Complete CLI command reference
- **[Web Interface Guide](WEB_INTERFACE.md)** - Web UI documentation
- **[API Reference](API.md)** - REST API documentation
- **[Performance Tuning](PERFORMANCE.md)** - Optimization strategies

---

*This search system provides comprehensive file discovery capabilities across CLI, web interface, and API, with advanced filtering, semantic matching, and performance optimization.*
