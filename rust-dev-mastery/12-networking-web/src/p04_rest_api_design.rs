//! # REST API Design
//!
//! Designing clean, predictable, and evolvable REST APIs is a critical skill.
//! This lesson covers resource naming conventions, HTTP method semantics,
//! status code usage, pagination, filtering, versioning, and HATEOAS.
//!
//! ## Key Concepts
//! - Resource-oriented URL design
//! - HTTP methods and their semantics (GET, POST, PUT, PATCH, DELETE)
//! - Proper status code selection
//! - Pagination patterns (offset, cursor, keyset)
//! - Filtering, sorting, and field selection
//! - API versioning strategies
//! - HATEOAS (Hypermedia)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Resource Models
// ---------------------------------------------------------------------------

/// Represents an API resource with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource<T: Serialize> {
    pub id: String,
    #[serde(flatten)]
    pub data: T,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<ResourceLinks>,
}

/// HATEOAS-style links for resource discoverability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLinks {
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub name: String,
    pub description: String,
    pub price_cents: u64,
    pub currency: String,
    pub category: String,
    pub in_stock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub status: OrderStatus,
    pub total_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

// ---------------------------------------------------------------------------
// 2. Pagination Patterns
// ---------------------------------------------------------------------------

/// Offset-based pagination (simple but has issues with large datasets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl OffsetPagination {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }

    pub fn offset(&self) -> u64 {
        ((self.page - 1) * self.per_page) as u64
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

/// Cursor-based pagination (better for real-time data and large datasets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPagination<T: Serialize> {
    pub data: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
    pub has_more: bool,
}

impl<T: Serialize> CursorPagination<T> {
    pub fn new(data: Vec<T>, next_cursor: Option<String>, has_more: bool) -> Self {
        Self {
            data,
            next_cursor,
            prev_cursor: None,
            has_more,
        }
    }
}

/// Keyset pagination query parameters.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub cursor: Option<String>,
    #[serde(default)]
    pub use_cursor: bool,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

// ---------------------------------------------------------------------------
// 3. Filtering and Sorting
// ---------------------------------------------------------------------------

/// Generic filter parameter for query strings.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FilterParams {
    pub search: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<u64>,
    pub max_price: Option<u64>,
    pub in_stock: Option<bool>,
    #[serde(default)]
    pub sort_by: SortField,
    #[serde(default)]
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    #[default]
    Name,
    Price,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

/// Apply filters to a collection of products (demonstrates the pattern).
pub fn filter_products(products: &[Resource<Product>], filters: &FilterParams) -> Vec<Resource<Product>> {
    let mut result: Vec<Resource<Product>> = products
        .iter()
        .filter(|p| {
            if let Some(ref search) = filters.search {
                let search_lower = search.to_lowercase();
                if !p.data.name.to_lowercase().contains(&search_lower)
                    && !p.data.description.to_lowercase().contains(&search_lower)
                {
                    return false;
                }
            }

            if let Some(ref category) = filters.category {
                if &p.data.category != category {
                    return false;
                }
            }

            if let Some(min) = filters.min_price {
                if p.data.price_cents < min {
                    return false;
                }
            }

            if let Some(max) = filters.max_price {
                if p.data.price_cents > max {
                    return false;
                }
            }

            if let Some(in_stock) = filters.in_stock {
                if p.data.in_stock != in_stock {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    // Sort
    match filters.sort_by {
        SortField::Name => result.sort_by(|a, b| a.data.name.cmp(&b.data.name)),
        SortField::Price => result.sort_by(|a, b| a.data.price_cents.cmp(&b.data.price_cents)),
        SortField::CreatedAt => result.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        SortField::UpdatedAt => result.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
    }

    if filters.sort_order == SortOrder::Desc {
        result.reverse();
    }

    result
}

// ---------------------------------------------------------------------------
// 4. API Response Envelope
// ---------------------------------------------------------------------------

/// Standard API response envelope for successful responses.
#[derive(Debug, Serialize)]
pub struct ApiResponseEnvelope<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,
}

impl<T: Serialize> ApiResponseEnvelope<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            pagination: None,
            links: None,
        }
    }

    pub fn with_pagination(mut self, pagination: serde_json::Value) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn with_links(mut self, links: HashMap<String, String>) -> Self {
        self.links = Some(links);
        self
    }
}

