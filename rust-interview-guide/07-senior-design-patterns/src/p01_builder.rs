/// Problem: Builder Pattern
///
/// Master the builder pattern in Rust.
///
/// Key Concepts:
/// - Fluent API
/// - Method chaining
/// - Default values
/// - Validation
/// - Complex construction

/// Problem 1: Basic builder
/// Create a basic builder
#[derive(Debug)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: String,
}

pub struct UserBuilder {
    name: String,
    age: u32,
    email: String,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            age: 0,
            email: String::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn age(mut self, age: u32) -> Self {
        self.age = age;
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    pub fn build(self) -> User {
        User {
            name: self.name,
            age: self.age,
            email: self.email,
        }
    }
}

/// Problem 2: Builder with validation
/// Add validation to builder
#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

pub struct ConfigBuilder {
    host: String,
    port: u16,
    debug: bool,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            debug: false,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn build(self) -> Result<Config, String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        Ok(Config {
            host: self.host,
            port: self.port,
            debug: self.debug,
        })
    }
}

/// Problem 3: Builder with defaults
/// Provide default values
#[derive(Debug)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

pub struct ServerBuilder {
    host: String,
    port: u16,
    max_connections: u32,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: 100,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn build(self) -> Server {
        Server {
            host: self.host,
            port: self.port,
            max_connections: self.max_connections,
        }
    }
}

/// Problem 4: Builder with optional fields
/// Handle optional fields
#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
}

pub struct ProfileBuilder {
    name: String,
    bio: Option<String>,
    website: Option<String>,
}

impl ProfileBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bio: None,
            website: None,
        }
    }

    pub fn bio(mut self, bio: &str) -> Self {
        self.bio = Some(bio.to_string());
        self
    }

    pub fn website(mut self, website: &str) -> Self {
        self.website = Some(website.to_string());
        self
    }

    pub fn build(self) -> Profile {
        Profile {
            name: self.name,
            bio: self.bio,
            website: self.website,
        }
    }
}

/// Problem 5: Builder with nested builders
/// Use nested builders
#[derive(Debug)]
pub struct Database {
    pub connection: Connection,
    pub pool_size: u32,
}

#[derive(Debug)]
pub struct Connection {
    pub host: String,
    pub port: u16,
}

pub struct DatabaseBuilder {
    host: String,
    port: u16,
    pool_size: u32,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            pool_size: 10,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    pub fn build(self) -> Database {
        Database {
            connection: Connection {
                host: self.host,
                port: self.port,
            },
            pool_size: self.pool_size,
        }
    }
}

/// Problem 6: Builder with collections
/// Handle collections in builder
#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub members: Vec<String>,
}

pub struct TeamBuilder {
    name: String,
    members: Vec<String>,
}

impl TeamBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add_member(mut self, member: &str) -> Self {
        self.members.push(member.to_string());
        self
    }

    pub fn build(self) -> Team {
        Team {
            name: self.name,
            members: self.members,
        }
    }
}

/// Problem 7: Builder with generics
/// Generic builder
#[derive(Debug)]
pub struct Container<T> {
    pub value: T,
    pub label: String,
}

pub struct ContainerBuilder<T> {
    value: Option<T>,
    label: String,
}

impl<T> ContainerBuilder<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            label: String::new(),
        }
    }

    pub fn value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn build(self) -> Result<Container<T>, String> {
        Ok(Container {
            value: self.value.ok_or("Value is required")?,
            label: self.label,
        })
    }
}

/// Problem 8: Builder with lifetime
/// Builder with lifetime
#[derive(Debug)]
pub struct Reference<'a> {
    pub data: &'a str,
    pub name: String,
}

pub struct ReferenceBuilder<'a> {
    data: &'a str,
    name: String,
}

impl<'a> ReferenceBuilder<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
            name: String::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn build(self) -> Reference<'a> {
        Reference {
            data: self.data,
            name: self.name,
        }
    }
}

/// Problem 9: Builder with error handling
/// Handle errors in builder
#[derive(Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
}

pub struct RequestBuilder {
    url: Option<String>,
    method: String,
    headers: Vec<(String, String)>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            method: "GET".to_string(),
            headers: Vec::new(),
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn build(self) -> Result<Request, String> {
        Ok(Request {
            url: self.url.ok_or("URL is required")?,
            method: self.method,
            headers: self.headers,
        })
    }
}

/// Problem 10: Builder with consume pattern
/// Consume builder on build
#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub struct DocumentBuilder {
    title: String,
    content: String,
    tags: Vec<String>,
}

impl DocumentBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: String::new(),
            tags: Vec::new(),
        }
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn build(self) -> Document {
        Document {
            title: self.title,
            content: self.content,
            tags: self.tags,
        }
    }
}

/// Problem 11: Builder with static method
/// Use static method for builder
#[derive(Debug)]
pub struct Query {
    pub table: String,
    pub conditions: Vec<String>,
    pub limit: Option<usize>,
}

impl Query {
    pub fn builder(table: &str) -> QueryBuilder {
        QueryBuilder::new(table)
    }
}

pub struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
    limit: Option<usize>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            conditions: Vec::new(),
            limit: None,
        }
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn build(self) -> Query {
        Query {
            table: self.table,
            conditions: self.conditions,
            limit: self.limit,
        }
    }
}

