//! # Database Serialization
//!
//! Using serde for database interactions: converting between Rust types
//! and database representations. This covers patterns for SQL databases,
//! document stores, and key-value stores.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Demonstrates a database row representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
    #[serde(default)]
    pub metadata: Option<String>,
}

/// Demonstrates a type that converts between database and API representations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub is_active: bool,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            email: row.email,
            created_at: row.created_at,
            is_active: row.is_active,
        }
    }
}

/// Demonstrates a JSON column pattern: storing structured data in a text column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub user_id: i64,
    /// Stored as JSON string in the database
    #[serde(serialize_with = "serialize_json_string", deserialize_with = "deserialize_json_string")]
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    pub theme: String,
    pub language: String,
    pub notifications: bool,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn serialize_json_string<S: serde::Serializer>(
    value: &UserPreferences,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let json = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&json)
}

fn deserialize_json_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<UserPreferences, D::Error> {
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

/// Demonstrates an insert DTO (Data Transfer Object) without auto-generated fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

/// Demonstrates an update DTO with optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

impl UpdateUserRequest {
    pub fn to_update_sql(&self) -> (String, Vec<String>) {
        let mut sets = Vec::new();
        let mut values = Vec::new();
        let mut idx = 1;

        if let Some(ref username) = self.username {
            sets.push(format!("username = ${idx}"));
            values.push(username.clone());
            idx += 1;
        }
        if let Some(ref email) = self.email {
            sets.push(format!("email = ${idx}"));
            values.push(email.clone());
            idx += 1;
        }
        if let Some(is_active) = self.is_active {
            sets.push(format!("is_active = ${idx}"));
            values.push(is_active.to_string());
        }

        (sets.join(", "), values)
    }
}

/// Demonstrates a query builder pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryBuilder {
    pub table: String,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub order_by: Vec<OrderBy>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub column: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub column: String,
    pub ascending: bool,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        QueryBuilder {
            table: table.to_string(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn where_eq(mut self, column: &str, value: serde_json::Value) -> Self {
        self.conditions.push(Condition {
            column: column.to_string(),
            operator: "=".to_string(),
            value,
        });
        self
    }

    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        self.order_by.push(OrderBy {
            column: column.to_string(),
            ascending,
        });
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let clauses: Vec<String> = self
                .conditions
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{} {} ${}", c.column, c.operator, i + 1))
                .collect();
            sql.push_str(&clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|o| {
                    if o.ascending {
                        format!("{} ASC", o.column)
                    } else {
                        format!("{} DESC", o.column)
                    }
                })
                .collect();
            sql.push_str(&clauses.join(", "));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        sql
    }
}

/// Demonstrates a document store pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document<T: Serialize> {
    pub _id: String,
    pub _rev: Option<String>,
    #[serde(flatten)]
    pub content: T,
}

/// Demonstrates a key-value store entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KvEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub ttl: Option<u64>,
    pub created_at: u64,
}

/// Demonstrates a migration record for tracking schema changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationRecord {
    pub version: u32,
    pub name: String,
    pub applied_at: String,
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_row() {
        let row = UserRow {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: "hash123".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            is_active: true,
            metadata: None,
        };
        let json = serde_json::to_string(&row).unwrap();
        let back: UserRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row, back);
    }

    #[test]
    fn test_user_from_row() {
        let row = UserRow {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            is_active: true,
            metadata: None,
        };
        let user: User = row.into();
        assert_eq!(user.username, "alice");
    }

    #[test]
    fn test_user_profile_json_column() {
        let profile = UserProfile {
            user_id: 1,
            preferences: UserPreferences {
                theme: "dark".to_string(),
                language: "en".to_string(),
                notifications: true,
                extra: HashMap::new(),
            },
        };
        let json = serde_json::to_string(&profile).unwrap();
        // preferences should be serialized as a JSON string
        assert!(json.contains("\"theme"));

        let back: UserProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.preferences.theme, "dark");
    }

    #[test]
    fn test_update_request() {
        let req = UpdateUserRequest {
            username: Some("bob".to_string()),
            email: None,
            is_active: Some(false),
        };
        let (sets, values) = req.to_update_sql();
        assert!(sets.contains("username"));
        assert!(sets.contains("is_active"));
        assert!(!sets.contains("email"));
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("users")
            .where_eq("is_active", serde_json::json!(true))
            .order_by("created_at", false)
            .limit(10);

        let sql = query.to_sql();
        assert!(sql.contains("SELECT * FROM users"));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_query_builder_serializable() {
        let query = QueryBuilder::new("posts")
            .where_eq("author_id", serde_json::json!(42));

        let json = serde_json::to_string(&query).unwrap();
        let back: QueryBuilder = serde_json::from_str(&json).unwrap();
        assert_eq!(back.table, "posts");
        assert_eq!(back.conditions.len(), 1);
    }

    #[test]
    fn test_create_user_request() {
        let req = CreateUserRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: "hashed".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("id")); // No id in insert
        assert!(!json.contains("created_at")); // No timestamps
    }

    #[test]
    fn test_document() {
        let doc = Document {
            _id: "user:1".to_string(),
            _rev: Some("1-abc".to_string()),
            content: User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: "2024-01-01".to_string(),
                is_active: true,
            },
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("_id"));
        assert!(json.contains("username"));
    }
}