/// Standard error response envelope.
#[derive(Debug, Serialize)]
pub struct ErrorResponseEnvelope {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl ErrorResponseEnvelope {
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ErrorDetail {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn with_details(code: u16, message: impl Into<String>, details: Vec<FieldError>) -> Self {
        Self {
            success: false,
            error: ErrorDetail {
                code,
                message: message.into(),
                details: Some(details),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// 5. API Versioning
// ---------------------------------------------------------------------------

/// Represents an API version extracted from the request.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}

impl ApiVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Parse version from URL path like "/v1/users" or "/v2.1/users".
    pub fn from_path(path: &str) -> Option<Self> {
        let segment = path
            .trim_start_matches('/')
            .split('/')
            .find(|s| s.starts_with('v'))?;
        let version_str = segment.strip_prefix('v')?;
        let parts: Vec<&str> = version_str.split('.').collect();
        match parts.as_slice() {
            [major] => major.parse().ok().map(|m| Self::new(m, 0)),
            [major, minor] => {
                let m = major.parse().ok()?;
                let n = minor.parse().ok()?;
                Some(Self::new(m, n))
            }
            _ => None,
        }
    }

    /// Parse version from Accept header like "application/vnd.api+json;version=2".
    pub fn from_accept_header(header: &str) -> Option<Self> {
        let version_part = header
            .split(';')
            .find(|s| s.trim().starts_with("version="))?;
        let version_str = version_part.trim().strip_prefix("version=")?;
        let parts: Vec<&str> = version_str.split('.').collect();
        match parts.as_slice() {
            [major] => major.parse().ok().map(|m| Self::new(m, 0)),
            [major, minor] => {
                let m = major.parse().ok()?;
                let n = minor.parse().ok()?;
                Some(Self::new(m, n))
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Resource URL Builder
// ---------------------------------------------------------------------------

/// Builds RESTful resource URLs with proper conventions.
pub struct ResourceUrlBuilder {
    base: String,
    version: Option<u32>,
}

impl ResourceUrlBuilder {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            version: None,
        }
    }

    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    pub fn collection(&self, resource: &str) -> String {
        match self.version {
            Some(v) => format!("{}/v{}/{resource}", self.base, v),
            None => format!("{}/{resource}", self.base),
        }
    }

    pub fn item(&self, resource: &str, id: &str) -> String {
        match self.version {
            Some(v) => format!("{}/v{}/{resource}/{id}", self.base, v),
            None => format!("{}/{resource}/{id}", self.base),
        }
    }

    pub fn sub_resource(&self, resource: &str, id: &str, sub: &str) -> String {
        match self.version {
            Some(v) => format!("{}/v{}/{resource}/{id}/{sub}", self.base, v),
            None => format!("{}/{resource}/{id}/{sub}", self.base),
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Content Negotiation
// ---------------------------------------------------------------------------

/// Supported response formats.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseFormat {
    Json,
    Csv,
    Xml,
}

impl ResponseFormat {
    /// Parse from Accept header value.
    pub fn from_accept(accept: &str) -> Self {
        if accept.contains("text/csv") {
            ResponseFormat::Csv
        } else if accept.contains("application/xml") || accept.contains("text/xml") {
            ResponseFormat::Xml
        } else {
            ResponseFormat::Json
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            ResponseFormat::Json => "application/json",
            ResponseFormat::Csv => "text/csv",
            ResponseFormat::Xml => "application/xml",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_product(id: &str, name: &str, price: u64, category: &str) -> Resource<Product> {
        Resource {
            id: id.into(),
            data: Product {
                name: name.into(),
                description: format!("Description for {name}"),
                price_cents: price,
                currency: "USD".into(),
                category: category.into(),
                in_stock: true,
            },
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
            links: None,
        }
    }

    #[test]
    fn test_resource_serialization() {
        let product = sample_product("1", "Widget", 2999, "tools");
        let json = serde_json::to_value(&product).unwrap();
        assert_eq!(json["id"], "1");
        assert_eq!(json["name"], "Widget");
        assert_eq!(json["price_cents"], 2999);
        assert_eq!(json["category"], "tools");
    }

    #[test]
    fn test_offset_pagination() {
        let pag = OffsetPagination::new(2, 10, 95);
        assert_eq!(pag.total_pages, 10);
        assert_eq!(pag.offset(), 10);
        assert!(pag.has_next());
        assert!(pag.has_prev());
    }

    #[test]
    fn test_offset_pagination_first_page() {
        let pag = OffsetPagination::new(1, 10, 95);
        assert!(!pag.has_prev());
        assert!(pag.has_next());
        assert_eq!(pag.offset(), 0);
    }

    #[test]
    fn test_offset_pagination_last_page() {
        let pag = OffsetPagination::new(10, 10, 95);
        assert!(pag.has_prev());
        assert!(!pag.has_next());
    }

    #[test]
    fn test_cursor_pagination() {
        let data = vec!["a", "b", "c"];
        let pag = CursorPagination::new(data, Some("cursor_abc".into()), true);
        assert_eq!(pag.data.len(), 3);
        assert_eq!(pag.next_cursor.as_deref(), Some("cursor_abc"));
        assert!(pag.has_more);
    }

    #[test]
    fn test_filter_by_search() {
        let products = vec![
            sample_product("1", "Rust Book", 3999, "books"),
            sample_product("2", "Python Guide", 2999, "books"),
            sample_product("3", "Rust Toolkit", 4999, "tools"),
        ];

        let filters = FilterParams {
            search: Some("rust".into()),
            ..Default::default()
        };

        let result = filter_products(&products, &filters);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|p| p.data.name.to_lowercase().contains("rust")));
    }

    #[test]
    fn test_filter_by_category() {
        let products = vec![
            sample_product("1", "Widget A", 100, "tools"),
            sample_product("2", "Widget B", 200, "gadgets"),
            sample_product("3", "Widget C", 300, "tools"),
        ];

        let filters = FilterParams {
            category: Some("tools".into()),
            ..Default::default()
        };

        let result = filter_products(&products, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_by_price_range() {
        let products = vec![
            sample_product("1", "Cheap", 500, "misc"),
            sample_product("2", "Mid", 1500, "misc"),
            sample_product("3", "Expensive", 5000, "misc"),
        ];

        let filters = FilterParams {
            min_price: Some(1000),
            max_price: Some(2000),
            ..Default::default()
        };

        let result = filter_products(&products, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.name, "Mid");
    }

    #[test]
    fn test_filter_sort_by_price_asc() {
        let products = vec![
            sample_product("1", "C", 300, "misc"),
            sample_product("2", "A", 100, "misc"),
            sample_product("3", "B", 200, "misc"),
        ];

        let filters = FilterParams {
            sort_by: SortField::Price,
            sort_order: SortOrder::Asc,
            ..Default::default()
        };

        let result = filter_products(&products, &filters);
        assert_eq!(result[0].data.name, "A");
        assert_eq!(result[1].data.name, "B");
        assert_eq!(result[2].data.name, "C");
    }

    #[test]
    fn test_filter_sort_by_name_desc() {
        let products = vec![
            sample_product("1", "Alpha", 100, "misc"),
            sample_product("2", "Gamma", 300, "misc"),
            sample_product("3", "Beta", 200, "misc"),
        ];

        let filters = FilterParams {
            sort_by: SortField::Name,
            sort_order: SortOrder::Desc,
            ..Default::default()
        };

        let result = filter_products(&products, &filters);
        assert_eq!(result[0].data.name, "Gamma");
        assert_eq!(result[1].data.name, "Beta");
        assert_eq!(result[2].data.name, "Alpha");
    }

    #[test]
    fn test_filter_no_filters() {
        let products = vec![
            sample_product("1", "A", 100, "misc"),
            sample_product("2", "B", 200, "misc"),
        ];

        let filters = FilterParams::default();
        let result = filter_products(&products, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_api_version_from_path() {
        assert_eq!(ApiVersion::from_path("/v1/users"), Some(ApiVersion::new(1, 0)));
        assert_eq!(
            ApiVersion::from_path("/v2.1/orders"),
            Some(ApiVersion::new(2, 1))
        );
        assert_eq!(ApiVersion::from_path("/users"), None);
        assert_eq!(ApiVersion::from_path("/api/v3/items"), Some(ApiVersion::new(3, 0)));
    }

    #[test]
    fn test_api_version_from_accept_header() {
        assert_eq!(
            ApiVersion::from_accept_header("application/vnd.api+json;version=2"),
            Some(ApiVersion::new(2, 0))
        );
        assert_eq!(
            ApiVersion::from_accept_header("application/json;version=1.5"),
            Some(ApiVersion::new(1, 5))
        );
        assert_eq!(
            ApiVersion::from_accept_header("application/json"),
            None
        );
    }

    #[test]
    fn test_resource_url_builder() {
        let builder = ResourceUrlBuilder::new("https://api.example.com").version(1);
        assert_eq!(builder.collection("users"), "https://api.example.com/v1/users");
        assert_eq!(
            builder.item("users", "42"),
            "https://api.example.com/v1/users/42"
        );
        assert_eq!(
            builder.sub_resource("users", "42", "orders"),
            "https://api.example.com/v1/users/42/orders"
        );
    }

    #[test]
    fn test_resource_url_builder_no_version() {
        let builder = ResourceUrlBuilder::new("https://api.example.com");
        assert_eq!(builder.collection("items"), "https://api.example.com/items");
    }

    #[test]
    fn test_content_negotiation() {
        assert_eq!(
            ResponseFormat::from_accept("application/json"),
            ResponseFormat::Json
        );
        assert_eq!(
            ResponseFormat::from_accept("text/csv"),
            ResponseFormat::Csv
        );
        assert_eq!(
            ResponseFormat::from_accept("application/xml"),
            ResponseFormat::Xml
        );
        assert_eq!(
            ResponseFormat::from_accept("text/html"),
            ResponseFormat::Json // default fallback
        );
    }

    #[test]
    fn test_content_type_header() {
        assert_eq!(ResponseFormat::Json.content_type(), "application/json");
        assert_eq!(ResponseFormat::Csv.content_type(), "text/csv");
        assert_eq!(ResponseFormat::Xml.content_type(), "application/xml");
    }

    #[test]
    fn test_error_response_envelope() {
        let err = ErrorResponseEnvelope::new(404, "user not found");
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"]["code"], 404);
        assert_eq!(json["error"]["message"], "user not found");
        assert!(json["error"]["details"].is_null());
    }

    #[test]
    fn test_error_response_with_field_errors() {
        let details = vec![
            FieldError {
                field: "email".into(),
                message: "invalid format".into(),
            },
            FieldError {
                field: "name".into(),
                message: "too short".into(),
            },
        ];
        let err = ErrorResponseEnvelope::with_details(422, "validation failed", details);
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["error"]["code"], 422);
        let details = json["error"]["details"].as_array().unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0]["field"], "email");
    }

    #[test]
    fn test_success_envelope() {
        let env = ApiResponseEnvelope::success("hello");
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], "hello");
        assert!(json["pagination"].is_null());
    }

    #[test]
    fn test_success_envelope_with_pagination() {
        let pag = serde_json::json!({"page": 1, "total": 100});
        let env = ApiResponseEnvelope::success(vec![1, 2, 3]).with_pagination(pag);
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["pagination"]["page"], 1);
    }

    #[test]
    fn test_success_envelope_with_links() {
        let mut links = HashMap::new();
        links.insert("self".into(), "/api/v1/users".into());
        links.insert("next".into(), "/api/v1/users?page=2".into());

        let env: ApiResponseEnvelope<Vec<String>> = ApiResponseEnvelope::success(vec![]).with_links(links);
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["links"]["self"], "/api/v1/users");
    }

    #[test]
    fn test_order_status_serde() {
        assert_eq!(
            serde_json::to_string(&OrderStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&OrderStatus::Shipped).unwrap(),
            "\"shipped\""
        );

        let status: OrderStatus = serde_json::from_str("\"delivered\"").unwrap();
        assert_eq!(status, OrderStatus::Delivered);
    }

    #[test]
    fn test_resource_with_links() {
        let mut related = HashMap::new();
        related.insert("store".into(), "/api/v1/stores/s1".into());

        let product = Resource {
            id: "1".into(),
            data: Product {
                name: "Widget".into(),
                description: "A widget".into(),
                price_cents: 999,
                currency: "USD".into(),
                category: "tools".into(),
                in_stock: true,
            },
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
            links: Some(ResourceLinks {
                self_link: Some("/api/v1/products/1".into()),
                next: None,
                prev: None,
                related: Some(related),
            }),
        };

        let json = serde_json::to_value(&product).unwrap();
        assert_eq!(json["links"]["self"], "/api/v1/products/1");
        assert_eq!(json["links"]["related"]["store"], "/api/v1/stores/s1");
    }

    #[test]
    fn test_pagination_params_defaults() {
        let params = PaginationParams {
            page: 0, // will be corrected in handler
            per_page: 100, // will be capped in handler
            cursor: None,
            use_cursor: false,
        };
        assert_eq!(params.page, 0);
        assert_eq!(params.per_page, 100);
    }
}
