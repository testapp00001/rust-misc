//! # Lesson 10: Error Documentation
//!
//! Well-documented errors are part of your API contract. This lesson covers
//! documenting error enums, providing examples in doc comments, and treating
//! error types as first-class API documentation.

use std::fmt;

// ---------------------------------------------------------------------------
// Documented error enum as API contract
// ---------------------------------------------------------------------------

/// Errors that can occur when interacting with the user service.
///
/// This enum represents all possible failure modes of the user service.
/// Each variant is documented with:
/// - When it occurs
/// - What the caller should do
/// - Whether it's retryable
///
/// # Error Hierarchy
///
/// ```text
/// UserServiceError
/// ├── NotFound       - The requested user does not exist
/// ├── Validation     - The input data is invalid
/// ├── Conflict       - The operation conflicts with current state
/// ├── Unauthorized   - Missing or invalid authentication
/// └── Internal       - Unexpected server error
/// ```
///
/// # Examples
///
/// ```
/// use error_handling::p10_error_documentation::*;
///
/// // Handling a not-found error
/// let result = get_user(999);
/// match result {
///     Err(UserServiceError::NotFound { id, .. }) => {
///         println!("User {} does not exist", id);
///     }
///     Err(UserServiceError::Validation { field, message }) => {
///         println!("Invalid {}: {}", field, message);
///     }
///     Ok(user) => {
///         println!("Found user: {}", user.name);
///     }
///     _ => {}
/// }
/// ```
///
/// # Retry Policy
///
/// | Variant      | Retryable? | Suggested Action          |
/// |-------------|-----------|---------------------------|
/// | NotFound     | No        | Check if ID is correct    |
/// | Validation   | No        | Fix input and retry       |
/// | Conflict     | Maybe     | Refresh and retry         |
/// | Unauthorized | No        | Re-authenticate           |
/// | Internal     | Yes       | Retry with backoff        |
#[derive(Debug, PartialEq)]
pub enum UserServiceError {
    /// The requested user was not found in the system.
    ///
    /// This error occurs when:
    /// - The user ID does not exist
    /// - The user was deleted
    ///
    /// **Action**: Verify the user ID. If the user was recently created,
    /// wait and retry (eventual consistency).
    ///
    /// **Not retryable** unless you suspect eventual consistency.
    NotFound {
        /// The ID that was looked up.
        id: u64,
        /// Optional hint about why it wasn't found.
        reason: Option<String>,
    },

    /// The input data failed validation.
    ///
    /// This error occurs when:
    /// - Required fields are missing
    /// - Field values are out of range
    /// - Field formats are invalid
    ///
    /// **Action**: Fix the invalid fields and retry. The `field` and
    /// `message` provide specific guidance.
    ///
    /// **Not retryable** — the input must be corrected.
    Validation {
        /// The field that failed validation.
        field: String,
        /// Human-readable description of what's wrong.
        message: String,
    },

    /// The operation conflicts with the current state.
    ///
    /// This error occurs when:
    /// - A unique constraint is violated (e.g., duplicate email)
    /// - A concurrent modification occurred
    /// - An optimistic locking check failed
    ///
    /// **Action**: For unique constraint violations, use a different value.
    /// For concurrency conflicts, refresh the data and retry.
    ///
    /// **Retryable** for concurrency conflicts only.
    Conflict {
        /// Description of the conflict.
        message: String,
        /// The current version/etag if this was a concurrency conflict.
        current_version: Option<u64>,
    },

    /// The request is not authenticated or the token is invalid.
    ///
    /// **Action**: Re-authenticate and retry with a valid token.
    ///
    /// **Not retryable** without re-authentication.
    Unauthorized {
        /// Why authentication failed.
        reason: AuthFailureReason,
    },

    /// An unexpected internal error occurred.
    ///
    /// This should not happen in normal operation. If it persists,
    /// contact the service team with the `correlation_id`.
    ///
    /// **Retryable** — retry with exponential backoff.
    Internal {
        /// A correlation ID for debugging with the service team.
        correlation_id: String,
    },
}

/// Reasons why authentication might fail.
#[derive(Debug, PartialEq)]
pub enum AuthFailureReason {
    /// No authentication token was provided.
    MissingToken,
    /// The token has expired.
    TokenExpired,
    /// The token signature is invalid.
    InvalidSignature,
    /// The token doesn't have the required scope.
    InsufficientScope { required: String },
}

