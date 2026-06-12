//! # Exercise 07: Dashboard Specification
//!
//! ## Learning Objective
//! Learn how to define monitoring dashboards as Rust structs. While dashboards
//! are typically configured in JSON or YAML (e.g., Grafana), defining them as
//! Rust types enables compile-time validation, version control, and programmatic
//! generation.
//!
//! ## Flash Sale Context
//! A flash sale operations center needs a dashboard showing: request rate,
//! error rate, latency percentiles, stock levels, revenue, and active users.
//! Defining the dashboard spec in code ensures it evolves alongside the metrics
//! it visualizes.
//!
//! ## Instructions
//! 1. Implement `Dashboard`, `Panel`, `Query`, and `Visualization` structs
//! 2. Implement `create_flash_sale_dashboard()` to build the full dashboard
//!
//! ## Hints
//! - A dashboard has a title, time range, and list of panels
//! - A panel has a title, position (grid), a query, and a visualization type
//! - A query has a metric name and optional label filters
//! - Visualization types: Graph, SingleStat, Table, Heatmap

use std::collections::HashMap;

/// Supported visualization types for dashboard panels.
#[derive(Debug, Clone, PartialEq)]
pub enum Visualization {
    /// Time-series line graph.
    Graph,
    /// Single number display (e.g., current stock).
    SingleStat,
    /// Tabular data display.
    Table,
    /// Heatmap for latency distribution.
    Heatmap,
}

/// A Prometheus query with optional label filters.
#[derive(Debug, Clone)]
pub struct Query {
    /// The metric name to query (e.g., "flash_sale_purchase_attempts_total").
    pub metric: String,
    /// Optional label filters (e.g., {"result": "success"}).
    pub filters: HashMap<String, String>,
    /// Optional rate interval for counter metrics (e.g., "5m").
    pub rate_interval: Option<String>,
    /// Optional aggregation function (e.g., "sum", "avg", "histogram_quantile").
    pub aggregation: Option<String>,
}

/// A single panel on a dashboard.
#[derive(Debug, Clone)]
pub struct Panel {
    /// Panel title.
    pub title: String,
    /// Grid position: (x, y, width, height) in grid units.
    pub position: (u32, u32, u32, u32),
    /// The data query for this panel.
    pub query: Query,
    /// How to visualize the data.
    pub visualization: Visualization,
    /// Optional unit for the y-axis (e.g., "req/s", "ms", "$").
    pub unit: Option<String>,
}

/// A monitoring dashboard.
#[derive(Debug, Clone)]
pub struct Dashboard {
    /// Dashboard title.
    pub title: String,
    /// Description of what this dashboard shows.
    pub description: String,
    /// Default time range (e.g., "1h", "6h", "24h").
    pub time_range: String,
    /// List of panels on the dashboard.
    pub panels: Vec<Panel>,
}

impl Dashboard {
    /// Validate the dashboard structure.
    ///
    /// Checks:
    /// - Title is non-empty
    /// - At least one panel exists
    /// - No panel titles are empty
    /// - No panels overlap in position
    ///
    /// # Returns
    /// `Ok(())` if valid, or an error description.
    pub fn validate(&self) -> Result<(), String> {
        // TODO: Check title is non-empty
        // TODO: Check panels is non-empty
        // TODO: Check all panel titles are non-empty
        // TODO: Check for overlapping panel positions
        todo!("Implement dashboard validation")
    }

    /// Count the number of panels by visualization type.
    ///
    /// # Returns
    /// A HashMap mapping visualization type names to counts.
    pub fn panel_count_by_type(&self) -> HashMap<String, usize> {
        // TODO: Iterate panels and count by visualization type
        todo!("Implement panel counting")
    }
}

/// Create the standard flash sale monitoring dashboard.
///
/// The dashboard should include panels for:
/// 1. Request rate (Graph, top-left)
/// 2. Error rate (Graph, top-right)
/// 3. Latency p50/p99 (Graph, middle-left)
/// 4. Current stock level (SingleStat, middle-right)
/// 5. Total revenue (SingleStat, bottom-left)
/// 6. Active users (SingleStat, bottom-middle)
/// 7. Purchase results breakdown (Table, bottom-right)
///
/// # Returns
/// A fully configured `Dashboard`.
pub fn create_flash_sale_dashboard() -> Dashboard {
    // TODO: Create a Dashboard with title, description, time_range
    // TODO: Add 7 panels with appropriate queries and visualizations
    todo!("Implement flash sale dashboard creation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dashboard_has_panels() {
        let dashboard = create_flash_sale_dashboard();
        assert_eq!(dashboard.panels.len(), 7, "Should have 7 panels");
    }

    #[test]
    fn test_dashboard_title() {
        let dashboard = create_flash_sale_dashboard();
        assert!(!dashboard.title.is_empty(), "Dashboard title should not be empty");
    }

    #[test]
    fn test_panel_titles_not_empty() {
        let dashboard = create_flash_sale_dashboard();
        for panel in &dashboard.panels {
            assert!(
                !panel.title.is_empty(),
                "Panel title should not be empty"
            );
        }
    }

    #[test]
    fn test_panels_have_valid_positions() {
        let dashboard = create_flash_sale_dashboard();
        for panel in &dashboard.panels {
            let (x, y, w, h) = panel.position;
            assert!(w > 0 && h > 0, "Panel dimensions must be positive");
            assert!(
                x + w <= 24 && y + h <= 12,
                "Panel should fit within standard 24x12 grid"
            );
        }
    }

    #[test]
    fn test_panels_have_queries() {
        let dashboard = create_flash_sale_dashboard();
        for panel in &dashboard.panels {
            assert!(
                !panel.query.metric.is_empty(),
                "Panel query metric should not be empty"
            );
        }
    }

    #[test]
    fn test_visualization_types_present() {
        let dashboard = create_flash_sale_dashboard();
        let counts = dashboard.panel_count_by_type();
        assert!(
            counts.contains_key("Graph"),
            "Should have Graph panels"
        );
        assert!(
            counts.contains_key("SingleStat"),
            "Should have SingleStat panels"
        );
        assert!(
            counts.contains_key("Table"),
            "Should have a Table panel"
        );
    }

    #[test]
    fn test_validate_passes() {
        let dashboard = create_flash_sale_dashboard();
        assert!(
            dashboard.validate().is_ok(),
            "Default dashboard should pass validation"
        );
    }

    #[test]
    fn test_validate_empty_title_fails() {
        let mut dashboard = create_flash_sale_dashboard();
        dashboard.title = String::new();
        assert!(dashboard.validate().is_err());
    }

    #[test]
    fn test_validate_no_panels_fails() {
        let mut dashboard = create_flash_sale_dashboard();
        dashboard.panels.clear();
        assert!(dashboard.validate().is_err());
    }
}
