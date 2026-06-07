/// Problem: Repository Pattern
///
/// Master the repository pattern in Rust.
///
/// Key Concepts:
/// - Repository trait
/// - CRUD operations
/// - Query building
/// - Data mapping
/// - Abstraction

use std::collections::HashMap;

/// Problem 1: Basic repository
/// Create basic repository
pub trait Repository<T> {
    fn find_by_id(&self, id: u32) -> Option<&T>;
    fn find_all(&self) -> Vec<&T>;
    fn save(&mut self, entity: T) -> T;
    fn delete(&mut self, id: u32) -> bool;
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

pub struct UserRepository {
    users: HashMap<u32, User>,
    next_id: u32,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }
}

impl Repository<User> for UserRepository {
    fn find_by_id(&self, id: u32) -> Option<&User> {
        self.users.get(&id)
    }

    fn find_all(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    fn save(&mut self, mut user: User) -> User {
        if user.id == 0 {
            user.id = self.next_id;
            self.next_id += 1;
        }
        let id = user.id;
        self.users.insert(id, user.clone());
        user
    }

    fn delete(&mut self, id: u32) -> bool {
        self.users.remove(&id).is_some()
    }
}

/// Problem 2: Repository with queries
/// Add query capabilities
pub trait QueryableRepository<T> {
    fn find_by(&self, predicate: &dyn Fn(&T) -> bool) -> Vec<&T>;
    fn find_one_by(&self, predicate: &dyn Fn(&T) -> bool) -> Option<&T>;
    fn count(&self) -> usize;
    fn exists(&self, predicate: &dyn Fn(&T) -> bool) -> bool;
}

impl QueryableRepository<User> for UserRepository {
    fn find_by(&self, predicate: &dyn Fn(&User) -> bool) -> Vec<&User> {
        self.users.values().filter(|u| predicate(u)).collect()
    }

    fn find_one_by(&self, predicate: &dyn Fn(&User) -> bool) -> Option<&User> {
        self.users.values().find(|u| predicate(u))
    }

    fn count(&self) -> usize {
        self.users.len()
    }

    fn exists(&self, predicate: &dyn Fn(&User) -> bool) -> bool {
        self.users.values().any(|u| predicate(u))
    }
}

/// Problem 3: Repository with pagination
/// Add pagination support
pub trait PaginatedRepository<T> {
    fn find_page(&self, page: usize, per_page: usize) -> Vec<&T>;
    fn total_pages(&self, per_page: usize) -> usize;
}

impl PaginatedRepository<User> for UserRepository {
    fn find_page(&self, page: usize, per_page: usize) -> Vec<&User> {
        let users: Vec<&User> = self.users.values().collect();
        let start = (page - 1) * per_page;
        users.into_iter().skip(start).take(per_page).collect()
    }

    fn total_pages(&self, per_page: usize) -> usize {
        (self.users.len() + per_page - 1) / per_page
    }
}

/// Problem 4: Repository with sorting
/// Add sorting support
pub trait SortedRepository<T> {
    fn find_all_sorted(&self, compare: &dyn Fn(&T, &T) -> std::cmp::Ordering) -> Vec<&T>;
}

impl SortedRepository<User> for UserRepository {
    fn find_all_sorted(&self, compare: &dyn Fn(&User, &User) -> std::cmp::Ordering) -> Vec<&User> {
        let mut users: Vec<&User> = self.users.values().collect();
        users.sort_by(|a, b| compare(a, b));
        users
    }
}

/// Problem 5: Repository with transactions
/// Simulate transactions
pub trait TransactionalRepository<T> {
    fn begin_transaction(&mut self);
    fn commit(&mut self);
    fn rollback(&mut self);
}

pub struct TransactionalUserRepository {
    users: HashMap<u32, User>,
    next_id: u32,
    transaction_users: Option<HashMap<u32, User>>,
}

impl TransactionalUserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
            transaction_users: None,
        }
    }
}

impl TransactionalRepository<User> for TransactionalUserRepository {
    fn begin_transaction(&mut self) {
        self.transaction_users = Some(self.users.clone());
    }

    fn commit(&mut self) {
        self.transaction_users = None;
    }

    fn rollback(&mut self) {
        if let Some(users) = self.transaction_users.take() {
            self.users = users;
        }
    }
}

/// Problem 6: Repository with caching
/// Add caching layer
pub struct CachedRepository<T: Clone> {
    inner: Box<dyn Repository<T>>,
    cache: HashMap<u32, T>,
}

impl<T: Clone> CachedRepository<T> {
    pub fn new(inner: Box<dyn Repository<T>>) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
        }
    }
}

/// Problem 7: Repository with validation
/// Add validation
pub trait ValidatedRepository<T> {
    fn validate(&self, entity: &T) -> Result<(), String>;
    fn save_validated(&mut self, entity: T) -> Result<T, String>;
}

/// Problem 8: Repository with events
/// Add event support
pub trait EventRepository<T> {
    fn on_save(&self, entity: &T);
    fn on_delete(&self, id: u32);
}

/// Problem 9: Repository with batch operations
/// Add batch support
pub trait BatchRepository<T> {
    fn save_batch(&mut self, entities: Vec<T>) -> Vec<T>;
    fn delete_batch(&mut self, ids: Vec<u32>) -> usize;
}

/// Problem 10: Repository with search
/// Add search capabilities
pub trait SearchableRepository<T> {
    fn search(&self, query: &str) -> Vec<&T>;
}

