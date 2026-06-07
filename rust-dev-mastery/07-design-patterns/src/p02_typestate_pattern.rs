//! # Typestate Pattern
//!
//! The typestate pattern encodes object state in the type system, making invalid
//! state transitions a compile error. Each state is a separate type, and methods
//! that change state consume `self` and return a new type.
//!
//! ## Key Concepts
//! - **Phantom types**: Zero-sized types used only at the type level
//! - **State transitions**: Methods consume `self` and return `NewState`
//! - **Compile-time safety**: Invalid operations become compile errors
//! - **Zero runtime cost**: Phantom types are erased at compile time

use std::marker::PhantomData;

/// A file handle that can only be read after being opened and only written
/// after being opened. Closed files cannot be read or written.
pub struct File<State> {
    path: String,
    _state: PhantomData<State>,
}

pub struct Closed;
pub struct Opened;
pub struct Written;

impl File<Closed> {
    pub fn new(path: impl Into<String>) -> Self {
        File {
            path: path.into(),
            _state: PhantomData,
        }
    }

    /// Opens the file. Only closed files can be opened.
    pub fn open(self) -> Result<File<Opened>, FileError> {
        Ok(File {
            path: self.path,
            _state: PhantomData,
        })
    }
}

impl File<Opened> {
    pub fn read(&self) -> Result<Vec<u8>, FileError> {
        Ok(format!("contents of {}", self.path).into_bytes())
    }

