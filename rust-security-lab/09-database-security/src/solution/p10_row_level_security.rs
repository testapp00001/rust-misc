//! # Lesson 10: Row-Level Security (Reference Solution)
//!
//! See the exercise file for full documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub department: String,
}

#[derive(Debug, Clone)]
pub enum RlsPolicy {
    OwnerOnly { column: String },
    TenantIsolation { column: String },
    RoleBased { column: String },
    DepartmentFilter { column: String },
    And(Vec<RlsPolicy>),
    Or(Vec<RlsPolicy>),
}

/// Evaluate an RLS policy against a row and security context.
pub fn evaluate_policy(
    policy: &RlsPolicy,
    row: &Row,
    context: &SecurityContext,
) -> bool {
    match policy {
        RlsPolicy::OwnerOnly { column } => {
            row.data
                .get(column)
                .map_or(false, |v| v == &context.user_id)
        }
        RlsPolicy::TenantIsolation { column } => {
            row.data
                .get(column)
                .map_or(false, |v| v == &context.tenant_id)
        }
        RlsPolicy::RoleBased { column } => {
            row.data
                .get(column)
                .map_or(false, |v| context.roles.contains(v))
        }
        RlsPolicy::DepartmentFilter { column } => {
            row.data
                .get(column)
                .map_or(false, |v| v == &context.department)
        }
        RlsPolicy::And(policies) => {
            policies.iter().all(|p| evaluate_policy(p, row, context))
        }
        RlsPolicy::Or(policies) => {
            policies.iter().any(|p| evaluate_policy(p, row, context))
        }
    }
}

/// Filter rows based on RLS policy.
pub fn filter_rows(
    policy: &RlsPolicy,
    rows: &[Row],
    context: &SecurityContext,
) -> Vec<Row> {
    rows.iter()
        .filter(|row| evaluate_policy(policy, row, context))
        .cloned()
        .collect()
}

/// Check if an UPDATE operation is allowed.
pub fn check_update_allowed(
    policy: &RlsPolicy,
    row: &Row,
    context: &SecurityContext,
) -> Result<(), String> {
    // First check read access
    if !evaluate_policy(policy, row, context) {
        return Err("Access denied: row not visible under current RLS policy".to_string());
    }

    // Check write permission (must be admin or editor)
    if context.roles.contains(&"admin".to_string())
        || context.roles.contains(&"editor".to_string())
    {
        Ok(())
    } else {
        Err("Access denied: requires admin or editor role".to_string())
    }
}

/// Check if a DELETE operation is allowed.
pub fn check_delete_allowed(
    owner_column: &str,
    row: &Row,
    context: &SecurityContext,
) -> Result<(), String> {
    // Admins can always delete
    if context.roles.contains(&"admin".to_string()) {
        return Ok(());
    }

    // Owners can delete their own rows
    if row
        .data
        .get(owner_column)
        .map_or(false, |v| v == &context.user_id)
    {
        return Ok(());
    }

    Err("Access denied: only owner or admin can delete".to_string())
}

/// Demonstrate multi-tenant row-level security.
pub fn demonstrate_multi_tenant_rls() -> (usize, usize, bool) {
    let policy = RlsPolicy::TenantIsolation {
        column: "tenant_id".to_string(),
    };

    let rows = vec![
        Row {
            id: 1,
            data: HashMap::from([("tenant_id".to_string(), "acme".to_string())]),
        },
        Row {
            id: 2,
            data: HashMap::from([("tenant_id".to_string(), "acme".to_string())]),
        },
        Row {
            id: 3,
            data: HashMap::from([("tenant_id".to_string(), "acme".to_string())]),
        },
        Row {
            id: 4,
            data: HashMap::from([("tenant_id".to_string(), "globex".to_string())]),
        },
        Row {
            id: 5,
            data: HashMap::from([("tenant_id".to_string(), "globex".to_string())]),
        },
        Row {
            id: 6,
            data: HashMap::from([("tenant_id".to_string(), "globex".to_string())]),
        },
    ];

    let acme_ctx = SecurityContext {
        user_id: "alice".to_string(),
        tenant_id: "acme".to_string(),
        roles: vec!["user".to_string()],
        department: "engineering".to_string(),
    };

    let globex_ctx = SecurityContext {
        user_id: "bob".to_string(),
        tenant_id: "globex".to_string(),
        roles: vec!["user".to_string()],
        department: "sales".to_string(),
    };

    let acme_rows = filter_rows(&policy, &rows, &acme_ctx);
    let globex_rows = filter_rows(&policy, &rows, &globex_ctx);

    // Verify cross-tenant access is blocked
    let acme_sees_globex = acme_rows.iter().any(|r| {
        r.data
            .get("tenant_id")
            .map_or(false, |v| v == "globex")
    });

    (acme_rows.len(), globex_rows.len(), !acme_sees_globex)
}

