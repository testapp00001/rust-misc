//! # Lesson 08: Secure Database Migrations (Reference Solution)
//!
//! See the exercise file for full documentation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationOp {
    CreateTable { name: String, columns: Vec<ColumnDef> },
    AddColumn { table: String, column: ColumnDef },
    DropTable { name: String },
    CreateIndex { table: String, columns: Vec<String>, unique: bool },
    RawSql { sql: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub description: String,
    pub author: String,
    pub operations: Vec<MigrationOp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    ReadOnly,
    Standard,
    Admin,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Check if a name contains only safe characters (alphanumeric + underscore).
fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate a migration for security issues.
pub fn validate_migration(migration: &Migration) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if migration.description.is_empty() {
        warnings.push("Migration has no description".to_string());
    }
    if migration.author.is_empty() {
        warnings.push("Migration has no author".to_string());
    }

    for op in &migration.operations {
        match op {
            MigrationOp::CreateTable { name, columns } => {
                if !is_safe_name(name) {
                    errors.push(format!("Unsafe table name: '{}'", name));
                }
                for col in columns {
                    if !is_safe_name(&col.name) {
                        errors.push(format!("Unsafe column name: '{}'", col.name));
                    }
                }
            }
            MigrationOp::AddColumn { table, column } => {
                if !is_safe_name(table) {
                    errors.push(format!("Unsafe table name: '{}'", table));
                }
                if !is_safe_name(&column.name) {
                    errors.push(format!("Unsafe column name: '{}'", column.name));
                }
            }
            MigrationOp::DropTable { name } => {
                warnings.push(format!("DROP TABLE '{}' is a destructive operation", name));
                if !is_safe_name(name) {
                    errors.push(format!("Unsafe table name: '{}'", name));
                }
            }
            MigrationOp::CreateIndex { table, columns, .. } => {
                if !is_safe_name(table) {
                    errors.push(format!("Unsafe table name: '{}'", table));
                }
                for col in columns {
                    if !is_safe_name(col) {
                        errors.push(format!("Unsafe column name: '{}'", col));
                    }
                }
            }
            MigrationOp::RawSql { sql } => {
                warnings.push(format!(
                    "RawSql operation requires extra review: '{}'",
                    &sql[..sql.len().min(50)]
                ));
            }
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        warnings,
        errors,
    }
}

/// Check if a user has permission to run a migration.
pub fn check_permissions(migration: &Migration, level: PermissionLevel) -> Result<(), Vec<String>> {
    let mut denied = Vec::new();

    for op in &migration.operations {
        match (level, op) {
            (PermissionLevel::ReadOnly, _) => {
                denied.push("ReadOnly users cannot run migrations".to_string());
                break;
            }
            (PermissionLevel::Standard, MigrationOp::DropTable { name }) => {
                denied.push(format!(
                    "Standard users cannot DROP TABLE '{}'",
                    name
                ));
            }
            (PermissionLevel::Standard, MigrationOp::RawSql { .. }) => {
                denied.push("Standard users cannot run RawSql operations".to_string());
            }
            _ => {} // Admin can do anything; Standard can do the rest
        }
    }

    if denied.is_empty() {
        Ok(())
    } else {
        Err(denied)
    }
}

/// Generate SQL statements for a dry run.
pub fn dry_run_migration(migration: &Migration) -> Vec<String> {
    let mut statements = Vec::new();

    for op in &migration.operations {
        match op {
            MigrationOp::CreateTable { name, columns } => {
                let col_defs: Vec<String> = columns
                    .iter()
                    .map(|c| {
                        let nullable = if c.nullable { " NULL" } else { " NOT NULL" };
                        let default = c
                            .default
                            .as_ref()
                            .map(|d| format!(" DEFAULT {}", d))
                            .unwrap_or_default();
                        format!("{} {}{}{}", c.name, c.col_type, nullable, default)
                    })
                    .collect();
                statements.push(format!(
                    "CREATE TABLE {} ({});",
                    name,
                    col_defs.join(", ")
                ));
            }
            MigrationOp::AddColumn { table, column } => {
                let nullable = if column.nullable { " NULL" } else { " NOT NULL" };
                statements.push(format!(
                    "ALTER TABLE {} ADD COLUMN {} {} {};",
                    table, column.name, column.col_type, nullable
                ));
            }
            MigrationOp::DropTable { name } => {
                statements.push(format!("DROP TABLE {};", name));
            }
            MigrationOp::CreateIndex {
                table,
                columns,
                unique,
            } => {
                let unique_str = if *unique { "UNIQUE " } else { "" };
                let idx_name = format!("idx_{}_{}", table, columns.join("_"));
                statements.push(format!(
                    "CREATE {}INDEX {} ON {} ({});",
                    unique_str,
                    idx_name,
                    table,
                    columns.join(", ")
                ));
            }
            MigrationOp::RawSql { sql } => {
                statements.push(sql.clone());
            }
        }
    }

    statements
}

/// Create a JSON audit log entry for a migration.
pub fn log_migration(migration: &Migration) -> String {
    let log_entry = serde_json::json!({
        "event": "migration_executed",
        "version": migration.version,
        "description": migration.description,
        "author": migration.author,
        "operation_count": migration.operations.len(),
        "timestamp": "2024-01-01T00:00:00Z"
    });
    serde_json::to_string_pretty(&log_entry).unwrap_or_else(|_| "{}".to_string())
}

/// Demonstrate the full migration security pipeline.
pub fn demonstrate_migration_security() -> (bool, bool, bool) {
    let safe_migration = Migration {
        version: 1,
        description: "Create users table".to_string(),
        author: "alice".to_string(),
        operations: vec![MigrationOp::CreateTable {
            name: "users".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    col_type: "INTEGER".to_string(),
                    nullable: false,
                    default: None,
                },
                ColumnDef {
                    name: "name".to_string(),
                    col_type: "TEXT".to_string(),
                    nullable: false,
                    default: None,
                },
            ],
        }],
    };

    let dangerous_migration = Migration {
        version: 2,
        description: "Drop users table".to_string(),
        author: "bob".to_string(),
        operations: vec![MigrationOp::DropTable {
            name: "users".to_string(),
        }],
    };

    let validation = validate_migration(&safe_migration);
    let validation_valid = validation.valid;

    let standard_result = check_permissions(&dangerous_migration, PermissionLevel::Standard);
    let standard_denied = standard_result.is_err();

    let admin_result = check_permissions(&dangerous_migration, PermissionLevel::Admin);
    let admin_allowed = admin_result.is_ok();

    (validation_valid, standard_denied, admin_allowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_migration() -> Migration {
        Migration {
            version: 1,
            description: "Create users table".to_string(),
            author: "alice".to_string(),
            operations: vec![MigrationOp::CreateTable {
                name: "users".to_string(),
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        col_type: "INTEGER".to_string(),
                        nullable: false,
                        default: None,
                    },
                    ColumnDef {
                        name: "name".to_string(),
                        col_type: "TEXT".to_string(),
                        nullable: false,
                        default: None,
                    },
                ],
            }],
        }
    }

    fn dangerous_migration() -> Migration {
        Migration {
            version: 2,
            description: "Drop users".to_string(),
            author: "bob".to_string(),
            operations: vec![MigrationOp::DropTable {
                name: "users".to_string(),
            }],
        }
    }

    #[test]
    fn test_validate_safe_migration() {
        let result = validate_migration(&safe_migration());
        assert!(result.valid, "Safe migration should be valid");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_dangerous_migration() {
        let result = validate_migration(&dangerous_migration());
        assert!(
            !result.warnings.is_empty() || !result.errors.is_empty(),
            "DropTable should trigger warnings or errors"
        );
    }

    #[test]
    fn test_validate_raw_sql_flagged() {
        let migration = Migration {
            version: 3,
            description: "Raw SQL".to_string(),
            author: "charlie".to_string(),
            operations: vec![MigrationOp::RawSql {
                sql: "SELECT * FROM users".to_string(),
            }],
        };
        let result = validate_migration(&migration);
        assert!(
            !result.warnings.is_empty() || !result.errors.is_empty(),
            "RawSql should trigger warnings"
        );
    }

    #[test]
    fn test_permissions_readonly_denied() {
        let result = check_permissions(&safe_migration(), PermissionLevel::ReadOnly);
        assert!(result.is_err(), "ReadOnly user cannot run migrations");
    }

    #[test]
    fn test_permissions_standard_allows_safe() {
        let result = check_permissions(&safe_migration(), PermissionLevel::Standard);
        assert!(result.is_ok(), "Standard user should run safe migrations");
    }

    #[test]
    fn test_permissions_standard_denies_drop() {
        let result = check_permissions(&dangerous_migration(), PermissionLevel::Standard);
        assert!(result.is_err(), "Standard user cannot DROP TABLE");
    }

    #[test]
    fn test_permissions_admin_allows_all() {
        let result = check_permissions(&dangerous_migration(), PermissionLevel::Admin);
        assert!(result.is_ok(), "Admin can do anything");
    }

    #[test]
    fn test_dry_run_produces_sql() {
        let sql = dry_run_migration(&safe_migration());
        assert!(!sql.is_empty(), "Dry run should produce SQL");
        assert!(sql[0].to_uppercase().contains("CREATE"));
    }

    #[test]
    fn test_migration_log_is_valid_json() {
        let log = log_migration(&safe_migration());
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&log);
        assert!(parsed.is_ok(), "Migration log must be valid JSON");
    }

    #[test]
    fn test_demonstrate_migration_security() {
        let (valid, standard_denied, admin_allowed) = demonstrate_migration_security();
        assert!(valid, "Safe migration validation should pass");
        assert!(standard_denied, "Standard user should be denied dangerous ops");
        assert!(admin_allowed, "Admin should be allowed everything");
    }
}
