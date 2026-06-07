//! # Dependency Injection
//!
//! Dependency injection (DI) decouples components by having dependencies provided
//! from outside rather than created internally. In Rust, DI is typically achieved
//! through generics (compile-time) or trait objects (runtime).
//!
//! ## Key Concepts
//! - **Constructor injection**: Dependencies passed via `new()`
//! - **Trait-based DI**: Dependencies are trait objects, enabling testing with mocks
//! - **Service locator**: A registry that provides dependencies on request
//! - **Testing with DI**: Replacing real dependencies with test doubles

use std::collections::HashMap;
use std::sync::Arc;

/// A user repository trait — the dependency interface.
pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: u64) -> Option<User>;
    fn save(&mut self, user: &User) -> Result<(), String>;
    fn find_all(&self) -> Vec<User>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

/// A real repository backed by a HashMap (simulating a database).
pub struct InMemoryUserRepository {
    users: HashMap<u64, User>,
    next_id: u64,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        InMemoryUserRepository {
            users: HashMap::new(),
            next_id: 1,
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn find_by_id(&self, id: u64) -> Option<User> {
        self.users.get(&id).cloned()
    }

    fn save(&mut self, user: &User) -> Result<(), String> {
        let mut user = user.clone();
        if user.id == 0 {
            user.id = self.next_id;
            self.next_id += 1;
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    fn find_all(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }
}

/// A service that depends on a UserRepository.
/// The dependency is injected via constructor.
pub struct UserService {
    repository: Box<dyn UserRepository>,
}

impl UserService {
    pub fn new(repository: Box<dyn UserRepository>) -> Self {
        UserService { repository }
    }

    pub fn get_user(&self, id: u64) -> Option<User> {
        self.repository.find_by_id(id)
    }

    pub fn create_user(&mut self, name: &str, email: &str) -> Result<User, String> {
        let user = User {
            id: 0, // Will be assigned by repository
            name: name.to_string(),
            email: email.to_string(),
            active: true,
        };
        self.repository.save(&user)?;
        // Fetch back the saved user to get the assigned id
        let saved = self.repository.find_all().into_iter().find(|u| u.name == name && u.email == email).unwrap();
        Ok(saved)
    }

    pub fn active_users(&self) -> Vec<User> {
        self.repository
            .find_all()
            .into_iter()
            .filter(|u| u.active)
            .collect()
    }
}

/// A notification service trait for testing.
pub trait NotificationService: Send + Sync {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
    fn send_sms(&self, phone: &str, message: &str) -> Result<(), String>;
}

/// A real email service (simulated).
pub struct EmailService {
    pub sent_emails: Vec<(String, String, String)>,
}

impl EmailService {
    pub fn new() -> Self {
        EmailService {
            sent_emails: Vec::new(),
        }
    }
}

impl NotificationService for EmailService {
    fn send_email(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), String> {
        // In real code, this would connect to an SMTP server
        Ok(())
    }

    fn send_sms(&self, _phone: &str, _message: &str) -> Result<(), String> {
        Ok(())
    }
}

/// A mock notification service for testing.
pub struct MockNotificationService {
    pub emails: std::sync::Mutex<Vec<(String, String, String)>>,
    pub sms: std::sync::Mutex<Vec<(String, String)>>,
    pub should_fail: bool,
}

impl MockNotificationService {
    pub fn new() -> Self {
        MockNotificationService {
            emails: std::sync::Mutex::new(Vec::new()),
            sms: std::sync::Mutex::new(Vec::new()),
            should_fail: false,
        }
    }
}

impl NotificationService for MockNotificationService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        if self.should_fail {
            return Err("Mock failure".into());
        }
        self.emails
            .lock()
            .unwrap()
            .push((to.to_string(), subject.to_string(), body.to_string()));
        Ok(())
    }

    fn send_sms(&self, phone: &str, message: &str) -> Result<(), String> {
        if self.should_fail {
            return Err("Mock failure".into());
        }
        self.sms
            .lock()
            .unwrap()
            .push((phone.to_string(), message.to_string()));
        Ok(())
    }
}

/// A notification service that depends on NotificationService.
pub struct NotificationManager {
    notifier: Box<dyn NotificationService>,
}

impl NotificationManager {
    pub fn new(notifier: Box<dyn NotificationService>) -> Self {
        NotificationManager { notifier }
    }

    pub fn notify_user_registration(&self, user: &User) -> Result<(), String> {
        self.notifier.send_email(
            &user.email,
            "Welcome!",
            &format!("Welcome, {}!", user.name),
        )
    }
}

/// A service container (simple service locator pattern).
pub struct ServiceContainer {
    services: HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        ServiceContainer {
            services: HashMap::new(),
        }
    }

    pub fn register<T: Send + Sync + 'static>(&mut self, service: T) {
        self.services
            .insert(std::any::TypeId::of::<T>(), Box::new(service));
    }

    pub fn resolve<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.services
            .get(&std::any::TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<T>())
    }
}

/// A builder for constructing services with all their dependencies.
pub struct ServiceBuilder {
    repository: Option<Box<dyn UserRepository>>,
    notifier: Option<Box<dyn NotificationService>>,
}