    pub fn write(self, _data: &[u8]) -> Result<File<Written>, FileError> {
        Ok(File {
            path: self.path,
            _state: PhantomData,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl File<Written> {
    pub fn flush(self) -> Result<File<Opened>, FileError> {
        Ok(File {
            path: self.path,
            _state: PhantomData,
        })
    }

    pub fn close(self) -> File<Closed> {
        File {
            path: self.path,
            _state: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileError {
    NotFound,
    PermissionDenied,
    WriteError,
}

/// A network connection with compile-time state tracking.
/// Demonstrates a multi-state lifecycle: Disconnected -> Connected -> Authenticated.
pub struct Connection<State> {
    host: String,
    session_id: Option<String>,
    _state: PhantomData<State>,
}

pub struct Disconnected;
pub struct Connected;
pub struct Authenticated;

impl Connection<Disconnected> {
    pub fn new(host: impl Into<String>) -> Self {
        Connection {
            host: host.into(),
            session_id: None,
            _state: PhantomData,
        }
    }

    pub fn connect(self) -> Result<Connection<Connected>, ConnectionError> {
        Ok(Connection {
            host: self.host,
            session_id: Some("session-123".to_string()),
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    pub fn authenticate(self, _token: &str) -> Result<Connection<Authenticated>, ConnectionError> {
        Ok(Connection {
            host: self.host,
            session_id: self.session_id,
            _state: PhantomData,
        })
    }

    pub fn disconnect(self) -> Connection<Disconnected> {
        Connection {
            host: self.host,
            session_id: None,
            _state: PhantomData,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

impl Connection<Authenticated> {
    pub fn query(&self, _sql: &str) -> Result<Vec<String>, ConnectionError> {
        Ok(vec!["row1".into(), "row2".into()])
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn session_id(&self) -> &str {
        self.session_id.as_ref().unwrap()
    }

    pub fn disconnect(self) -> Connection<Disconnected> {
        Connection {
            host: self.host,
            session_id: None,
            _state: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionError {
    ConnectionRefused,
    AuthenticationFailed,
    QueryError,
}

/// A request builder where the type encodes which parts have been configured.
/// This ensures that a request can only be sent when the URL has been set.
pub struct Request<State> {
    url: Option<String>,
    method: String,
    body: Option<Vec<u8>>,
    _state: PhantomData<State>,
}

pub struct NoUrl;
pub struct HasUrl;

impl Request<NoUrl> {
    pub fn new() -> Self {
        Request {
            url: None,
            method: "GET".to_string(),
            body: None,
            _state: PhantomData,
        }
    }

    pub fn url(self, url: impl Into<String>) -> Request<HasUrl> {
        Request {
            url: Some(url.into()),
            method: self.method,
            body: self.body,
            _state: PhantomData,
        }
    }
}

impl<State> Request<State> {
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

impl Request<HasUrl> {
    pub fn send(&self) -> Result<Response, RequestError> {
        Ok(Response {
            status: 200,
            body: b"OK".to_vec(),
        })
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestError {
    NoUrl,
    NetworkError,
}

/// A state machine for order processing using typestates.
#[derive(Debug, Clone)]
pub struct Order<State> {
    id: u64,
    items: Vec<OrderItem>,
    total: f64,
    _state: PhantomData<State>,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub name: String,
    pub quantity: u32,
    pub price: f64,
}

pub struct Created;
pub struct Validated;
pub struct Paid;
pub struct Shipped;

impl Order<Created> {
    pub fn new(id: u64) -> Self {
        Order {
            id,
            items: Vec::new(),
            total: 0.0,
            _state: PhantomData,
        }
    }

    pub fn add_item(&mut self, name: impl Into<String>, quantity: u32, price: f64) {
        self.total += quantity as f64 * price;
        self.items.push(OrderItem {
            name: name.into(),
            quantity,
            price,
        });
    }

    pub fn validate(self) -> Result<Order<Validated>, OrderError> {
        if self.items.is_empty() {
            return Err(OrderError::EmptyOrder);
        }
        if self.total <= 0.0 {
            return Err(OrderError::InvalidTotal);
        }
        Ok(Order {
            id: self.id,
            items: self.items,
            total: self.total,
            _state: PhantomData,
        })
    }
}

impl Order<Validated> {
    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn items(&self) -> &[OrderItem] {
        &self.items
    }

    pub fn pay(self) -> Result<Order<Paid>, OrderError> {
        Ok(Order {
            id: self.id,
            items: self.items,
            total: self.total,
            _state: PhantomData,
        })
    }
}

impl Order<Paid> {
    pub fn ship(self) -> Order<Shipped> {
        Order {
            id: self.id,
            items: self.items,
            total: self.total,
            _state: PhantomData,
        }
    }
}

impl Order<Shipped> {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn total(&self) -> f64 {
        self.total
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderError {
    EmptyOrder,
    InvalidTotal,
    PaymentFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_lifecycle() {
        let file = File::new("test.txt");
        let file = file.open().unwrap();

        let contents = file.read().unwrap();
        assert!(!contents.is_empty());

        let file = file.write(b"new data").unwrap();
        let _file = file.flush().unwrap();
        // Note: close() is only available on File<Written>, not File<Opened>
    }

    #[test]
    fn test_file_open_read() {
        let file = File::new("data.csv").open().unwrap();
        let data = file.read().unwrap();
        assert!(String::from_utf8_lossy(&data).contains("data.csv"));
    }

    // These would be compile errors:
    // File::new("x").read(); // Can't read a closed file
    // File::new("x").write(b"y"); // Can't write to a closed file
    // File::new("x").close(); // Can't close an already-closed file (well, you can, but it's the same type)

    #[test]
    fn test_connection_lifecycle() {
        let conn = Connection::new("localhost:5432");
        let conn = conn.connect().unwrap();
        assert_eq!(conn.host(), "localhost:5432");

        let conn = conn.authenticate("token123").unwrap();
        assert_eq!(conn.session_id(), "session-123");

        let results = conn.query("SELECT * FROM users").unwrap();
        assert_eq!(results.len(), 2);

        let _disconnected = conn.disconnect();
    }

    // Compile errors:
    // Connection::new("x").query("..."); // Can't query disconnected
    // Connection::new("x").connect().unwrap().query("..."); // Can't query unauthenticated

    #[test]
    fn test_request_builder() {
        let response = Request::new()
            .url("https://api.example.com")
            .method("POST")
            .body(b"{}".to_vec())
            .send()
            .unwrap();

        assert_eq!(response.status, 200);
    }

    // Compile error:
    // Request::new().send(); // No URL set

    #[test]
    fn test_order_lifecycle() {
        let mut order = Order::new(1);
        order.add_item("Widget", 2, 9.99);
        order.add_item("Gadget", 1, 24.99);

        let validated = order.validate().unwrap();
        assert!((validated.total() - 44.97).abs() < 0.01);

        let paid = validated.pay().unwrap();
        let shipped = paid.ship();
        assert_eq!(shipped.id(), 1);
    }

    #[test]
    fn test_order_empty_validation() {
        let order = Order::new(1);
        let result = order.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_order_items() {
        let mut order = Order::new(42);
        order.add_item("Book", 3, 15.0);
        order.add_item("Pen", 10, 1.5);

        let validated = order.validate().unwrap();
        assert_eq!(validated.items().len(), 2);
        assert!((validated.total() - 60.0).abs() < 0.01);
    }

    // Compile errors for invalid transitions:
    // Order::new(1).pay(); // Can't pay an unvalidated order
    // Order::new(1).ship(); // Can't ship an unpaid order
}
