//! # Query Optimization
//!
//! Efficient queries are critical for application performance. This lesson covers
//! index strategies, N+1 query detection, batch loading, cursor pagination, and
//! query analysis patterns.
//!
//! ## Key Concepts
//! - Index design and usage
//! - N+1 query problem and solutions
//! - Batch loading / eager loading
//! - Cursor-based pagination
//! - Query plan analysis
//! - Denormalization strategies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Query Plan Analysis
// ---------------------------------------------------------------------------

/// Represents an execution plan for a query.
#[derive(Debug, Clone, Serialize)]
pub struct QueryPlan {
    pub table: String,
    pub scan_type: ScanType,
    pub rows_examined: u64,
    pub rows_returned: u64,
    pub index_used: Option<String>,
    pub filters_applied: Vec<String>,
    pub cost_estimate: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ScanType {
    /// Full table scan (bad for large tables).
    FullTableScan,
    /// Index scan (good).
    IndexScan,
    /// Index range scan (good for ranges).
    IndexRangeScan,
    /// Point lookup using primary key (best).
    PrimaryKeyLookup,
    /// Using covering index (best for read queries).
    CoveringIndex,
}

impl QueryPlan {
    /// Analyze whether this query plan is efficient.
    pub fn is_efficient(&self) -> bool {
        match self.scan_type {
            ScanType::FullTableScan => self.rows_examined < 1000,
            ScanType::IndexScan | ScanType::IndexRangeScan => {
                self.rows_examined <= self.rows_returned * 10
            }
            ScanType::PrimaryKeyLookup | ScanType::CoveringIndex => true,
        }
    }

    /// Get optimization suggestions.
    pub fn suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.scan_type == ScanType::FullTableScan && self.rows_examined > 1000 {
            suggestions.push(format!(
                "Add an index on '{}' for the filtered columns",
                self.table
            ));
        }

        if self.rows_examined > self.rows_returned * 100 {
            suggestions.push("Consider adding a more selective index".into());
        }

        if self.index_used.is_none() && !self.filters_applied.is_empty() {
            suggestions.push(format!(
                "Query filters on {:?} but no index is used",
                self.filters_applied
            ));
        }

        suggestions
    }

    pub fn summary(&self) -> String {
        format!(
            "{}: {} scan, {} rows examined, {} returned, index: {}",
            self.table,
            match self.scan_type {
                ScanType::FullTableScan => "FULL TABLE",
                ScanType::IndexScan => "INDEX",
                ScanType::IndexRangeScan => "INDEX RANGE",
                ScanType::PrimaryKeyLookup => "PK LOOKUP",
                ScanType::CoveringIndex => "COVERING INDEX",
            },
            self.rows_examined,
            self.rows_returned,
            self.index_used.as_deref().unwrap_or("none")
        )
    }
}

// ---------------------------------------------------------------------------
// 2. Index Advisor
// ---------------------------------------------------------------------------

/// Recommends indexes based on query patterns.
#[derive(Debug)]
pub struct IndexAdvisor {
    queries: Vec<QueryPattern>,
}

#[derive(Debug, Clone)]
pub struct QueryPattern {
    pub table: String,
    pub where_columns: Vec<String>,
    pub order_columns: Vec<String>,
    pub frequency: u64,
}

