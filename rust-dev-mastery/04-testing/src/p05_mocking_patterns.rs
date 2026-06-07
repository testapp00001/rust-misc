//! # Lesson 5: Mocking Patterns
//!
//! Mocking isolates code under test from its dependencies.
//! This lesson covers mockall, automock, expectations, and predicate-based
//! mocking for creating reliable tests.

use mockall::automock;
use mockall::predicate::*;

// ---------------------------------------------------------------------------
// Trait to mock
// ---------------------------------------------------------------------------

/// A trait representing a database connection.
/// mockall generates a MockDatabase from this.
#[automock]
pub trait Database {
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn set(&mut self, key: &str, value: &str) -> Result<(), String>;
    fn delete(&mut self, key: &str) -> Result<bool, String>;
    fn list_keys(&self) -> Result<Vec<String>, String>;
}

/// A trait for sending notifications.
#[automock]
pub trait NotificationService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
    fn send_sms(&self, phone: &str, message: &str) -> Result<(), String>;
}

/// A trait for getting the current time.
#[automock]
pub trait Clock {
    fn now_unix(&self) -> u64;
    fn now_iso8601(&self) -> String;
}

// ---------------------------------------------------------------------------
// Service that depends on traits (code under test)
// ---------------------------------------------------------------------------

/// A user service that depends on a database and notification service.
pub struct UserService<D: Database, N: NotificationService> {
    db: D,
    notifications: N,
}

impl<D: Database, N: NotificationService> UserService<D, N> {
    pub fn new(db: D, notifications: N) -> Self {
        Self { db, notifications }
    }

    /// Get a user by ID.
    pub fn get_user(&self, user_id: &str) -> Result<Option<User>, String> {
        let data = self.db.get(&format!("user:{}", user_id))?;
        match data {
            Some(json) => {
                let user: User =
                    serde_json::from_str(&json).map_err(|e| format!("parse error: {}", e))?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Create a new user and send a welcome email.
    pub fn create_user(&mut self, user: &User) -> Result<(), String> {
        let json =
            serde_json::to_string(user).map_err(|e| format!("serialize error: {}", e))?;
        self.db.set(&format!("user:{}", user.id), &json)?;

        self.notifications
            .send_email(&user.email, "Welcome!", &format!("Welcome, {}!", user.name))?;

        Ok(())
    }

    /// Delete a user.
    pub fn delete_user(&mut self, user_id: &str) -> Result<bool, String> {
        self.db.delete(&format!("user:{}", user_id))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_found() {
        let mut mock_db = MockDatabase::new();
        let mock_notif = MockNotificationService::new();

        let user = User {
            id: "1".into(),
            name: "Alice".into(),
            email: "alice@example.com".into(),
        };
        let json = serde_json::to_string(&user).unwrap();

        mock_db
            .expect_get()
            .with(eq("user:1"))
            .times(1)
            .returning(move |_| Ok(Some(json.clone())));

        let service = UserService::new(mock_db, mock_notif);
        let result = service.get_user("1").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Alice");
    }

    #[test]
    fn test_get_user_not_found() {
        let mut mock_db = MockDatabase::new();
        let mock_notif = MockNotificationService::new();

        mock_db
            .expect_get()
            .with(eq("user:999"))
            .times(1)
            .returning(|_| Ok(None));

        let service = UserService::new(mock_db, mock_notif);
        let result = service.get_user("999").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_create_user() {
        let mut mock_db = MockDatabase::new();
        let mut mock_notif = MockNotificationService::new();

        // Expect a set call
        mock_db
            .expect_set()
            .with(eq("user:1"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        // Expect a welcome email
        mock_notif
            .expect_send_email()
            .with(
                eq("alice@example.com"),
                eq("Welcome!"),
                mockall::predicate::str::contains("Alice"),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let mut service = UserService::new(mock_db, mock_notif);
        let user = User {
            id: "1".into(),
            name: "Alice".into(),
            email: "alice@example.com".into(),
        };
        service.create_user(&user).unwrap();
    }

    #[test]
    fn test_create_user_db_error() {
        let mut mock_db = MockDatabase::new();
        let mock_notif = MockNotificationService::new();

        mock_db
            .expect_set()
            .returning(|_, _| Err("db error".into()));

        let mut service = UserService::new(mock_db, mock_notif);
        let user = User {
            id: "1".into(),
            name: "Alice".into(),
            email: "alice@example.com".into(),
        };
        let result = service.create_user(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("db error"));
    }

    #[test]
    fn test_create_user_email_error() {
        let mut mock_db = MockDatabase::new();
        let mut mock_notif = MockNotificationService::new();

        mock_db.expect_set().returning(|_, _| Ok(()));

        mock_notif
            .expect_send_email()
            .returning(|_, _, _| Err("email service down".into()));

        let mut service = UserService::new(mock_db, mock_notif);
        let user = User {
            id: "1".into(),
            name: "Alice".into(),
            email: "alice@example.com".into(),
        };
        let result = service.create_user(&user);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_user() {
        let mut mock_db = MockDatabase::new();
        let mock_notif = MockNotificationService::new();

        mock_db
            .expect_delete()
            .with(eq("user:1"))
            .times(1)
            .returning(|_| Ok(true));

        let mut service = UserService::new(mock_db, mock_notif);
        assert!(service.delete_user("1").unwrap());
    }

    #[test]
    fn test_clock_mock() {
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now_unix().returning(|| 1700000000);
        mock_clock
            .expect_now_iso8601()
            .returning(|| "2023-11-14T22:13:20Z".into());

        assert_eq!(mock_clock.now_unix(), 1700000000);
        assert_eq!(mock_clock.now_iso8601(), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn test_mock_with_predicates() {
        let mut mock_db = MockDatabase::new();

        // Use predicate: key starts with "user:"
        mock_db
            .expect_get()
            .with(mockall::predicate::str::contains("user:"))
            .returning(|_| Ok(None));

        let service = UserService::new(mock_db, MockNotificationService::new());
        let result = service.get_user("123").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_calls() {
        let mut mock_db = MockDatabase::new();

        mock_db
            .expect_get()
            .times(3)
            .returning(|_| Ok(None));

        let service = UserService::new(mock_db, MockNotificationService::new());
        let _ = service.get_user("1");
        let _ = service.get_user("2");
        let _ = service.get_user("3");
    }

    #[test]
    fn test_mock_sequence() {
        let mut mock_db = MockDatabase::new();
        let mut seq = mockall::Sequence::new();

        let user_json = serde_json::to_string(&User {
            id: "1".into(),
            name: "Alice".into(),
            email: "alice@example.com".into(),
        })
        .unwrap();

        // First call returns value, second returns None
        mock_db
            .expect_get()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(Some(user_json.clone())));
        mock_db
            .expect_get()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(None));

        let service = UserService::new(mock_db, MockNotificationService::new());
        assert!(service.get_user("1").unwrap().is_some());
        assert!(service.get_user("1").unwrap().is_none());
    }
}