impl SearchableRepository<User> for UserRepository {
    fn search(&self, query: &str) -> Vec<&User> {
        self.users
            .values()
            .filter(|u| u.name.contains(query) || u.email.contains(query))
            .collect()
    }
}

/// Problem 11: Repository with aggregation
/// Add aggregation support
pub trait AggregateRepository<T> {
    fn count_by(&self, predicate: &dyn Fn(&T) -> bool) -> usize;
}

impl AggregateRepository<User> for UserRepository {
    fn count_by(&self, predicate: &dyn Fn(&User) -> bool) -> usize {
        self.users.values().filter(|u| predicate(u)).count()
    }
}

/// Problem 12: Repository with unique constraints
/// Enforce unique constraints
pub trait UniqueRepository<T> {
    fn find_unique(&self, field: &str, value: &str) -> Option<&T>;
}

impl UniqueRepository<User> for UserRepository {
    fn find_unique(&self, field: &str, value: &str) -> Option<&User> {
        match field {
            "email" => self.users.values().find(|u| u.email == value),
            "name" => self.users.values().find(|u| u.name == value),
            _ => None,
        }
    }
}

/// Problem 13: Repository with soft delete
/// Implement soft delete
#[derive(Debug, Clone)]
pub struct SoftDeleteUser {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub deleted: bool,
}

pub struct SoftDeleteRepository {
    users: HashMap<u32, SoftDeleteUser>,
}

impl SoftDeleteRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn soft_delete(&mut self, id: u32) -> bool {
        if let Some(user) = self.users.get_mut(&id) {
            user.deleted = true;
            true
        } else {
            false
        }
    }

    pub fn find_active(&self) -> Vec<&SoftDeleteUser> {
        self.users.values().filter(|u| !u.deleted).collect()
    }
}

/// Problem 14: Repository with versioning
/// Track entity versions
#[derive(Debug, Clone)]
pub struct VersionedEntity<T: Clone> {
    pub data: T,
    pub version: u32,
}

pub struct VersionedRepository<T: Clone> {
    entities: HashMap<u32, VersionedEntity<T>>,
}

impl<T: Clone> VersionedRepository<T> {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn save(&mut self, id: u32, entity: T) {
        let version = self.entities.get(&id).map(|e| e.version + 1).unwrap_or(1);
        self.entities.insert(id, VersionedEntity { data: entity, version });
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        self.entities.get(&id).map(|e| &e.data)
    }

    pub fn get_version(&self, id: u32) -> Option<u32> {
        self.entities.get(&id).map(|e| e.version)
    }
}

/// Problem 15: Repository with factory
/// Create repository with factory
pub trait RepositoryFactory<T> {
    fn create_repository(&self) -> Box<dyn Repository<T>>;
}

pub struct UserRepositoryFactory;

impl RepositoryFactory<User> for UserRepositoryFactory {
    fn create_repository(&self) -> Box<dyn Repository<User>> {
        Box::new(UserRepository::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_repository() {
        let mut repo = UserRepository::new();
        let user = User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let saved = repo.save(user);
        assert_eq!(saved.id, 1);
        assert!(repo.find_by_id(1).is_some());
    }

    #[test]
    fn test_queryable_repository() {
        let mut repo = UserRepository::new();
        repo.save(User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        repo.save(User {
            id: 0,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        });
        let found = repo.find_by(&|u| u.name.starts_with("A"));
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_paginated_repository() {
        let mut repo = UserRepository::new();
        for i in 0..10 {
            repo.save(User {
                id: 0,
                name: format!("User{}", i),
                email: format!("user{}@example.com", i),
            });
        }
        let page = repo.find_page(1, 3);
        assert_eq!(page.len(), 3);
    }

    #[test]
    fn test_sorted_repository() {
        let mut repo = UserRepository::new();
        repo.save(User {
            id: 0,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        });
        repo.save(User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        let sorted = repo.find_all_sorted(&|a, b| a.name.cmp(&b.name));
        assert_eq!(sorted[0].name, "Alice");
    }

    #[test]
    fn test_transactional_repository() {
        let mut repo = TransactionalUserRepository::new();
        repo.begin_transaction();
        repo.commit();
    }

    #[test]
    fn test_searchable_repository() {
        let mut repo = UserRepository::new();
        repo.save(User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        let found = repo.search("Alice");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_aggregate_repository() {
        let mut repo = UserRepository::new();
        repo.save(User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        let count = repo.count_by(&|u| u.name.starts_with("A"));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_unique_repository() {
        let mut repo = UserRepository::new();
        repo.save(User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        });
        let found = repo.find_unique("email", "alice@example.com");
        assert!(found.is_some());
    }

    #[test]
    fn test_soft_delete_repository() {
        let mut repo = SoftDeleteRepository::new();
        repo.users.insert(1, SoftDeleteUser {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            deleted: false,
        });
        repo.soft_delete(1);
        assert_eq!(repo.find_active().len(), 0);
    }

    #[test]
    fn test_versioned_repository() {
        let mut repo = VersionedRepository::new();
        repo.save(1, "value1".to_string());
        repo.save(1, "value2".to_string());
        assert_eq!(repo.get_version(1), Some(2));
    }

    #[test]
    fn test_repository_factory() {
        let factory = UserRepositoryFactory;
        let mut repo = factory.create_repository();
        let user = User {
            id: 0,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        repo.save(user);
        assert!(repo.find_by_id(1).is_some());
    }
}