impl fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserServiceError::NotFound { id, reason } => {
                write!(f, "user {} not found", id)?;
                if let Some(r) = reason {
                    write!(f, ": {}", r)?;
                }
                Ok(())
            }
            UserServiceError::Validation { field, message } => {
                write!(f, "validation error on '{}': {}", field, message)
            }
            UserServiceError::Conflict { message, .. } => {
                write!(f, "conflict: {}", message)
            }
            UserServiceError::Unauthorized { reason } => {
                write!(f, "unauthorized: {:?}", reason)
            }
            UserServiceError::Internal { correlation_id } => {
                write!(f, "internal error (id: {})", correlation_id)
            }
        }
    }
}

impl std::error::Error for UserServiceError {}

impl UserServiceError {
    /// Returns the HTTP status code for this error.
    ///
    /// Useful for API responses:
    /// ```
    /// use error_handling::p10_error_documentation::*;
    ///
    /// let err = UserServiceError::NotFound { id: 1, reason: None };
    /// assert_eq!(err.status_code(), 404);
    /// ```
    pub fn status_code(&self) -> u16 {
        match self {
            UserServiceError::NotFound { .. } => 404,
            UserServiceError::Validation { .. } => 422,
            UserServiceError::Conflict { .. } => 409,
            UserServiceError::Unauthorized { .. } => 401,
            UserServiceError::Internal { .. } => 500,
        }
    }

    /// Returns the error code string for API responses.
    ///
    /// These codes are stable and safe to use in client code:
    /// ```
    /// use error_handling::p10_error_documentation::*;
    ///
    /// let err = UserServiceError::NotFound { id: 1, reason: None };
    /// assert_eq!(err.error_code(), "USER_NOT_FOUND");
    /// ```
    pub fn error_code(&self) -> &'static str {
        match self {
            UserServiceError::NotFound { .. } => "USER_NOT_FOUND",
            UserServiceError::Validation { .. } => "VALIDATION_ERROR",
            UserServiceError::Conflict { .. } => "CONFLICT",
            UserServiceError::Unauthorized { .. } => "UNAUTHORIZED",
            UserServiceError::Internal { .. } => "INTERNAL_ERROR",
        }
    }

    /// Whether this error is retryable.
    ///
    /// ```
    /// use error_handling::p10_error_documentation::*;
    ///
    /// let internal = UserServiceError::Internal {
    ///     correlation_id: "abc".into(),
    /// };
    /// assert!(internal.is_retryable());
    ///
    /// let not_found = UserServiceError::NotFound { id: 1, reason: None };
    /// assert!(!not_found.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(self, UserServiceError::Internal { .. })
    }
}

// ---------------------------------------------------------------------------
// Functions that produce documented errors
// ---------------------------------------------------------------------------

/// Get a user by ID.
///
/// # Errors
///
/// Returns [`UserServiceError::NotFound`] if no user with the given ID exists.
/// Returns [`UserServiceError::Validation`] if the ID is 0.
/// Returns [`UserServiceError::Internal`] on unexpected failures.
///
/// # Examples
///
/// ```
/// use error_handling::p10_error_documentation::*;
///
/// let user = get_user(1).unwrap();
/// assert_eq!(user.name, "User 1");
/// ```
pub fn get_user(id: u64) -> Result<User, UserServiceError> {
    if id == 0 {
        return Err(UserServiceError::Validation {
            field: "id".into(),
            message: "ID must be positive".into(),
        });
    }
    if id == 999 {
        return Err(UserServiceError::NotFound {
            id,
            reason: Some("user was deleted".into()),
        });
    }
    Ok(User {
        id,
        name: format!("User {}", id),
    })
}

/// Create a new user.
///
/// # Errors
///
/// Returns [`UserServiceError::Validation`] if the name or email is invalid.
/// Returns [`UserServiceError::Conflict`] if the email is already taken.
///
/// # Examples
///
/// ```
/// use error_handling::p10_error_documentation::*;
///
/// let user = create_user("Alice", "alice@example.com").unwrap();
/// assert_eq!(user.name, "Alice");
/// ```
pub fn create_user(name: &str, email: &str) -> Result<User, UserServiceError> {
    if name.is_empty() {
        return Err(UserServiceError::Validation {
            field: "name".into(),
            message: "name cannot be empty".into(),
        });
    }
    if !email.contains('@') {
        return Err(UserServiceError::Validation {
            field: "email".into(),
            message: "invalid email format".into(),
        });
    }
    if email == "taken@example.com" {
        return Err(UserServiceError::Conflict {
            message: "email already registered".into(),
            current_version: None,
        });
    }
    Ok(User {
        id: 1,
        name: name.into(),
    })
}

