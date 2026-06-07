/// Problem: Extensibility
///
/// Master extensibility in Rust.
///
/// Key Concepts:
/// - Plugin systems
/// - Trait objects
/// - Generics
/// - Middleware
/// - Hooks

/// Problem 1: Plugin system
/// Design plugin system
pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self, input: &str) -> String;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn execute(&self, input: &str) -> Vec<String> {
        self.plugins.iter().map(|p| p.execute(input)).collect()
    }
}

/// Problem 2: Middleware
/// Design middleware
pub trait Middleware {
    fn handle(&self, request: &str) -> String;
}

impl<F: Fn(&str) -> String> Middleware for F {
    fn handle(&self, request: &str) -> String {
        self(request)
    }
}

pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add(&mut self, middleware: Box<dyn Middleware>) {
        self.middlewares.push(middleware);
    }

    pub fn execute(&self, request: &str) -> String {
        let mut result = request.to_string();
        for middleware in &self.middlewares {
            result = middleware.handle(&result);
        }
        result
    }
}

/// Problem 3: Hooks
/// Design hook system
pub struct HookSystem {
    hooks: std::collections::HashMap<String, Vec<Box<dyn Fn(&str)>>>,
}

impl HookSystem {
    pub fn new() -> Self {
        Self {
            hooks: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, event: &str, hook: Box<dyn Fn(&str)>) {
        self.hooks
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(hook);
    }

    pub fn trigger(&self, event: &str, data: &str) {
        if let Some(hooks) = self.hooks.get(event) {
            for hook in hooks {
                hook(data);
            }
        }
    }
}

/// Problem 4: Strategy pattern
/// Design strategy pattern
pub trait Strategy {
    fn execute(&self, data: &[i32]) -> i32;
}

pub struct SumStrategy;
pub struct MaxStrategy;

impl Strategy for SumStrategy {
    fn execute(&self, data: &[i32]) -> i32 {
        data.iter().sum()
    }
}

impl Strategy for MaxStrategy {
    fn execute(&self, data: &[i32]) -> i32 {
        *data.iter().max().unwrap_or(&0)
    }
}

pub struct Context {
    strategy: Box<dyn Strategy>,
}

impl Context {
    pub fn new(strategy: Box<dyn Strategy>) -> Self {
        Self { strategy }
    }

    pub fn execute(&self, data: &[i32]) -> i32 {
        self.strategy.execute(data)
    }
}

/// Problem 5: Observer pattern
/// Design observer pattern
pub trait Observer {
    fn update(&self, event: &str);
}

pub struct EventSystem {
    observers: Vec<Box<dyn Observer>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self { observers: Vec::new() }
    }

    pub fn subscribe(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, event: &str) {
        for observer in &self.observers {
            observer.update(event);
        }
    }
}

/// Problem 6: Decorator pattern
/// Design decorator pattern
pub trait Component {
    fn operation(&self) -> String;
}

pub struct ConcreteComponent {
    name: String,
}

impl Component for ConcreteComponent {
    fn operation(&self) -> String {
        format!("Component: {}", self.name)
    }
}

pub struct LoggingDecorator {
    wrapped: Box<dyn Component>,
}

impl LoggingDecorator {
    pub fn new(wrapped: Box<dyn Component>) -> Self {
        Self { wrapped }
    }
}

impl Component for LoggingDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        println!("Log: {}", result);
        result
    }
}

/// Problem 7: Factory method
/// Design factory method
pub trait Factory {
    fn create(&self) -> Box<dyn Component>;
}

pub struct ConcreteFactory;

impl Factory for ConcreteFactory {
    fn create(&self) -> Box<dyn Component> {
        Box::new(ConcreteComponent {
            name: "default".to_string(),
        })
    }
}

/// Problem 8: Extension trait
/// Design extension trait
pub trait StringExt {
    fn is_palindrome(&self) -> bool;
    fn word_count(&self) -> usize;
}

impl StringExt for str {
    fn is_palindrome(&self) -> bool {
        let reversed: String = self.chars().rev().collect();
        self == reversed
    }

    fn word_count(&self) -> usize {
        self.split_whitespace().count()
    }
}

/// Problem 9: Generic container
/// Design generic container
pub struct Container<T> {
    items: Vec<T>,
}

impl<T> Container<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T: Clone> Container<T> {
    pub fn get_all(&self) -> Vec<T> {
        self.items.clone()
    }
}

/// Problem 10: Builder pattern
/// Design builder pattern
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
            url: self.url.ok_or("URL required")?,
            method: self.method,
            headers: self.headers,
        })
    }
}