impl IndexAdvisor {
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
        }
    }

    pub fn record_query(&mut self, pattern: QueryPattern) {
        self.queries.push(pattern);
    }

    /// Generate index recommendations.
    pub fn recommend(&self) -> Vec<IndexRecommendation> {
        let mut recommendations = Vec::new();

        // Group by table
        let mut table_queries: HashMap<String, Vec<&QueryPattern>> = HashMap::new();
        for query in &self.queries {
            table_queries
                .entry(query.table.clone())
                .or_default()
                .push(query);
        }

        for (table, queries) in &table_queries {
            // Recommend indexes for frequently queried columns
            let mut column_frequency: HashMap<String, u64> = HashMap::new();
            for query in queries {
                for col in &query.where_columns {
                    *column_frequency.entry(col.clone()).or_default() += query.frequency;
                }
            }

            for (column, frequency) in &column_frequency {
                if *frequency >= 5 {
                    recommendations.push(IndexRecommendation {
                        table: table.clone(),
                        columns: vec![column.clone()],
                        estimated_benefit: (*frequency as f64).log2(),
                        reason: format!("column '{column}' used in {frequency} queries"),
                    });
                }
            }

            // Recommend composite indexes for WHERE + ORDER BY
            for query in queries {
                if !query.where_columns.is_empty() && !query.order_columns.is_empty() {
                    let mut cols = query.where_columns.clone();
                    cols.extend(query.order_columns.clone());
                    recommendations.push(IndexRecommendation {
                        table: table.clone(),
                        columns: cols.clone(),
                        estimated_benefit: (query.frequency as f64 * 2.0).log2(),
                        reason: format!(
                            "composite index for WHERE {:?} + ORDER BY {:?}",
                            query.where_columns, query.order_columns
                        ),
                    });
                }
            }
        }

        // Sort by benefit
        recommendations.sort_by(|a, b| {
            b.estimated_benefit
                .partial_cmp(&a.estimated_benefit)
                .unwrap()
        });

        recommendations
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexRecommendation {
    pub table: String,
    pub columns: Vec<String>,
    pub estimated_benefit: f64,
    pub reason: String,
}

impl IndexRecommendation {
    pub fn to_create_sql(&self) -> String {
        let index_name = format!(
            "idx_{}_{}",
            self.table,
            self.columns.join("_")
        );
        let cols = self.columns.join(", ");
        format!("CREATE INDEX {index_name} ON {} ({cols});", self.table)
    }
}

// ---------------------------------------------------------------------------
// 3. N+1 Query Detector
// ---------------------------------------------------------------------------

/// Detects N+1 query patterns by counting queries.
#[derive(Debug)]
pub struct QueryCounter {
    queries: Vec<QueryRecord>,
}

#[derive(Debug, Clone)]
pub struct QueryRecord {
    pub sql: String,
    pub timestamp: u64,
    pub duration_ms: u64,
}

impl QueryCounter {
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
        }
    }

    pub fn record(&mut self, sql: impl Into<String>) {
        self.queries.push(QueryRecord {
            sql: sql.into(),
            timestamp: self.queries.len() as u64,
            duration_ms: 0,
        });
    }

    pub fn query_count(&self) -> usize {
        self.queries.len()
    }

    /// Detect potential N+1 patterns: many similar queries in sequence.
    pub fn detect_n_plus_one(&self) -> Vec<NPlusOneWarning> {
        let mut warnings = Vec::new();

        // Look for repeated query patterns (same structure, different parameters)
        let mut pattern_counts: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, query) in self.queries.iter().enumerate() {
            let pattern = normalize_query(&query.sql);
            pattern_counts.entry(pattern).or_default().push(i);
        }

        for (pattern, indices) in &pattern_counts {
            if indices.len() > 3 {
                warnings.push(NPlusOneWarning {
                    pattern: pattern.clone(),
                    occurrences: indices.len(),
                    suggestion: "Consider using batch loading or JOIN instead".into(),
                });
            }
        }

        warnings
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.queries.iter().map(|q| q.duration_ms).sum()
    }
}

#[derive(Debug)]
pub struct NPlusOneWarning {
    pub pattern: String,
    pub occurrences: usize,
    pub suggestion: String,
}

/// Normalize a query by replacing parameters with placeholders.
fn normalize_query(sql: &str) -> String {
    let mut normalized = sql.to_lowercase();
    // Replace numbers with ?
    let mut result = String::new();
    let mut in_number = false;
    for ch in normalized.chars() {
        if ch.is_ascii_digit() {
            if !in_number {
                result.push('?');
                in_number = true;
            }
        } else {
            in_number = false;
            result.push(ch);
        }
    }
    // Replace quoted strings
    result = result
        .split('\'')
        .enumerate()
        .map(|(i, s)| if i % 2 == 1 { "?" } else { s })
        .collect::<Vec<_>>()
        .join("?");
    result
}

// ---------------------------------------------------------------------------
// 4. Cursor Pagination
// /// Cursor-based pagination for efficient traversal of large datasets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Encode a cursor from an ID and timestamp.
pub fn encode_cursor(id: u64, timestamp: u64) -> String {
    use base64::Engine;
    let raw = format!("{id}:{timestamp}");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

/// Decode a cursor.
pub fn decode_cursor(cursor: &str) -> Result<(u64, u64), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|e| format!("invalid cursor: {e}"))?;
    let raw = String::from_utf8(bytes).map_err(|e| format!("invalid cursor: {e}"))?;
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 2 {
        return Err("invalid cursor format".into());
    }
    let id = parts[0].parse().map_err(|_| "invalid cursor id")?;
    let ts = parts[1].parse().map_err(|_| "invalid cursor timestamp")?;
    Ok((id, ts))
}

