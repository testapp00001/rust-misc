//! # Lesson 10: Row-Level Security
//!
//! ## What Is Row-Level Security (RLS)?
//!
//! Row-level security restricts which rows a user can see or modify based on
//! their identity. Instead of filtering in application code (error-prone), the
//! database enforces access at the row level.
//!
//! ## Why RLS?
//!
//! ```text
//! WITHOUT RLS:
//!   SELECT * FROM documents WHERE tenant_id = ?
//!   ↑ If a developer forgets this WHERE clause, ALL tenants' data leaks
//!
//! WITH RLS:
//!   The database AUTOMATICALLY adds: WHERE tenant_id = current_user_tenant()
//!   ↑ Even if the application forgets, the database enforces it
//! ```
//!
//! ## Common Patterns
//!
//! ```text
//! MULTI-TENANCY:   Each tenant sees only their rows
//!   Policy: tenant_id = current_setting('app.tenant_id')
//!
//! OWNERSHIP:       Users see only their own records
//!   Policy: user_id = current_setting('app.user_id')
//!
//! ROLE-BASED:      Managers see their department's rows
//!   Policy: department_id IN (SELECT dept FROM user_departments WHERE user_id = ...)
//!
//! HIERARCHICAL:    Users see their row and all children
//!   Policy: path <@ (SELECT path FROM users WHERE id = current_user_id())
//! ```
//!
//! ## This Module
//!
//! We simulate row-level security in Rust. Since we don't have a real database,
//! we implement the policy engine that filters rows based on user context.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A row in our simulated database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub data: HashMap<String, String>,
}

/// Security context for the current user/session.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// The current user's ID
    pub user_id: String,
    /// The current user's tenant
    pub tenant_id: String,
    /// The current user's roles
    pub roles: Vec<String>,
    /// The current user's department
    pub department: String,
}

/// A row-level security policy.
#[derive(Debug, Clone)]
pub enum RlsPolicy {
    /// User can only see rows where `column == context.user_id`
    OwnerOnly { column: String },
    /// User can only see rows where `column == context.tenant_id`
    TenantIsolation { column: String },
    /// User can see rows where `column` is in their `context.roles`
    RoleBased { column: String },
    /// User can see rows where `column == context.department`
    DepartmentFilter { column: String },
    /// Combine multiple policies with AND (all must pass)
    And(Vec<RlsPolicy>),
    /// Combine multiple policies with OR (any must pass)
    Or(Vec<RlsPolicy>),
}

/// Evaluate an RLS policy against a row and security context.
///
/// Returns true if the row is accessible, false if it should be filtered out.
///
/// Exercise: Implement policy evaluation.
///
/// Hints:
/// - OwnerOnly: check `row.data[column] == context.user_id`
/// - TenantIsolation: check `row.data[column] == context.tenant_id`
/// - RoleBased: check if `row.data[column]` is in `context.roles`
/// - DepartmentFilter: check `row.data[column] == context.department`
/// - And: all sub-policies must return true
/// - Or: at least one sub-policy must return true
pub fn evaluate_policy(
    policy: &RlsPolicy,
    row: &Row,
    context: &SecurityContext,
) -> bool {
    todo!("Evaluate RLS policy against a row")
}

/// Filter a table (list of rows) using an RLS policy.
///
/// Returns only the rows that pass the policy check.
///
/// Exercise: Filter rows using `evaluate_policy`.
///
/// Hints:
/// - Use `rows.iter().filter(|row| evaluate_policy(policy, row, context))`
/// - Collect into a Vec
pub fn filter_rows(
    policy: &RlsPolicy,
    rows: &[Row],
    context: &SecurityContext,
) -> Vec<Row> {
    todo!("Filter rows based on RLS policy")
}

/// Check if an UPDATE operation is allowed by the RLS policy.
///
/// The user must be able to READ the row (policy passes) AND must have the
/// appropriate role for write access.
///
/// Exercise: Check both read access and write permission.
///
/// Hints:
/// - First check if the user can read the row (evaluate_policy)
/// - Then check if the user has an "admin" or "editor" role
/// - Return Ok(()) if allowed, Err(message) if denied
pub fn check_update_allowed(
    policy: &RlsPolicy,
    row: &Row,
    context: &SecurityContext,
) -> Result<(), String> {
    todo!("Check if UPDATE is allowed for this row and context")
}

/// Check if a DELETE operation is allowed.
///
/// Only owners (user_id matches) and admins can delete.
///
/// Exercise: Implement delete permission check.
///
/// Hints:
/// - Check if user is admin (role contains "admin")
/// - Or check if user owns the row (user_id column matches)
/// - Return Ok(()) if allowed, Err(message) if denied
pub fn check_delete_allowed(
    owner_column: &str,
    row: &Row,
    context: &SecurityContext,
) -> Result<(), String> {
    todo!("Check if DELETE is allowed")
}

/// Demonstrate multi-tenant row-level security.
///
/// Exercise: Create rows for multiple tenants and show that each tenant
/// can only see their own rows.
///
/// Hints:
/// - Create 6 rows: 3 for tenant "acme", 3 for tenant "globex"
/// - Create a TenantIsolation policy on the "tenant_id" column
/// - Filter with context for each tenant
/// - Return (acme_count, globex_count, cross_tenant_blocked)
pub fn demonstrate_multi_tenant_rls() -> (usize, usize, bool) {
    todo!("Demonstrate multi-tenant RLS")
}