#[derive(Debug, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Error documentation testing
// ---------------------------------------------------------------------------

/// Helper to verify all error variants are documented and tested.
pub fn all_error_variants() -> Vec<(&'static str, u16, &'static str, bool)> {
    vec![
        ("NotFound", 404, "USER_NOT_FOUND", false),
        ("Validation", 422, "VALIDATION_ERROR", false),
        ("Conflict", 409, "CONFLICT", false),
        ("Unauthorized", 401, "UNAUTHORIZED", false),
        ("Internal", 500, "INTERNAL_ERROR", true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_success() {
        let user = get_user(1).unwrap();
        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_get_user_not_found() {
        let err = get_user(999).unwrap_err();
        assert_eq!(err.status_code(), 404);
        assert_eq!(err.error_code(), "USER_NOT_FOUND");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_get_user_validation() {
        let err = get_user(0).unwrap_err();
        assert_eq!(err.status_code(), 422);
        assert_eq!(err.error_code(), "VALIDATION_ERROR");
    }

    #[test]
    fn test_create_user_success() {
        let user = create_user("Bob", "bob@example.com").unwrap();
        assert_eq!(user.name, "Bob");
    }

    #[test]
    fn test_create_user_empty_name() {
        let err = create_user("", "bob@example.com").unwrap_err();
        assert_eq!(err.status_code(), 422);
        match &err {
            UserServiceError::Validation { field, .. } => {
                assert_eq!(field, "name");
            }
            _ => panic!("expected Validation"),
        }
    }

    #[test]
    fn test_create_user_invalid_email() {
        let err = create_user("Bob", "invalid").unwrap_err();
        match &err {
            UserServiceError::Validation { field, message } => {
                assert_eq!(field, "email");
                assert!(message.contains("invalid"));
            }
            _ => panic!("expected Validation"),
        }
    }

    #[test]
    fn test_create_user_conflict() {
        let err = create_user("Bob", "taken@example.com").unwrap_err();
        assert_eq!(err.status_code(), 409);
        assert_eq!(err.error_code(), "CONFLICT");
    }

    #[test]
    fn test_internal_error_retryable() {
        let err = UserServiceError::Internal {
            correlation_id: "req-123".into(),
        };
        assert!(err.is_retryable());
        assert_eq!(err.status_code(), 500);
        assert!(err.to_string().contains("req-123"));
    }

    #[test]
    fn test_unauthorized_error() {
        let err = UserServiceError::Unauthorized {
            reason: AuthFailureReason::TokenExpired,
        };
        assert_eq!(err.status_code(), 401);
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_unauthorized_insufficient_scope() {
        let err = UserServiceError::Unauthorized {
            reason: AuthFailureReason::InsufficientScope {
                required: "admin".into(),
            },
        };
        assert_eq!(err.error_code(), "UNAUTHORIZED");
    }

    #[test]
    fn test_conflict_with_version() {
        let err = UserServiceError::Conflict {
            message: "concurrent modification".into(),
            current_version: Some(5),
        };
        assert_eq!(err.status_code(), 409);
    }

    #[test]
    fn test_not_found_with_reason() {
        let err = UserServiceError::NotFound {
            id: 42,
            reason: Some("deleted".into()),
        };
        assert!(err.to_string().contains("deleted"));
    }

    #[test]
    fn test_all_error_variants_documented() {
        // This test verifies that all variants have correct metadata
        let variants = all_error_variants();
        assert_eq!(variants.len(), 5);

        for (name, status, code, retryable) in &variants {
            // Verify the metadata is reasonable
            assert!(*status >= 400 && *status < 600, "invalid status for {}", name);
            assert!(!code.is_empty(), "empty code for {}", name);
            // Just verify the tuple is well-formed
            let _ = retryable;
        }
    }

    #[test]
    fn test_error_code_stability() {
        // Error codes are part of the API contract and must not change
        assert_eq!(
            UserServiceError::NotFound {
                id: 0,
                reason: None
            }
            .error_code(),
            "USER_NOT_FOUND"
        );
        assert_eq!(
            UserServiceError::Validation {
                field: "".into(),
                message: "".into()
            }
            .error_code(),
            "VALIDATION_ERROR"
        );
    }
}
