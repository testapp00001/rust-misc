/// Problem: Adapter Pattern
///
/// Master the adapter pattern in Rust.
///
/// Key Concepts:
/// - Target trait
/// - Adaptee
/// - Adapter
/// - Interface conversion
/// - Compatibility

/// Problem 1: Basic adapter
/// Create basic adapter
pub trait Target {
    fn request(&self) -> String;
}

pub struct Adaptee {
    name: String,
}

impl Adaptee {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn specific_request(&self) -> String {
        format!("Adaptee: {}", self.name)
    }
}

pub struct Adapter {
    adaptee: Adaptee,
}

impl Adapter {
    pub fn new(adaptee: Adaptee) -> Self {
        Self { adaptee }
    }
}

impl Target for Adapter {
    fn request(&self) -> String {
        self.adaptee.specific_request()
    }
}

/// Problem 2: Two-way adapter
/// Adapter that works both ways
pub trait InterfaceA {
    fn method_a(&self) -> String;
}

pub trait InterfaceB {
    fn method_b(&self) -> String;
}

pub struct TwoWayAdapter {
    data: String,
}

impl TwoWayAdapter {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

impl InterfaceA for TwoWayAdapter {
    fn method_a(&self) -> String {
        format!("A: {}", self.data)
    }
}

impl InterfaceB for TwoWayAdapter {
    fn method_b(&self) -> String {
        format!("B: {}", self.data)
    }
}

/// Problem 3: Class adapter
/// Adapter using inheritance (simulated with trait)
pub trait OldInterface {
    fn old_method(&self) -> String;
}

pub trait NewInterface {
    fn new_method(&self) -> String;
}

pub struct OldClass {
    data: String,
}

impl OldClass {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

impl OldInterface for OldClass {
    fn old_method(&self) -> String {
        format!("Old: {}", self.data)
    }
}

pub struct ClassAdapter {
    old: OldClass,
}

impl ClassAdapter {
    pub fn new(data: &str) -> Self {
        Self {
            old: OldClass::new(data),
        }
    }
}

impl NewInterface for ClassAdapter {
    fn new_method(&self) -> String {
        self.old.old_method()
    }
}

/// Problem 4: Object adapter
/// Adapter using composition
pub struct ObjectAdapter {
    adaptee: Box<dyn OldInterface>,
}

impl ObjectAdapter {
    pub fn new(adaptee: Box<dyn OldInterface>) -> Self {
        Self { adaptee }
    }
}

impl NewInterface for ObjectAdapter {
    fn new_method(&self) -> String {
        self.adaptee.old_method()
    }
}

/// Problem 5: Collection adapter
/// Adapt collection types
pub struct VecAdapter {
    data: Vec<i32>,
}

impl VecAdapter {
    pub fn new(data: Vec<i32>) -> Self {
        Self { data }
    }

    pub fn to_array(&self) -> [i32; 3] {
        [self.data[0], self.data[1], self.data[2]]
    }