impl ServiceBuilder {
    pub fn new() -> Self {
        ServiceBuilder {
            repository: None,
            notifier: None,
        }
    }

    pub fn with_repository(mut self, repo: impl UserRepository + 'static) -> Self {
        self.repository = Some(Box::new(repo));
        self
    }

    pub fn with_notifier(mut self, notifier: impl NotificationService + 'static) -> Self {
        self.notifier = Some(Box::new(notifier));
        self
    }

    pub fn build(self) -> Result<AppServices, String> {
        Ok(AppServices {
            user_service: UserService::new(
                self.repository
                    .ok_or("Repository not configured")?,
            ),
            notification_manager: NotificationManager::new(
                self.notifier
                    .ok_or("Notification service not configured")?,
            ),
        })
    }
}

pub struct AppServices {
    pub user_service: UserService,
    pub notification_manager: NotificationManager,
}

/// A mock repository that records calls for verification.
pub struct MockUserRepository {
    pub users: HashMap<u64, User>,
    pub find_calls: std::sync::Mutex<Vec<u64>>,
    pub save_calls: std::sync::Mutex<Vec<User>>,
    pub next_id: u64,
}

impl MockUserRepository {
    pub fn new() -> Self {
        MockUserRepository {
            users: HashMap::new(),
            find_calls: std::sync::Mutex::new(Vec::new()),
            save_calls: std::sync::Mutex::new(Vec::new()),
            next_id: 1,
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }
}

impl UserRepository for MockUserRepository {
    fn find_by_id(&self, id: u64) -> Option<User> {
        self.find_calls.lock().unwrap().push(id);
        self.users.get(&id).cloned()
    }

    fn save(&mut self, user: &User) -> Result<(), String> {
        self.save_calls.lock().unwrap().push(user.clone());
        Ok(())
    }

    fn find_all(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_service_with_in_memory_repo() {
        let mut repo = InMemoryUserRepository::new();
        let user = User {
            id: 0,
            name: "Alice".into(),
            email: "alice@example.com".into(),
            active: true,
        };
        repo.save(&user).unwrap();

        let service = UserService::new(Box::new(repo));
        let found = service.get_user(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Alice");
    }

    #[test]
    fn test_user_service_create() {
        let repo = InMemoryUserRepository::new();
        let mut service = UserService::new(Box::new(repo));

        let user = service.create_user("Bob", "bob@example.com").unwrap();
        assert_eq!(user.name, "Bob");

        let found = service.get_user(user.id);
        assert!(found.is_some());
    }

    #[test]
    fn test_user_service_active_users() {
        let mut repo = InMemoryUserRepository::new();
        repo.save(&User { id: 1, name: "A".into(), email: "a@e.com".into(), active: true }).unwrap();
        repo.save(&User { id: 2, name: "B".into(), email: "b@e.com".into(), active: false }).unwrap();
        repo.save(&User { id: 3, name: "C".into(), email: "c@e.com".into(), active: true }).unwrap();

        let service = UserService::new(Box::new(repo));
        let active = service.active_users();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_notification_with_mock() {
        let mock = MockNotificationService::new();
        let manager = NotificationManager::new(Box::new(mock));

        let user = User {
            id: 1,
            name: "Alice".into(),
            email: "alice@example.com".into(),
            active: true,
        };

        manager.notify_user_registration(&user).unwrap();

        // Can't easily verify mock state through the trait object,
        // but the call succeeded
    }

    #[test]
    fn test_service_builder() {
        let repo = InMemoryUserRepository::new();
        let notifier = MockNotificationService::new();

        let services = ServiceBuilder::new()
            .with_repository(repo)
            .with_notifier(notifier)
            .build()
            .unwrap();

        // Services are constructed with all dependencies
        let user = services.user_service.get_user(1);
        assert!(user.is_none());
    }

    #[test]
    fn test_service_builder_missing_dependency() {
        let result = ServiceBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_container() {
        let mut container = ServiceContainer::new();
        container.register(42u32);
        container.register("hello".to_string());

        assert_eq!(container.resolve::<u32>(), Some(&42));
        assert_eq!(container.resolve::<String>(), Some(&"hello".to_string()));
        assert!(container.resolve::<f64>().is_none());
    }

    #[test]
    fn test_mock_repository() {
        let mut mock = MockUserRepository::new();
        mock.add_user(User {
            id: 1,
            name: "Test".into(),
            email: "test@example.com".into(),
            active: true,
        });

        let found = mock.find_by_id(1);
        assert!(found.is_some());
        assert_eq!(*mock.find_calls.lock().unwrap(), vec![1]);

        let not_found = mock.find_by_id(999);
        assert!(not_found.is_none());
        assert_eq!(*mock.find_calls.lock().unwrap(), vec![1, 999]);
    }

    #[test]
    fn test_in_memory_repo_save_and_find() {
        let mut repo = InMemoryUserRepository::new();

        let user = User {
            id: 0, // Auto-assign
            name: "New".into(),
            email: "new@example.com".into(),
            active: true,
        };
        repo.save(&user).unwrap();

        let all = repo.find_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, 1);
    }
}
