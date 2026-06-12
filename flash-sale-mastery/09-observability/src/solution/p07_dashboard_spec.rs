//! # Solution 07: Dashboard Specification
//!
//! Complete implementation of monitoring dashboard specification structs.

use std::collections::HashMap;

/// Supported visualization types for dashboard panels.
#[derive(Debug, Clone, PartialEq)]
pub enum Visualization {
    Graph,
    SingleStat,
    Table,
    Heatmap,
}

/// A Prometheus query with optional label filters.
#[derive(Debug, Clone)]
pub struct Query {
    pub metric: String,
    pub filters: HashMap<String, String>,
    pub rate_interval: Option<String>,
    pub aggregation: Option<String>,
}

/// A single panel on a dashboard.
#[derive(Debug, Clone)]
pub struct Panel {
    pub title: String,
    pub position: (u32, u32, u32, u32),
    pub query: Query,
    pub visualization: Visualization,
    pub unit: Option<String>,
}

/// A monitoring dashboard.
#[derive(Debug, Clone)]
pub struct Dashboard {
    pub title: String,
    pub description: String,
    pub time_range: String,
    pub panels: Vec<Panel>,
}

impl Dashboard {
    /// Validate the dashboard structure.
    pub fn validate(&self) -> Result<(), String> {
        if self.title.is_empty() {
            return Err("Dashboard title is empty".to_string());
        }

        if self.panels.is_empty() {
            return Err("Dashboard has no panels".to_string());
        }

        for panel in &self.panels {
            if panel.title.is_empty() {
                return Err("Panel has empty title".to_string());
            }
        }

        // Check for overlapping panels
        for i in 0..self.panels.len() {
            for j in (i + 1)..self.panels.len() {
                let a = &self.panels[i];
                let b = &self.panels[j];
                let (ax, ay, aw, ah) = a.position;
                let (bx, by, bw, bh) = b.position;

                let a_right = ax + aw;
                let a_bottom = ay + ah;
                let b_right = bx + bw;
                let b_bottom = by + bh;

                let overlaps = ax < b_right && a_right > bx && ay < b_bottom && a_bottom > by;
                if overlaps {
                    return Err(format!(
                        "Panels '{}' and '{}' overlap",
                        a.title, b.title
                    ));
                }
            }
        }

        Ok(())
    }

    /// Count the number of panels by visualization type.
    pub fn panel_count_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for panel in &self.panels {
            let key = format!("{:?}", panel.visualization);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

/// Helper to create a query with a metric name.
fn query(metric: &str) -> Query {
    Query {
        metric: metric.to_string(),
        filters: HashMap::new(),
        rate_interval: None,
        aggregation: None,
    }
}

/// Helper to create a rate query.
fn rate_query(metric: &str, interval: &str) -> Query {
    Query {
        metric: metric.to_string(),
        filters: HashMap::new(),
        rate_interval: Some(interval.to_string()),
        aggregation: Some("rate".to_string()),
    }
}

/// Create the standard flash sale monitoring dashboard.
pub fn create_flash_sale_dashboard() -> Dashboard {
    Dashboard {
        title: "Flash Sale Operations".to_string(),
        description: "Real-time monitoring dashboard for flash sale events".to_string(),
        time_range: "1h".to_string(),
        panels: vec![
            // Row 1: Request rate and error rate
            Panel {
                title: "Request Rate".to_string(),
                position: (0, 0, 12, 4),
                query: rate_query("flash_sale_purchase_attempts_total", "1m"),
                visualization: Visualization::Graph,
                unit: Some("req/s".to_string()),
            },
            Panel {
                title: "Error Rate".to_string(),
                position: (12, 0, 12, 4),
                query: rate_query("flash_sale_purchase_attempts_total{result=\"error\"}", "1m"),
                visualization: Visualization::Graph,
                unit: Some("%".to_string()),
            },
            // Row 2: Latency and stock
            Panel {
                title: "Latency (p50 / p99)".to_string(),
                position: (0, 4, 12, 4),
                query: Query {
                    metric: "flash_sale_request_latency_seconds".to_string(),
                    filters: HashMap::new(),
                    rate_interval: None,
                    aggregation: Some("histogram_quantile".to_string()),
                },
                visualization: Visualization::Graph,
                unit: Some("ms".to_string()),
            },
            Panel {
                title: "Current Stock Level".to_string(),
                position: (12, 4, 12, 4),
                query: query("flash_sale_stock_level"),
                visualization: Visualization::SingleStat,
                unit: Some("units".to_string()),
            },
            // Row 3: Revenue, active users, results table
            Panel {
                title: "Total Revenue".to_string(),
                position: (0, 8, 8, 4),
                query: query("flash_sale_revenue_total"),
                visualization: Visualization::SingleStat,
                unit: Some("$".to_string()),
            },
            Panel {
                title: "Active Users".to_string(),
                position: (8, 8, 8, 4),
                query: query("flash_sale_active_users"),
                visualization: Visualization::SingleStat,
                unit: Some("users".to_string()),
            },
            Panel {
                title: "Purchase Results".to_string(),
                position: (16, 8, 8, 4),
                query: query("flash_sale_purchase_attempts_total"),
                visualization: Visualization::Table,
                unit: None,
            },
        ],
    }
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
            assert!(!panel.title.is_empty(), "Panel title should not be empty");
        }
    }

    #[test]
    fn test_panels_have_valid_positions() {
        let dashboard = create_flash_sale_dashboard();
        for panel in &dashboard.panels {
            let (x, y, w, h) = panel.position;
            assert!(w > 0 && h > 0, "Panel dimensions must be positive");
            assert!(x + w <= 24 && y + h <= 12, "Panel should fit within standard 24x12 grid");
        }
    }

    #[test]
    fn test_panels_have_queries() {
        let dashboard = create_flash_sale_dashboard();
        for panel in &dashboard.panels {
            assert!(!panel.query.metric.is_empty(), "Panel query metric should not be empty");
        }
    }

    #[test]
    fn test_visualization_types_present() {
        let dashboard = create_flash_sale_dashboard();
        let counts = dashboard.panel_count_by_type();
        assert!(counts.contains_key("Graph"), "Should have Graph panels");
        assert!(counts.contains_key("SingleStat"), "Should have SingleStat panels");
        assert!(counts.contains_key("Table"), "Should have a Table panel");
    }

    #[test]
    fn test_validate_passes() {
        let dashboard = create_flash_sale_dashboard();
        assert!(dashboard.validate().is_ok(), "Default dashboard should pass validation");
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
