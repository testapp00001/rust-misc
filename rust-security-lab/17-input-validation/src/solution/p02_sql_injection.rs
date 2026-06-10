//! # Lesson 02: SQL Injection (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Text(String),
    Boolean(bool),
}

impl SqlValue {
    pub fn to_sql_literal(&self) -> String {
        match self {
            SqlValue::Null => "NULL".to_string(),
            SqlValue::Integer(n) => n.to_string(),
            SqlValue::Text(s) => {
                let escaped = s.replace('\'', "''");
                format!("'{}'", escaped)
            }
            SqlValue::Boolean(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
        }
    }
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

const ALLOWED_OPERATORS: &[&str] = &["=", "!=", "<", ">", "<=", ">=", "LIKE"];

pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<(String, String, SqlValue)>,
    limit: Option<usize>,
}

impl QueryBuilder {
    pub fn select(table: &str) -> Result<Self, String> {
        if !is_valid_identifier(table) {
            return Err(format!("Invalid table name: '{}'", table));
        }
        Ok(Self {
            table: table.to_string(),
            columns: Vec::new(),
            conditions: Vec::new(),
            limit: None,
        })
    }

    pub fn column(mut self, name: &str) -> Result<Self, String> {
        if name != "*" && !is_valid_identifier(name) {
            return Err(format!("Invalid column name: '{}'", name));
        }
        self.columns.push(name.to_string());
        Ok(self)
    }

    pub fn where_clause(
        mut self,
        column: &str,
        operator: &str,
        value: SqlValue,
    ) -> Result<Self, String> {
        if !is_valid_identifier(column) {
            return Err(format!("Invalid column name: '{}'", column));
        }
        let op_upper = operator.to_uppercase();
        if !ALLOWED_OPERATORS.contains(&op_upper.as_str()) {
            return Err(format!("Invalid operator: '{}'", operator));
        }
        self.conditions
            .push((column.to_string(), operator.to_string(), value));
        Ok(self)
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn build(self) -> (String, Vec<SqlValue>) {
        let cols = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut query = format!("SELECT {} FROM {}", cols, self.table);
        let mut params = Vec::new();

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let clauses: Vec<String> = self
                .conditions
                .iter()
                .map(|(col, op, _)| format!("{} {} ?", col, op))
                .collect();
            query.push_str(&clauses.join(" AND "));
            for (_, _, val) in &self.conditions {
                params.push(val.clone());
            }
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        (query, params)
    }
}

pub fn detect_sql_injection(input: &str) -> bool {
    let lower = input.to_lowercase();

    let keywords = [
        "select", "insert", "update", "delete", "drop", "union",
        "where", "exec", "execute", " or ", " and ",
    ];

    for keyword in &keywords {
        if lower.contains(keyword) {
            return true;
        }
    }

    if lower.contains("--") || lower.contains("/*") || lower.contains(';') {
        return true;
    }

    if lower.contains("' or '") || lower.contains("\" or \"") || lower.contains("' and '") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_value_escape_quotes() {
        let val = SqlValue::Text("O'Brien".to_string());
        assert_eq!(val.to_sql_literal(), "'O''Brien'");
    }

    #[test]
    fn test_sql_value_null() {
        assert_eq!(SqlValue::Null.to_sql_literal(), "NULL");
    }

    #[test]
    fn test_sql_value_integer() {
        assert_eq!(SqlValue::Integer(42).to_sql_literal(), "42");
    }

    #[test]
    fn test_query_builder_simple() {
        let (query, params) = QueryBuilder::select("users")
            .unwrap()
            .column("*")
            .unwrap()
            .where_clause("name", "=", SqlValue::Text("alice".into()))
            .unwrap()
            .build();
        assert!(query.contains("SELECT * FROM users"));
        assert!(query.contains("WHERE name = ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_query_builder_rejects_bad_table_name() {
        assert!(QueryBuilder::select("users; DROP TABLE users--").is_err());
    }

    #[test]
    fn test_query_builder_rejects_bad_operator() {
        let result = QueryBuilder::select("users")
            .unwrap()
            .where_clause("name", "; DROP", SqlValue::Text("x".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_injection_union() {
        assert!(detect_sql_injection("' UNION SELECT * FROM passwords--"));
    }

    #[test]
    fn test_detect_injection_or_true() {
        assert!(detect_sql_injection("' OR '1'='1'"));
    }

    #[test]
    fn test_detect_injection_clean_input() {
        assert!(!detect_sql_injection("alice@example.com"));
    }
}