/// Demonstrate combined policies (AND).
pub fn demonstrate_combined_policies() -> (usize, usize) {
    let policy = RlsPolicy::And(vec![
        RlsPolicy::TenantIsolation {
            column: "tenant_id".to_string(),
        },
        RlsPolicy::DepartmentFilter {
            column: "dept".to_string(),
        },
    ]);

    let rows = vec![
        Row {
            id: 1,
            data: HashMap::from([
                ("tenant_id".to_string(), "acme".to_string()),
                ("dept".to_string(), "engineering".to_string()),
            ]),
        },
        Row {
            id: 2,
            data: HashMap::from([
                ("tenant_id".to_string(), "acme".to_string()),
                ("dept".to_string(), "sales".to_string()),
            ]),
        },
        Row {
            id: 3,
            data: HashMap::from([
                ("tenant_id".to_string(), "globex".to_string()),
                ("dept".to_string(), "engineering".to_string()),
            ]),
        },
        Row {
            id: 4,
            data: HashMap::from([
                ("tenant_id".to_string(), "acme".to_string()),
                ("dept".to_string(), "engineering".to_string()),
            ]),
        },
    ];

    let ctx = SecurityContext {
        user_id: "alice".to_string(),
        tenant_id: "acme".to_string(),
        roles: vec!["user".to_string()],
        department: "engineering".to_string(),
    };

    let total = rows.len();
    let passing = filter_rows(&policy, &rows, &ctx).len();

    (total, passing)
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
        let policy = RlsPolicy::OwnerOnly {
            column: "owner".to_string(),
        };
        let row = make_row(1, vec![("owner", "alice")]);
        assert!(evaluate_policy(&policy, &row, &alice_context()));
        assert!(!evaluate_policy(&policy, &row, &bob_context()));
    }

    #[test]
    fn test_tenant_isolation() {
        let policy = RlsPolicy::TenantIsolation {
            column: "tenant_id".to_string(),
        };
        let acme_row = make_row(1, vec![("tenant_id", "acme")]);
        let globex_row = make_row(2, vec![("tenant_id", "globex")]);

        assert!(evaluate_policy(&policy, &acme_row, &alice_context()));
        assert!(!evaluate_policy(&policy, &globex_row, &alice_context()));
    }

    #[test]
    fn test_role_based_policy() {
        let policy = RlsPolicy::RoleBased {
            column: "required_role".to_string(),
        };
        let row = make_row(1, vec![("required_role", "editor")]);

        assert!(
            evaluate_policy(&policy, &row, &alice_context()),
            "Alice has editor role"
        );
        assert!(
            !evaluate_policy(&policy, &row, &bob_context()),
            "Bob does not have editor role"
        );
    }

    #[test]
    fn test_department_filter() {
        let policy = RlsPolicy::DepartmentFilter {
            column: "dept".to_string(),
        };
        let eng_row = make_row(1, vec![("dept", "engineering")]);
        let sales_row = make_row(2, vec![("dept", "sales")]);

        assert!(evaluate_policy(&policy, &eng_row, &alice_context()));
        assert!(!evaluate_policy(&policy, &sales_row, &alice_context()));
    }

    #[test]
    fn test_combined_and_policy() {
        let policy = RlsPolicy::And(vec![
            RlsPolicy::TenantIsolation {
                column: "tenant_id".to_string(),
            },
            RlsPolicy::DepartmentFilter {
                column: "dept".to_string(),
            },
        ]);

        let both_match = make_row(1, vec![("tenant_id", "acme"), ("dept", "engineering")]);
        let tenant_only = make_row(2, vec![("tenant_id", "acme"), ("dept", "sales")]);

        assert!(evaluate_policy(&policy, &both_match, &alice_context()));
        assert!(!evaluate_policy(&policy, &tenant_only, &alice_context()));
    }

    #[test]
    fn test_filter_rows() {
        let policy = RlsPolicy::TenantIsolation {
            column: "tenant_id".to_string(),
        };
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
        let policy = RlsPolicy::OwnerOnly {
            column: "owner".to_string(),
        };
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