/// Problem 12: Builder with clone
/// Clone builder for reuse
#[derive(Debug, Clone)]
pub struct Settings {
    pub theme: String,
    pub font_size: u32,
}

#[derive(Clone)]
pub struct SettingsBuilder {
    theme: String,
    font_size: u32,
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Self {
            theme: "light".to_string(),
            font_size: 14,
        }
    }

    pub fn theme(mut self, theme: &str) -> Self {
        self.theme = theme.to_string();
        self
    }

    pub fn font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }

    pub fn build(self) -> Settings {
        Settings {
            theme: self.theme,
            font_size: self.font_size,
        }
    }
}

/// Problem 13: Builder with validation chain
/// Chain validations
#[derive(Debug)]
pub struct Form {
    pub name: String,
    pub email: String,
    pub age: u32,
}

pub struct FormBuilder {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            email: None,
            age: None,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    pub fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    pub fn build(self) -> Result<Form, String> {
        let name = self.name.ok_or("Name is required")?;
        let email = self.email.ok_or("Email is required")?;
        let age = self.age.ok_or("Age is required")?;

        if !email.contains('@') {
            return Err("Invalid email".to_string());
        }

        Ok(Form { name, email, age })
    }
}

/// Problem 14: Builder with default trait
/// Implement Default for builder
#[derive(Debug)]
pub struct Options {
    pub verbose: bool,
    pub color: bool,
    pub output: String,
}

pub struct OptionsBuilder {
    verbose: bool,
    color: bool,
    output: String,
}

impl Default for OptionsBuilder {
    fn default() -> Self {
        Self {
            verbose: false,
            color: true,
            output: "stdout".to_string(),
        }
    }
}

impl OptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    pub fn output(mut self, output: &str) -> Self {
        self.output = output.to_string();
        self
    }

    pub fn build(self) -> Options {
        Options {
            verbose: self.verbose,
            color: self.color,
            output: self.output,
        }
    }
}

/// Problem 15: Builder with into
/// Convert builder into target type
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

pub struct ResponseBuilder {
    status: u16,
    body: String,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status: 200,
            body: String::new(),
        }
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }
}

impl From<ResponseBuilder> for Response {
    fn from(builder: ResponseBuilder) -> Self {
        Response {
            status: builder.status,
            body: builder.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_builder() {
        let user = UserBuilder::new()
            .name("Alice")
            .age(30)
            .email("alice@example.com")
            .build();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_builder_with_validation() {
        let config = ConfigBuilder::new()
            .host("example.com")
            .port(3000)
            .build();
        assert!(config.is_ok());
    }

    #[test]
    fn test_builder_with_defaults() {
        let server = ServerBuilder::new().build();
        assert_eq!(server.host, "0.0.0.0");
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_builder_with_optional() {
        let profile = ProfileBuilder::new("Alice")
            .bio("Hello")
            .build();
        assert_eq!(profile.bio, Some("Hello".to_string()));
    }

    #[test]
    fn test_builder_with_nested() {
        let db = DatabaseBuilder::new()
            .host("db.example.com")
            .port(5432)
            .build();
        assert_eq!(db.connection.host, "db.example.com");
    }

    #[test]
    fn test_builder_with_collections() {
        let team = TeamBuilder::new("Engineering")
            .add_member("Alice")
            .add_member("Bob")
            .build();
        assert_eq!(team.members.len(), 2);
    }

    #[test]
    fn test_builder_with_generics() {
        let container = ContainerBuilder::new()
            .value(42)
            .label("test")
            .build()
            .unwrap();
        assert_eq!(container.value, 42);
    }

    #[test]
    fn test_builder_with_lifetime() {
        let data = "hello";
        let reference = ReferenceBuilder::new(data)
            .name("test")
            .build();
        assert_eq!(reference.data, "hello");
    }

    #[test]
    fn test_builder_with_error() {
        let request = RequestBuilder::new()
            .url("https://example.com")
            .build();
        assert!(request.is_ok());
    }

    #[test]
    fn test_builder_consume() {
        let doc = DocumentBuilder::new("Title")
            .content("Content")
            .tag("rust")
            .build();
        assert_eq!(doc.title, "Title");
    }

    #[test]
    fn test_builder_static_method() {
        let query = Query::builder("users")
            .where_clause("age > 18")
            .limit(10)
            .build();
        assert_eq!(query.table, "users");
    }

    #[test]
    fn test_builder_clone() {
        let builder = SettingsBuilder::new();
        let settings1 = builder.clone().theme("dark").build();
        let settings2 = builder.theme("light").build();
        assert_eq!(settings1.theme, "dark");
        assert_eq!(settings2.theme, "light");
    }

    #[test]
    fn test_builder_validation_chain() {
        let form = FormBuilder::new()
            .name("Alice")
            .email("alice@example.com")
            .age(30)
            .build();
        assert!(form.is_ok());
    }

    #[test]
    fn test_builder_default() {
        let options = OptionsBuilder::new().build();
        assert!(!options.verbose);
        assert!(options.color);
    }

    #[test]
    fn test_builder_into() {
        let response: Response = ResponseBuilder::new()
            .status(200)
            .body("OK")
            .into();
        assert_eq!(response.status, 200);
    }
}