#[derive(Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
}

/// Problem 11: Adapter pattern
/// Design adapter pattern
pub trait OldApi {
    fn old_method(&self) -> String;
}

pub trait NewApi {
    fn new_method(&self) -> String;
}

pub struct OldImpl;

impl OldApi for OldImpl {
    fn old_method(&self) -> String {
        "old".to_string()
    }
}

pub struct Adapter {
    old: Box<dyn OldApi>,
}

impl Adapter {
    pub fn new(old: Box<dyn OldApi>) -> Self {
        Self { old }
    }
}

impl NewApi for Adapter {
    fn new_method(&self) -> String {
        self.old.old_method()
    }
}

/// Problem 12: Command pattern
/// Design command pattern
pub trait Command {
    fn execute(&self) -> String;
    fn undo(&self) -> String;
}

pub struct AddCommand {
    value: i32,
    receiver: std::sync::Arc<std::sync::Mutex<i32>>,
}

impl AddCommand {
    pub fn new(value: i32, receiver: std::sync::Arc<std::sync::Mutex<i32>>) -> Self {
        Self { value, receiver }
    }
}

impl Command for AddCommand {
    fn execute(&self) -> String {
        let mut num = self.receiver.lock().unwrap();
        *num += self.value;
        format!("Added {}", self.value)
    }

    fn undo(&self) -> String {
        let mut num = self.receiver.lock().unwrap();
        *num -= self.value;
        format!("Undid add {}", self.value)
    }
}

/// Problem 13: Repository pattern
/// Design repository pattern
pub trait Repository<T> {
    fn find_by_id(&self, id: u32) -> Option<&T>;
    fn save(&mut self, entity: T) -> T;
    fn delete(&mut self, id: u32) -> bool;
}

/// Problem 14: Service locator
/// Design service locator
pub struct ServiceLocator {
    services: std::collections::HashMap<String, Box<dyn std::any::Any>>,
}

impl ServiceLocator {
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, name: &str, service: T) {
        self.services.insert(name.to_string(), Box::new(service));
    }

    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.services.get(name)?.downcast_ref::<T>()
    }
}

/// Problem 15: Event sourcing
/// Design event sourcing
#[derive(Debug, Clone)]
pub enum Event {
    UserCreated { name: String },
    UserUpdated { name: String },
    UserDeleted,
}

pub struct EventStore {
    events: Vec<Event>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn append(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn get_events(&self) -> &[Event] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test"
        }

        fn execute(&self, input: &str) -> String {
            format!("processed: {}", input)
        }
    }

    #[test]
    fn test_plugin_system() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin));
        let results = manager.execute("input");
        assert_eq!(results, vec!["processed: input"]);
    }

    #[test]
    fn test_middleware() {
        let mut stack = MiddlewareStack::new();
        stack.add(Box::new(|req: &str| format!("prefix_{}", req)));
        assert_eq!(stack.execute("test"), "prefix_test");
    }

    #[test]
    fn test_hook_system() {
        let mut hooks = HookSystem::new();
        hooks.register("test", Box::new(|_| {}));
        hooks.trigger("test", "data");
    }

    #[test]
    fn test_strategy() {
        let context = Context::new(Box::new(SumStrategy));
        assert_eq!(context.execute(&[1, 2, 3]), 6);
    }

    #[test]
    fn test_string_ext() {
        assert!("racecar".is_palindrome());
        assert!(!"hello".is_palindrome());
        assert_eq!("hello world".word_count(), 2);
    }

    #[test]
    fn test_container() {
        let mut container = Container::new();
        container.add(1);
        container.add(2);
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_builder() {
        let request = RequestBuilder::new()
            .url("https://example.com")
            .method("POST")
            .build()
            .unwrap();
        assert_eq!(request.url, "https://example.com");
    }

    #[test]
    fn test_adapter() {
        let old = OldImpl;
        let adapter = Adapter::new(Box::new(old));
        assert_eq!(adapter.new_method(), "old");
    }

    #[test]
    fn test_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        let command = AddCommand::new(5, receiver.clone());
        command.execute();
        assert_eq!(*receiver.lock().unwrap(), 5);
    }

    #[test]
    fn test_service_locator() {
        let mut locator = ServiceLocator::new();
        locator.register("value", 42);
        assert_eq!(locator.get::<i32>("value"), Some(&42));
    }

    #[test]
    fn test_event_sourcing() {
        let mut store = EventStore::new();
        store.append(Event::UserCreated { name: "Alice".to_string() });
        assert_eq!(store.get_events().len(), 1);
    }
}
