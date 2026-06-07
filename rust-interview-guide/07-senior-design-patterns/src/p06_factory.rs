/// Problem: Factory Pattern
///
/// Master the factory pattern in Rust.
///
/// Key Concepts:
/// - Factory method
/// - Abstract factory
/// - Factory with parameters
/// - Factory with configuration
/// - Factory with validation

/// Problem 1: Basic factory
/// Create basic factory
pub trait Animal {
    fn speak(&self) -> String;
    fn name(&self) -> String;
}

pub struct Dog {
    name: String,
}

pub struct Cat {
    name: String,
}

impl Animal for Dog {
    fn speak(&self) -> String {
        "Woof!".to_string()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Animal for Cat {
    fn speak(&self) -> String {
        "Meow!".to_string()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct AnimalFactory;

impl AnimalFactory {
    pub fn create(animal_type: &str, name: &str) -> Box<dyn Animal> {
        match animal_type {
            "dog" => Box::new(Dog { name: name.to_string() }),
            "cat" => Box::new(Cat { name: name.to_string() }),
            _ => panic!("Unknown animal type"),
        }
    }
}

/// Problem 2: Factory with enum
/// Use enum for factory
#[derive(Debug, Clone)]
pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64),
}

pub struct ShapeFactory;

impl ShapeFactory {
    pub fn create(shape_type: &str, params: &[f64]) -> Shape {
        match shape_type {
            "circle" => Shape::Circle(params[0]),
            "rectangle" => Shape::Rectangle(params[0], params[1]),
            "triangle" => Shape::Triangle(params[0], params[1]),
            _ => panic!("Unknown shape type"),
        }
    }
}

/// Problem 3: Factory with builder
/// Combine factory with builder
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

pub struct ConfigFactory;

impl ConfigFactory {
    pub fn development() -> Config {
        Config {
            host: "localhost".to_string(),
            port: 3000,
            debug: true,
        }
    }

    pub fn production() -> Config {
        Config {
            host: "0.0.0.0".to_string(),
            port: 8080,
            debug: false,
        }
    }

    pub fn custom(host: &str, port: u16, debug: bool) -> Config {
        Config {
            host: host.to_string(),
            port,
            debug,
        }
    }
}

/// Problem 4: Factory with registration
/// Register factory methods
pub type CreatorFn = Box<dyn Fn(&str) -> Box<dyn Animal>>;

pub struct RegistryFactory {
    creators: std::collections::HashMap<String, CreatorFn>,
}

impl RegistryFactory {
    pub fn new() -> Self {
        Self {
            creators: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, creator: CreatorFn) {
        self.creators.insert(name.to_string(), creator);
    }

    pub fn create(&self, name: &str, type_name: &str) -> Option<Box<dyn Animal>> {
        self.creators.get(type_name).map(|creator| creator(name))
    }
}

/// Problem 5: Factory with validation
/// Validate factory inputs
pub struct ValidatingFactory;

impl ValidatingFactory {
    pub fn create_email(email: &str) -> Result<String, String> {
        if email.contains('@') {
            Ok(email.to_string())
        } else {
            Err("Invalid email".to_string())
        }
    }

    pub fn create_age(age: u32) -> Result<u32, String> {
        if age > 0 && age < 150 {
            Ok(age)
        } else {
            Err("Invalid age".to_string())
        }
    }
}

/// Problem 6: Factory with caching
/// Cache created objects
pub struct CachedFactory {
    cache: std::collections::HashMap<String, Box<dyn Animal>>,
}

impl CachedFactory {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, name: &str, type_name: &str) -> &dyn Animal {
        if !self.cache.contains_key(name) {
            let animal = AnimalFactory::create(type_name, name);
            self.cache.insert(name.to_string(), animal);
        }
        self.cache.get(name).unwrap().as_ref()
    }
}

/// Problem 7: Abstract factory
/// Create families of related objects
pub trait GUIFactory {
    fn create_button(&self) -> Box<dyn Button>;
    fn create_checkbox(&self) -> Box<dyn Checkbox>;
}

pub trait Button {
    fn render(&self) -> String;
}

pub trait Checkbox {
    fn render(&self) -> String;
}

pub struct WindowsButton;
pub struct WindowsCheckbox;
pub struct LinuxButton;
pub struct LinuxCheckbox;

impl Button for WindowsButton {
    fn render(&self) -> String {
        "Windows Button".to_string()
    }
}

impl Checkbox for WindowsCheckbox {
    fn render(&self) -> String {
        "Windows Checkbox".to_string()
    }
}

impl Button for LinuxButton {
    fn render(&self) -> String {
        "Linux Button".to_string()
    }
}

impl Checkbox for LinuxCheckbox {
    fn render(&self) -> String {
        "Linux Checkbox".to_string()
    }
}

pub struct WindowsFactory;
pub struct LinuxFactory;

impl GUIFactory for WindowsFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(WindowsButton)
    }

    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(WindowsCheckbox)
    }
}

impl GUIFactory for LinuxFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(LinuxButton)
    }

    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(LinuxCheckbox)
    }
}

/// Problem 8: Factory with generics
/// Generic factory
pub trait GenericFactory<T> {
    fn create(&self) -> T;
}

pub struct I32Factory;
pub struct StringFactory;

impl GenericFactory<i32> for I32Factory {
    fn create(&self) -> i32 {
        0
    }
}

impl GenericFactory<String> for StringFactory {
    fn create(&self) -> String {
        String::new()
    }
}

/// Problem 9: Factory with closure
/// Use closures as factory
pub struct ClosureFactory {
    creator: Box<dyn Fn() -> i32>,
}

