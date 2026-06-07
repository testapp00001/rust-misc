//! # Technical Debt Management
//!
//! Technical debt is the implied cost of rework caused by choosing an easy
//! solution now instead of a better approach. This module covers identifying,
//! tracking, and prioritizing technical debt.
//!
//! ## Debt Categories:
//!
//! - **Deliberate**: Known shortcuts taken intentionally
//! - **Inadvertent**: Code that wasn't well-designed
//! - **Bit rot**: Code that degraded over time
//!
//! ## TODO/FIXME Conventions:
//!
//! - `TODO:` - Planned improvement
//! - `FIXME:` - Known bug that needs fixing
//! - `HACK:` - Workaround that should be removed
//! - `XXX:` - Warning about problematic code

use std::collections::HashMap;

/// Technical debt item.
#[derive(Debug, Clone)]
pub struct DebtItem {
    pub id: String,
    pub category: DebtCategory,
    pub description: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub severity: DebtSeverity,
    pub effort_hours: f64,
    pub created_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebtCategory {
    Deliberate,
    Inadvertent,
    BitRot,
    Dependency,
    Testing,
    Documentation,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DebtSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DebtItem {
    pub fn new(id: &str, category: DebtCategory, description: &str, severity: DebtSeverity) -> Self {
        Self {
            id: id.into(),
            category,
            description: description.into(),
            file: None,
            line: None,
            severity,
            effort_hours: 0.0,
            created_at: "2024-01-01".into(),
            tags: Vec::new(),
        }
    }

    pub fn with_location(mut self, file: &str, line: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    pub fn with_effort(mut self, hours: f64) -> Self {
        self.effort_hours = hours;
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Calculate priority score (higher = more urgent).
    pub fn priority_score(&self) -> f64 {
        let severity_weight = match self.severity {
            DebtSeverity::Critical => 10.0,
            DebtSeverity::High => 7.0,
            DebtSeverity::Medium => 4.0,
            DebtSeverity::Low => 1.0,
        };
        // Higher score for less effort (quicker wins)
        let effort_factor = if self.effort_hours > 0.0 {
            1.0 / self.effort_hours
        } else {
            1.0
        };
        severity_weight * effort_factor * 10.0
    }
}

/// TODO/FIXME scanner.
pub struct TodoScanner {
    markers: Vec<String>,
}

impl TodoScanner {
    pub fn new() -> Self {
        Self {
            markers: vec![
                "TODO".into(),
                "FIXME".into(),
                "HACK".into(),
                "XXX".into(),
                "OPTIMIZE".into(),
            ],
        }
    }

    /// Scan a line for TODO markers.
    pub fn scan_line(&self, line: &str) -> Option<TodoMarker> {
        for marker in &self.markers {
            if let Some(pos) = line.find(marker.as_str()) {
                let after = &line[pos + marker.len()..];
                let message = if after.starts_with(':') {
                    after[1..].trim().to_string()
                } else if after.starts_with(' ') {
                    after.trim().to_string()
                } else {
                    continue;
                };
                return Some(TodoMarker {
                    marker: marker.clone(),
                    message,
                    line_content: line.trim().to_string(),
                });
            }
        }
        None
    }

    /// Scan multiple lines.
    pub fn scan_lines(&self, lines: &[&str]) -> Vec<(usize, TodoMarker)> {
        lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| self.scan_line(line).map(|m| (i + 1, m)))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TodoMarker {
    pub marker: String,
    pub message: String,
    pub line_content: String,
}

/// Debt tracker for managing technical debt items.
pub struct DebtTracker {
    items: Vec<DebtItem>,
    next_id: usize,
}

impl DebtTracker {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_item(&mut self, mut item: DebtItem) -> String {
        let id = format!("DEBT-{:04}", self.next_id);
        self.next_id += 1;
        item.id = id.clone();
        self.items.push(item);
        id
    }

    pub fn get_item(&self, id: &str) -> Option<&DebtItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn total_effort(&self) -> f64 {
        self.items.iter().map(|i| i.effort_hours).sum()
    }

    pub fn by_severity(&self, severity: &DebtSeverity) -> Vec<&DebtItem> {
        self.items
            .iter()
            .filter(|i| &i.severity == severity)
            .collect()
    }

    pub fn by_category(&self, category: &DebtCategory) -> Vec<&DebtItem> {
        self.items
            .iter()
            .filter(|i| &i.category == category)
            .collect()
    }

    /// Get items sorted by priority.
    pub fn prioritized(&self) -> Vec<&DebtItem> {
        let mut items: Vec<&DebtItem> = self.items.iter().collect();
        items.sort_by(|a, b| {
            b.priority_score()
                .partial_cmp(&a.priority_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items
    }

    pub fn summary(&self) -> DebtSummary {
        DebtSummary {
            total_items: self.items.len(),
            critical: self.by_severity(&DebtSeverity::Critical).len(),
            high: self.by_severity(&DebtSeverity::High).len(),
            medium: self.by_severity(&DebtSeverity::Medium).len(),
            low: self.by_severity(&DebtSeverity::Low).len(),
            total_effort_hours: self.total_effort(),
        }
    }
}

#[derive(Debug)]
pub struct DebtSummary {
    pub total_items: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub total_effort_hours: f64,
}

impl DebtSummary {
    pub fn report(&self) -> String {
        format!(
            "Technical Debt Report\n\
             ====================\n\
             Total items: {}\n\
             Critical: {}\n\
             High: {}\n\
             Medium: {}\n\
             Low: {}\n\
             Total effort: {:.1} hours",
            self.total_items, self.critical, self.high, self.medium, self.low, self.total_effort_hours,
        )
    }
}

/// Debt ratio calculator.
pub struct DebtRatio {
    pub total_lines: usize,
    pub debt_lines: usize,
}

impl DebtRatio {
    pub fn ratio(&self) -> f64 {
        if self.total_lines == 0 {
            return 0.0;
        }
        self.debt_lines as f64 / self.total_lines as f64
    }

    pub fn percentage(&self) -> f64 {
        self.ratio() * 100.0
    }

    pub fn health_rating(&self) -> &str {
        let pct = self.percentage();
        if pct < 1.0 {
            "Excellent"
        } else if pct < 3.0 {
            "Good"
        } else if pct < 5.0 {
            "Fair"
        } else {
            "Needs Attention"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debt_item_priority() {
        let critical_quick = DebtItem::new(
            "1",
            DebtCategory::Deliberate,
            "quick fix",
            DebtSeverity::Critical,
        )
        .with_effort(1.0);

        let low_long = DebtItem::new(
            "2",
            DebtCategory::BitRot,
            "big refactor",
            DebtSeverity::Low,
        )
        .with_effort(40.0);

        assert!(critical_quick.priority_score() > low_long.priority_score());
    }

    #[test]
    fn test_debt_item_with_location() {
        let item = DebtItem::new("1", DebtCategory::Inadvertent, "test", DebtSeverity::Medium)
            .with_location("src/main.rs", 42)
            .with_tag("performance");

        assert_eq!(item.file, Some("src/main.rs".into()));
        assert_eq!(item.line, Some(42));
        assert!(item.tags.contains(&"performance".to_string()));
    }

    #[test]
    fn test_todo_scanner() {
        let scanner = TodoScanner::new();
        let marker = scanner.scan_line("// TODO: fix this later").unwrap();
        assert_eq!(marker.marker, "TODO");
        assert_eq!(marker.message, "fix this later");
    }

    #[test]
    fn test_todo_scanner_fixme() {
        let scanner = TodoScanner::new();
        let marker = scanner.scan_line("// FIXME: broken logic").unwrap();
        assert_eq!(marker.marker, "FIXME");
    }

    #[test]
    fn test_todo_scanner_no_marker() {
        let scanner = TodoScanner::new();
        assert!(scanner.scan_line("// just a comment").is_none());
    }

    #[test]
    fn test_todo_scanner_scan_lines() {
        let scanner = TodoScanner::new();
        let lines = vec![
            "// normal code",
            "// TODO: improve this",
            "// more code",
            "// FIXME: bug here",
        ];
        let markers = scanner.scan_lines(&lines);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].0, 2); // Line 2
        assert_eq!(markers[1].0, 4); // Line 4
    }

    #[test]
    fn test_debt_tracker() {
        let mut tracker = DebtTracker::new();
        let id = tracker.add_item(
            DebtItem::new("", DebtCategory::Deliberate, "test debt", DebtSeverity::High)
                .with_effort(4.0),
        );

        assert!(id.starts_with("DEBT-"));
        assert_eq!(tracker.total_effort(), 4.0);
    }

    #[test]
    fn test_debt_tracker_by_severity() {
        let mut tracker = DebtTracker::new();
        tracker.add_item(DebtItem::new("", DebtCategory::Deliberate, "a", DebtSeverity::Critical));
        tracker.add_item(DebtItem::new("", DebtCategory::Deliberate, "b", DebtSeverity::High));
        tracker.add_item(DebtItem::new("", DebtCategory::Deliberate, "c", DebtSeverity::Critical));

        assert_eq!(tracker.by_severity(&DebtSeverity::Critical).len(), 2);
        assert_eq!(tracker.by_severity(&DebtSeverity::High).len(), 1);
    }

    #[test]
    fn test_debt_tracker_prioritized() {
        let mut tracker = DebtTracker::new();
        tracker.add_item(
            DebtItem::new("", DebtCategory::Deliberate, "low effort critical", DebtSeverity::Critical)
                .with_effort(1.0),
        );
        tracker.add_item(
            DebtItem::new("", DebtCategory::BitRot, "high effort low", DebtSeverity::Low)
                .with_effort(40.0),
        );

        let prioritized = tracker.prioritized();
        assert_eq!(prioritized[0].severity, DebtSeverity::Critical);
    }

    #[test]
    fn test_debt_summary() {
        let mut tracker = DebtTracker::new();
        tracker.add_item(DebtItem::new("", DebtCategory::Deliberate, "a", DebtSeverity::Critical));
        tracker.add_item(DebtItem::new("", DebtCategory::Deliberate, "b", DebtSeverity::Low));

        let summary = tracker.summary();
        assert_eq!(summary.total_items, 2);
        assert_eq!(summary.critical, 1);
        assert_eq!(summary.low, 1);
    }

    #[test]
    fn test_debt_ratio() {
        let ratio = DebtRatio {
            total_lines: 1000,
            debt_lines: 20,
        };
        assert!((ratio.percentage() - 2.0).abs() < f64::EPSILON);
        assert_eq!(ratio.health_rating(), "Good");
    }

    #[test]
    fn test_debt_ratio_health() {
        // 0.5% debt is "Excellent" (< 1.0%)
        assert_eq!(
            DebtRatio { total_lines: 1000, debt_lines: 5 }.health_rating(),
            "Excellent"
        );
        // 5.0% debt is "Needs Attention" (>= 5.0%)
        assert_eq!(
            DebtRatio { total_lines: 1000, debt_lines: 50 }.health_rating(),
            "Needs Attention"
        );
    }

    #[test]
    fn test_debt_tracker_by_category() {
        let mut tracker = DebtTracker::new();
        tracker.add_item(DebtItem::new("", DebtCategory::Testing, "test debt", DebtSeverity::Medium));
        tracker.add_item(DebtItem::new("", DebtCategory::Documentation, "doc debt", DebtSeverity::Low));

        assert_eq!(tracker.by_category(&DebtCategory::Testing).len(), 1);
        assert_eq!(tracker.by_category(&DebtCategory::Documentation).len(), 1);
    }
}