/// Demonstrate combined policies (AND/OR).
///
/// Exercise: Create a policy that requires BOTH tenant isolation AND
/// department filtering. Show that rows must pass both checks.
///
/// Hints:
/// - Create an And policy with TenantIsolation and DepartmentFilter
/// - Create rows that pass one check but not the other
/// - Show that only rows passing both are returned
pub fn demonstrate_combined_policies() -> (usize, usize) {
    todo!("Return (total_rows, rows_passing_both_policies)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(id: u64, fields: Vec<(&str, &str)>) -> Row {
        let mut data = HashMap::new();
        for (k, v) in fields {
            data.insert(k.to_string(), v.to_string());
        }
        Row { id, data }
    }

    fn alice_context() -> SecurityContext {
        SecurityContext {
            user_id: "alice".to_string(),
            tenant_id: "acme".to_string(),
            roles: vec!["user".to_string(), "editor".to_string()],
            department: "engineering".to_string(),
        }
    }

    fn bob_context() -> SecurityContext {
        SecurityContext {
            user_id: "bob".to_string(),
            tenant_id: "globex".to_string(),
            roles: vec!["user".to_string()],
            department: "sales".to_string(),
        }
    }

    #[test]
    fn test_owner_only_policy() {
        let policy = RlsPolicy::OwnerOnly { column: "owner".to_string() };
        let row = make_row(1, vec![("owner", "alice")]);
        assert!(evaluate_policy(&policy, &row, &alice_context()));
        assert!(!evaluate_policy(&policy, &row, &bob_context()));
    }

    #[test]
    fn test_tenant_isolation() {
        let policy = RlsPolicy::TenantIsolation { column: "tenant_id".to_string() };
        let acme_row = make_row(1, vec![("tenant_id", "acme")]);
        let globex_row = make_row(2, vec![("tenant_id", "globex")]);

        assert!(evaluate_policy(&policy, &acme_row, &alice_context()));
        assert!(!evaluate_policy(&policy, &globex_row, &alice_context()));
    }

    #[test]
    fn test_role_based_policy() {
        let policy = RlsPolicy::RoleBased { column: "required_role".to_string() };
        let row = make_row(1, vec![("required_role", "editor")]);

        assert!(evaluate_policy(&policy, &row, &alice_context()), "Alice has editor role");
        assert!(!evaluate_policy(&policy, &row, &bob_context()), "Bob does not have editor role");
    }

    #[test]
    fn test_department_filter() {
        let policy = RlsPolicy::DepartmentFilter { column: "dept".to_string() };
        let eng_row = make_row(1, vec![("dept", "engineering")]);
        let sales_row = make_row(2, vec![("dept", "sales")]);

        assert!(evaluate_policy(&policy, &eng_row, &alice_context()));
        assert!(!evaluate_policy(&policy, &sales_row, &alice_context()));
    }

    #[test]
    fn test_combined_and_policy() {
        let policy = RlsPolicy::And(vec![
            RlsPolicy::TenantIsolation { column: "tenant_id".to_string() },
            RlsPolicy::DepartmentFilter { column: "dept".to_string() },
        ]);

        let both_match = make_row(1, vec![("tenant_id", "acme"), ("dept", "engineering")]);
        let tenant_only = make_row(2, vec![("tenant_id", "acme"), ("dept", "sales")]);

        assert!(evaluate_policy(&policy, &both_match, &alice_context()));
        assert!(!evaluate_policy(&policy, &tenant_only, &alice_context()));
    }

    #[test]
    fn test_filter_rows() {
        let policy = RlsPolicy::TenantIsolation { column: "tenant_id".to_string() };
        let rows = vec![
            make_row(1, vec![("tenant_id", "acme")]),
            make_row(2, vec![("tenant_id", "globex")]),
            make_row(3, vec![("tenant_id", "acme")]),
        ];
        let filtered = filter_rows(&policy, &rows, &alice_context());
        assert_eq!(filtered.len(), 2, "Alice should see only acme rows");
    }

    #[test]
    fn test_update_allowed() {
        let policy = RlsPolicy::OwnerOnly { column: "owner".to_string() };
        let row = make_row(1, vec![("owner", "alice")]);

        assert!(check_update_allowed(&policy, &row, &alice_context()).is_ok());
        assert!(check_update_allowed(&policy, &row, &bob_context()).is_err());
    }

    #[test]
    fn test_delete_only_owner_or_admin() {
        let row = make_row(1, vec![("owner", "alice")]);
        assert!(check_delete_allowed("owner", &row, &alice_context()).is_ok());

        let admin_ctx = SecurityContext {
            user_id: "admin_user".to_string(),
            tenant_id: "acme".to_string(),
            roles: vec!["admin".to_string()],
            department: "ops".to_string(),
        };
        assert!(check_delete_allowed("owner", &row, &admin_ctx).is_ok());
        assert!(check_delete_allowed("owner", &row, &bob_context()).is_err());
    }

    #[test]
    fn test_demonstrate_multi_tenant() {
        let (acme, globex, blocked) = demonstrate_multi_tenant_rls();
        assert_eq!(acme, 3, "Acme tenant should see 3 rows");
        assert_eq!(globex, 3, "Globex tenant should see 3 rows");
        assert!(blocked, "Cross-tenant access should be blocked");
    }

    #[test]
    fn test_demonstrate_combined() {
        let (total, passing) = demonstrate_combined_policies();
        assert!(total > passing, "Combined policy should filter some rows");
        assert!(passing > 0, "At least some rows should pass both policies");
    }
}