impl ClosureFactory {
    pub fn new(creator: Box<dyn Fn() -> i32>) -> Self {
        Self { creator }
    }

    pub fn create(&self) -> i32 {
        (self.creator)()
    }
}

/// Problem 10: Factory with configuration
/// Factory based on configuration
#[derive(Debug, Clone)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
}

pub struct DatabaseFactory;

impl DatabaseFactory {
    pub fn create(config: &DatabaseConfig) -> String {
        match config.db_type {
            DatabaseType::PostgreSQL => {
                format!("postgres://{}:{}", config.host, config.port)
            }
            DatabaseType::MySQL => {
                format!("mysql://{}:{}", config.host, config.port)
            }
            DatabaseType::SQLite => {
                "sqlite://:memory:".to_string()
            }
        }
    }
}

/// Problem 11: Factory with error handling
/// Handle factory errors
pub enum FactoryError {
    InvalidType(String),
    MissingParameter(String),
    ValidationError(String),
}

pub struct SafeFactory;

impl SafeFactory {
    pub fn create_user(name: &str, email: &str) -> Result<User, FactoryError> {
        if name.is_empty() {
            return Err(FactoryError::MissingParameter("name".to_string()));
        }
        if !email.contains('@') {
            return Err(FactoryError::ValidationError("Invalid email".to_string()));
        }
        Ok(User {
            id: 0,
            name: name.to_string(),
            email: email.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

/// Problem 12: Factory with trait objects
/// Create trait objects
pub trait Serializable {
    fn serialize(&self) -> String;
}

pub struct JsonSerializable {
    data: String,
}

pub struct XmlSerializable {
    data: String,
}

impl Serializable for JsonSerializable {
    fn serialize(&self) -> String {
        format!("{{\"data\":\"{}\"}}", self.data)
    }
}

impl Serializable for XmlSerializable {
    fn serialize(&self) -> String {
        format!("<data>{}</data>", self.data)
    }
}

pub struct SerializableFactory;

impl SerializableFactory {
    pub fn create(format: &str, data: &str) -> Box<dyn Serializable> {
        match format {
            "json" => Box::new(JsonSerializable {
                data: data.to_string(),
            }),
            "xml" => Box::new(XmlSerializable {
                data: data.to_string(),
            }),
            _ => panic!("Unknown format"),
        }
    }
}

/// Problem 13: Factory with multiple methods
/// Multiple factory methods
pub struct LoggerFactory;

impl LoggerFactory {
    pub fn console_logger() -> String {
        "Console Logger".to_string()
    }

    pub fn file_logger(path: &str) -> String {
        format!("File Logger: {}", path)
    }

    pub fn remote_logger(url: &str) -> String {
        format!("Remote Logger: {}", url)
    }
}

/// Problem 14: Factory with builder pattern
/// Factory returning builder
pub struct QueryFactory;

impl QueryFactory {
    pub fn select(table: &str) -> QueryBuilder {
        QueryBuilder::new(table)
    }
}

pub struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            conditions: Vec::new(),
        }
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn build(self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }
        query
    }
}

/// Problem 15: Factory with dependency injection
/// Inject dependencies into factory
pub trait Logger {
    fn log(&self, message: &str);
}

pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}

pub struct ServiceFactory {
    logger: Box<dyn Logger>,
}

impl ServiceFactory {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        Self { logger }
    }

    pub fn create_service(&self) -> String {
        self.logger.log("Creating service");
        "Service".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_factory() {
        let dog = AnimalFactory::create("dog", "Buddy");
        assert_eq!(dog.speak(), "Woof!");
    }

    #[test]
    fn test_shape_factory() {
        let circle = ShapeFactory::create("circle", &[5.0]);
        assert!(matches!(circle, Shape::Circle(5.0)));
    }

    #[test]
    fn test_config_factory() {
        let dev_config = ConfigFactory::development();
        assert!(dev_config.debug);
        let prod_config = ConfigFactory::production();
        assert!(!prod_config.debug);
    }

    #[test]
    fn test_validating_factory() {
        assert!(ValidatingFactory::create_email("test@example.com").is_ok());
        assert!(ValidatingFactory::create_email("invalid").is_err());
    }

    #[test]
    fn test_abstract_factory() {
        let factory = WindowsFactory;
        let button = factory.create_button();
        assert_eq!(button.render(), "Windows Button");
    }

    #[test]
    fn test_generic_factory() {
        let factory = I32Factory;
        assert_eq!(factory.create(), 0);
    }

    #[test]
    fn test_closure_factory() {
        let factory = ClosureFactory::new(Box::new(|| 42));
        assert_eq!(factory.create(), 42);
    }

    #[test]
    fn test_database_factory() {
        let config = DatabaseConfig {
            db_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
        };
        assert_eq!(DatabaseFactory::create(&config), "postgres://localhost:5432");
    }

    #[test]
    fn test_safe_factory() {
        assert!(SafeFactory::create_user("Alice", "alice@example.com").is_ok());
        assert!(SafeFactory::create_user("", "alice@example.com").is_err());
    }

    #[test]
    fn test_serializable_factory() {
        let json = SerializableFactory::create("json", "test");
        assert_eq!(json.serialize(), "{\"data\":\"test\"}");
    }

    #[test]
    fn test_logger_factory() {
        assert_eq!(LoggerFactory::console_logger(), "Console Logger");
    }

    #[test]
    fn test_query_factory() {
        let query = QueryFactory::select("users")
            .where_clause("age > 18")
            .build();
        assert!(query.contains("users"));
    }

    #[test]
    fn test_service_factory() {
        let factory = ServiceFactory::new(Box::new(ConsoleLogger));
        assert_eq!(factory.create_service(), "Service");
    }
}