// ---------------------------------------------------------------------------
// 5. Batch Loader
// /// Loads related entities in batches to avoid N+1 queries.
#[derive(Debug)]
pub struct BatchLoader<K, V> {
    pending_keys: Vec<K>,
    loaded: HashMap<K, V>,
    batch_size: usize,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> BatchLoader<K, V> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            pending_keys: Vec::new(),
            loaded: HashMap::new(),
            batch_size,
        }
    }

    /// Queue a key for loading.
    pub fn load(&mut self, key: K) {
        if !self.loaded.contains_key(&key) {
            self.pending_keys.push(key);
        }
    }

    /// Check if there are pending loads.
    pub fn has_pending(&self) -> bool {
        !self.pending_keys.is_empty()
    }

    /// Get the next batch of keys to load.
    pub fn next_batch(&mut self) -> Vec<K> {
        let batch: Vec<K> = self.pending_keys.drain(..self.batch_size.min(self.pending_keys.len())).collect();
        batch
    }

    /// Mark keys as loaded.
    pub fn mark_loaded(&mut self, key: K, value: V) {
        self.loaded.insert(key, value);
    }

    /// Get a loaded value.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.loaded.get(key)
    }

    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_plan_efficient() {
        let plan = QueryPlan {
            table: "users".into(),
            scan_type: ScanType::PrimaryKeyLookup,
            rows_examined: 1,
            rows_returned: 1,
            index_used: Some("PRIMARY".into()),
            filters_applied: vec![],
            cost_estimate: 0.01,
        };
        assert!(plan.is_efficient());
    }

    #[test]
    fn test_query_plan_inefficient() {
        let plan = QueryPlan {
            table: "users".into(),
            scan_type: ScanType::FullTableScan,
            rows_examined: 100000,
            rows_returned: 10,
            index_used: None,
            filters_applied: vec!["email".into()],
            cost_estimate: 100.0,
        };
        assert!(!plan.is_efficient());
    }

    #[test]
    fn test_query_plan_suggestions() {
        let plan = QueryPlan {
            table: "users".into(),
            scan_type: ScanType::FullTableScan,
            rows_examined: 10000,
            rows_returned: 5,
            index_used: None,
            filters_applied: vec!["email".into()],
            cost_estimate: 50.0,
        };
        let suggestions = plan.suggestions();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_query_plan_summary() {
        let plan = QueryPlan {
            table: "orders".into(),
            scan_type: ScanType::IndexRangeScan,
            rows_examined: 100,
            rows_returned: 50,
            index_used: Some("idx_created_at".into()),
            filters_applied: vec![],
            cost_estimate: 1.0,
        };
        let summary = plan.summary();
        assert!(summary.contains("INDEX RANGE"));
        assert!(summary.contains("100"));
    }

    #[test]
    fn test_index_advisor() {
        let mut advisor = IndexAdvisor::new();

        for _ in 0..10 {
            advisor.record_query(QueryPattern {
                table: "users".into(),
                where_columns: vec!["email".into()],
                order_columns: vec![],
                frequency: 1,
            });
        }

        advisor.record_query(QueryPattern {
            table: "users".into(),
            where_columns: vec!["status".into()],
            order_columns: vec!["created_at".into()],
            frequency: 5,
        });

        let recommendations = advisor.recommend();
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_index_recommendation_sql() {
        let rec = IndexRecommendation {
            table: "users".into(),
            columns: vec!["email".into()],
            estimated_benefit: 5.0,
            reason: "test".into(),
        };
        let sql = rec.to_create_sql();
        assert!(sql.contains("CREATE INDEX"));
        assert!(sql.contains("idx_users_email"));
        assert!(sql.contains("ON users (email)"));
    }

    #[test]
    fn test_query_counter_n_plus_one() {
        let mut counter = QueryCounter::new();

        // Simulate N+1 pattern
        for i in 0..10 {
            counter.record(format!("SELECT * FROM orders WHERE user_id = {i}"));
        }

        let warnings = counter.detect_n_plus_one();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_query_counter_no_n_plus_one() {
        let mut counter = QueryCounter::new();
        counter.record("SELECT * FROM users");
        counter.record("SELECT * FROM orders");

        let warnings = counter.detect_n_plus_one();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_cursor_encode_decode() {
        let cursor = encode_cursor(42, 1700000000);
        let (id, ts) = decode_cursor(&cursor).unwrap();
        assert_eq!(id, 42);
        assert_eq!(ts, 1700000000);
    }

    #[test]
    fn test_cursor_decode_invalid() {
        assert!(decode_cursor("not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_batch_loader() {
        let mut loader = BatchLoader::new(3);

        loader.load("a");
        loader.load("b");
        loader.load("c");
        loader.load("d");

        assert!(loader.has_pending());

        let batch = loader.next_batch();
        assert_eq!(batch.len(), 3);

        for key in &batch {
            loader.mark_loaded(*key, format!("value_{key}"));
        }

        assert_eq!(loader.loaded_count(), 3);
        assert!(loader.has_pending());

        let batch2 = loader.next_batch();
        assert_eq!(batch2.len(), 1);
    }

    #[test]
    fn test_batch_loader_get() {
        let mut loader = BatchLoader::new(10);
        loader.load("key");
        let batch = loader.next_batch();
        assert_eq!(batch, vec!["key"]);

        loader.mark_loaded("key", "value".to_string());
        assert_eq!(loader.get(&"key"), Some(&"value".to_string()));
        assert!(loader.get(&"missing").is_none());
    }

    #[test]
    fn test_normalize_query() {
        let q1 = "SELECT * FROM users WHERE id = 42";
        let q2 = "SELECT * FROM users WHERE id = 99";
        assert_eq!(normalize_query(q1), normalize_query(q2));

        let q3 = "SELECT * FROM users WHERE name = 'Alice'";
        let q4 = "SELECT * FROM users WHERE name = 'Bob'";
        assert_eq!(normalize_query(q3), normalize_query(q4));
    }

    #[test]
    fn test_cursor_page() {
        let page = CursorPage {
            data: vec![1, 2, 3],
            next_cursor: Some("cursor_abc".into()),
            has_more: true,
        };
        assert_eq!(page.data.len(), 3);
        assert!(page.has_more);
    }
}