    pub fn to_tuple(&self) -> (i32, i32, i32) {
        (self.data[0], self.data[1], self.data[2])
    }
}

/// Problem 6: Format adapter
/// Adapt between formats
pub trait JsonFormat {
    fn to_json(&self) -> String;
}

pub trait XmlFormat {
    fn to_xml(&self) -> String;
}

pub struct Data {
    name: String,
    value: i32,
}

impl Data {
    pub fn new(name: &str, value: i32) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

impl JsonFormat for Data {
    fn to_json(&self) -> String {
        format!("{{\"name\":\"{}\",\"value\":{}}}", self.name, self.value)
    }
}

pub struct JsonToXmlAdapter {
    data: Data,
}

impl JsonToXmlAdapter {
    pub fn new(data: Data) -> Self {
        Self { data }
    }
}

impl XmlFormat for JsonToXmlAdapter {
    fn to_xml(&self) -> String {
        format!("<data><name>{}</name><value>{}</value></data>", self.data.name, self.data.value)
    }
}

/// Problem 7: API adapter
/// Adapt API responses
pub trait OldApi {
    fn get_data(&self) -> Vec<String>;
}

pub trait NewApi {
    fn fetch_items(&self) -> Vec<Item>;
}

pub struct Item {
    pub id: u32,
    pub name: String,
}

pub struct OldApiClient;

impl OldApi for OldApiClient {
    fn get_data(&self) -> Vec<String> {
        vec!["item1".to_string(), "item2".to_string()]
    }
}

pub struct ApiAdapter {
    old_api: Box<dyn OldApi>,
}

impl ApiAdapter {
    pub fn new(old_api: Box<dyn OldApi>) -> Self {
        Self { old_api }
    }
}

impl NewApi for ApiAdapter {
    fn fetch_items(&self) -> Vec<Item> {
        self.old_api
            .get_data()
            .into_iter()
            .enumerate()
            .map(|(i, name)| Item {
                id: i as u32,
                name,
            })
            .collect()
    }
}

/// Problem 8: Database adapter
/// Adapt database interfaces
pub trait SqlDatabase {
    fn query(&self, sql: &str) -> Vec<String>;
}

pub trait NoSqlDatabase {
    fn find(&self, collection: &str) -> Vec<String>;
}

pub struct SqlClient;

impl SqlDatabase for SqlClient {
    fn query(&self, sql: &str) -> Vec<String> {
        vec![format!("Result of: {}", sql)]
    }
}

pub struct SqlToNoSqlAdapter {
    sql: Box<dyn SqlDatabase>,
}

impl SqlToNoSqlAdapter {
    pub fn new(sql: Box<dyn SqlDatabase>) -> Self {
        Self { sql }
    }
}

impl NoSqlDatabase for SqlToNoSqlAdapter {
    fn find(&self, collection: &str) -> Vec<String> {
        self.sql.query(&format!("SELECT * FROM {}", collection))
    }
}

/// Problem 9: Logger adapter
/// Adapt logger interfaces
pub trait OldLogger {
    fn log_message(&self, message: &str);
}

pub trait NewLogger {
    fn info(&self, message: &str);
    fn error(&self, message: &str);
}

pub struct OldLoggerImpl {
    messages: std::sync::Mutex<Vec<String>>,
}

impl OldLoggerImpl {
    pub fn new() -> Self {
        Self {
            messages: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn get_messages(&self) -> Vec<String> {
let result =         self.messages.lock().unwrap().clone(); result
    }
}

impl OldLogger for OldLoggerImpl {
    fn log_message(&self, message: &str) {
        self.messages.lock().unwrap().push(message.to_string());
    }
}

pub struct LoggerAdapter {
    old_logger: Box<dyn OldLogger>,
}

impl LoggerAdapter {
    pub fn new(old_logger: Box<dyn OldLogger>) -> Self {
        Self { old_logger }
    }
}

impl NewLogger for LoggerAdapter {
    fn info(&self, message: &str) {
        self.old_logger.log_message(&format!("[INFO] {}", message));
    }

    fn error(&self, message: &str) {
        self.old_logger.log_message(&format!("[ERROR] {}", message));
    }
}

/// Problem 10: Cache adapter
/// Adapt cache interfaces
pub trait MemoryCache {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
}

pub trait RedisCache {
    fn get_redis(&self, key: &str) -> Option<String>;
    fn set_redis(&self, key: &str, value: &str);
}

pub struct MemoryCacheImpl {
    data: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl MemoryCacheImpl {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl MemoryCache for MemoryCacheImpl {
    fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) {
        self.data.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}

pub struct CacheAdapter {
    cache: Box<dyn MemoryCache>,
}

impl CacheAdapter {
    pub fn new(cache: Box<dyn MemoryCache>) -> Self {
        Self { cache }
    }
}

impl RedisCache for CacheAdapter {
    fn get_redis(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    fn set_redis(&self, key: &str, value: &str) {
        self.cache.set(key, value);
    }
}

/// Problem 11: Serialization adapter
/// Adapt serialization
pub trait JsonSerializer {
    fn serialize_json(&self, data: &str) -> String;
}

pub trait MessagePackSerializer {
    fn serialize_msgpack(&self, data: &str) -> String;
}

pub struct JsonSerde;

impl JsonSerializer for JsonSerde {
    fn serialize_json(&self, data: &str) -> String {
        format!("{{\"data\":\"{}\"}}", data)
    }
}

pub struct JsonToMsgPackAdapter {
    json: Box<dyn JsonSerializer>,
}

impl JsonToMsgPackAdapter {
    pub fn new(json: Box<dyn JsonSerializer>) -> Self {
        Self { json }
    }
}

impl MessagePackSerializer for JsonToMsgPackAdapter {
    fn serialize_msgpack(&self, data: &str) -> String {
        let json = self.json.serialize_json(data);
        format!("msgpack({})", json)
    }
}

/// Problem 12: Authentication adapter
/// Adapt authentication
pub trait BasicAuth {
    fn authenticate_basic(&self, username: &str, password: &str) -> bool;
}

pub trait TokenAuth {
    fn authenticate_token(&self, token: &str) -> bool;
}

pub struct BasicAuthImpl {
    users: std::collections::HashMap<String, String>,
}

impl BasicAuthImpl {
    pub fn new() -> Self {
        let mut users = std::collections::HashMap::new();
        users.insert("admin".to_string(), "password".to_string());
        Self { users }
    }
}

impl BasicAuth for BasicAuthImpl {
    fn authenticate_basic(&self, username: &str, password: &str) -> bool {
        self.users.get(username).map_or(false, |p| p == password)
    }
}

pub struct AuthAdapter {
    basic: Box<dyn BasicAuth>,
}

impl AuthAdapter {
    pub fn new(basic: Box<dyn BasicAuth>) -> Self {
        Self { basic }
    }
}

impl TokenAuth for AuthAdapter {
    fn authenticate_token(&self, token: &str) -> bool {
        // Simulate token validation
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() == 2 {
            self.basic.authenticate_basic(parts[0], parts[1])
        } else {
            false
        }
    }
}

/// Problem 13: File system adapter
/// Adapt file system interfaces
pub trait LocalFileSystem {
    fn read_local(&self, path: &str) -> String;
    fn write_local(&self, path: &str, content: &str);
}

pub trait CloudStorage {
    fn read_cloud(&self, path: &str) -> String;
    fn write_cloud(&self, path: &str, content: &str);
}

pub struct LocalFs;

impl LocalFileSystem for LocalFs {
    fn read_local(&self, path: &str) -> String {
        format!("Local content of {}", path)
    }

    fn write_local(&self, path: &str, content: &str) {
        let _ = (path, content);
    }
}

pub struct CloudAdapter {
    local: Box<dyn LocalFileSystem>,
}

impl CloudAdapter {
    pub fn new(local: Box<dyn LocalFileSystem>) -> Self {
        Self { local }
    }
}

impl CloudStorage for CloudAdapter {
    fn read_cloud(&self, path: &str) -> String {
        self.local.read_local(path)
    }

    fn write_cloud(&self, path: &str, content: &str) {
        self.local.write_local(path, content);
    }
}

/// Problem 14: Notification adapter
/// Adapt notification interfaces
pub trait EmailSender {
    fn send_email(&self, to: &str, subject: &str, body: &str);
}

pub trait SlackNotifier {
    fn send_slack(&self, channel: &str, message: &str);
}

pub struct EmailSenderImpl;

impl EmailSender for EmailSenderImpl {
    fn send_email(&self, to: &str, subject: &str, body: &str) {
        let _ = (to, subject, body);
    }
}

pub struct EmailToSlackAdapter {
    email: Box<dyn EmailSender>,
}

impl EmailToSlackAdapter {
    pub fn new(email: Box<dyn EmailSender>) -> Self {
        Self { email }
    }
}

impl SlackNotifier for EmailToSlackAdapter {
    fn send_slack(&self, channel: &str, message: &str) {
        self.email.send_email(channel, "Slack", message);
    }
}

/// Problem 15: Generic adapter
/// Generic adapter
pub struct GenericAdapter<T, U> {
    adaptee: T,
    converter: Box<dyn Fn(&T) -> U>,
}

impl<T, U> GenericAdapter<T, U> {
    pub fn new(adaptee: T, converter: Box<dyn Fn(&T) -> U>) -> Self {
        Self { adaptee, converter }
    }

    pub fn adapt(&self) -> U {
        (self.converter)(&self.adaptee)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_adapter() {
        let adaptee = Adaptee::new("test");
        let adapter = Adapter::new(adaptee);
        assert!(adapter.request().contains("Adaptee"));
    }

    #[test]
    fn test_two_way_adapter() {
        let adapter = TwoWayAdapter::new("test");
        assert!(adapter.method_a().contains("A"));
        assert!(adapter.method_b().contains("B"));
    }

    #[test]
    fn test_class_adapter() {
        let adapter = ClassAdapter::new("test");
        assert!(adapter.new_method().contains("Old"));
    }

    #[test]
    fn test_object_adapter() {
        let old = OldClass::new("test");
        let adapter = ObjectAdapter::new(Box::new(old));
        assert!(adapter.new_method().contains("Old"));
    }

    #[test]
    fn test_vec_adapter() {
        let adapter = VecAdapter::new(vec![1, 2, 3]);
        assert_eq!(adapter.to_array(), [1, 2, 3]);
        assert_eq!(adapter.to_tuple(), (1, 2, 3));
    }

    #[test]
    fn test_format_adapter() {
        let data = Data::new("test", 42);
        let adapter = JsonToXmlAdapter::new(data);
        assert!(adapter.to_xml().contains("test"));
    }

    #[test]
    fn test_api_adapter() {
        let old_api = OldApiClient;
        let adapter = ApiAdapter::new(Box::new(old_api));
        let items = adapter.fetch_items();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_database_adapter() {
        let sql = SqlClient;
        let adapter = SqlToNoSqlAdapter::new(Box::new(sql));
        let results = adapter.find("users");
        assert!(results[0].contains("SELECT"));
    }

    #[test]
    fn test_logger_adapter() {
        let old_logger = OldLoggerImpl::new();
        let adapter = LoggerAdapter::new(Box::new(old_logger));
        adapter.info("test message");
    }

    #[test]
    fn test_cache_adapter() {
        let cache = MemoryCacheImpl::new();
        let adapter = CacheAdapter::new(Box::new(cache));
        adapter.set_redis("key", "value");
        assert_eq!(adapter.get_redis("key"), Some("value".to_string()));
    }

    #[test]
    fn test_serialization_adapter() {
        let json = JsonSerde;
        let adapter = JsonToMsgPackAdapter::new(Box::new(json));
        assert!(adapter.serialize_msgpack("test").contains("msgpack"));
    }

    #[test]
    fn test_auth_adapter() {
        let basic = BasicAuthImpl::new();
        let adapter = AuthAdapter::new(Box::new(basic));
        assert!(adapter.authenticate_token("admin:password"));
    }

    #[test]
    fn test_cloud_adapter() {
        let local = LocalFs;
        let adapter = CloudAdapter::new(Box::new(local));
        assert!(adapter.read_cloud("test.txt").contains("Local"));
    }

    #[test]
    fn test_notification_adapter() {
        let email = EmailSenderImpl;
        let adapter = EmailToSlackAdapter::new(Box::new(email));
        adapter.send_slack("#general", "Hello");
    }

    #[test]
    fn test_generic_adapter() {
        let adapter = GenericAdapter::new(42, Box::new(|x| x.to_string()));
        assert_eq!(adapter.adapt(), "42");
    }
}
