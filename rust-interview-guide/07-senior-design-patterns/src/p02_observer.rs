/// Problem: Observer Pattern
///
/// Master the observer pattern in Rust.
///
/// Key Concepts:
/// - Observer trait
/// - Event system
/// - Subscription
/// - Notification
/// - Decoupling

use std::collections::HashMap;

/// Problem 1: Basic observer
/// Create basic observer
pub trait Observer {
    fn update(&self, event: &str);
}

pub struct EventSystem {
    observers: Vec<Box<dyn Observer>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
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

/// Problem 2: Observer with data
/// Pass data to observers
#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub data: String,
}

pub trait DataObserver {
    fn on_event(&self, event: &Event);
}

pub struct DataEventSystem {
    observers: Vec<Box<dyn DataObserver>>,
}

impl DataEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn DataObserver>) {
        self.observers.push(observer);
    }

    pub fn emit(&self, event: &Event) {
        for observer in &self.observers {
            observer.on_event(event);
        }
    }
}

/// Problem 3: Observer with multiple events
/// Handle multiple event types
#[derive(Debug, Clone)]
pub enum EventType {
    Click { x: i32, y: i32 },
    KeyPress { key: char },
    Scroll { delta: i32 },
}

pub trait MultiObserver {
    fn on_click(&self, x: i32, y: i32);
    fn on_key_press(&self, key: char);
    fn on_scroll(&self, delta: i32);
}

pub struct MultiEventSystem {
    observers: Vec<Box<dyn MultiObserver>>,
}

impl MultiEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn MultiObserver>) {
        self.observers.push(observer);
    }

    pub fn emit(&self, event: &EventType) {
        for observer in &self.observers {
            match event {
                EventType::Click { x, y } => observer.on_click(*x, *y),
                EventType::KeyPress { key } => observer.on_key_press(*key),
                EventType::Scroll { delta } => observer.on_scroll(*delta),
            }
        }
    }
}

/// Problem 4: Observer with subscription management
/// Manage subscriptions
pub struct SubscriptionManager {
    observers: HashMap<String, Vec<Box<dyn Observer>>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            observers: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, event: &str, observer: Box<dyn Observer>) {
        self.observers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(observer);
    }

    pub fn notify(&self, event: &str) {
        if let Some(observers) = self.observers.get(event) {
            for observer in observers {
                observer.update(event);
            }
        }
    }
}

/// Problem 5: Observer with weak references
/// Use weak references to prevent memory leaks
pub trait WeakObserver {
    fn update(&self, event: &str);
}

pub struct WeakEventSystem {
    observers: Vec<Box<dyn WeakObserver>>,
}

impl WeakEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn WeakObserver>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, event: &str) {
        for observer in &self.observers {
            observer.update(event);
        }
    }
}

/// Problem 6: Observer with priority
/// Prioritize observers
pub struct PriorityObserver {
    pub priority: u32,
    pub observer: Box<dyn Observer>,
}

pub struct PriorityEventSystem {
    observers: Vec<PriorityObserver>,
}

impl PriorityEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, priority: u32, observer: Box<dyn Observer>) {
        self.observers.push(PriorityObserver { priority, observer });
        self.observers.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn notify(&self, event: &str) {
        for observer in &self.observers {
            observer.observer.update(event);
        }
    }
}

/// Problem 7: Observer with filtering
/// Filter events before notifying
pub trait FilterObserver {
    fn should_notify(&self, event: &str) -> bool;
    fn on_event(&self, event: &str);
}

pub struct FilteredEventSystem {
    observers: Vec<Box<dyn FilterObserver>>,
}

impl FilteredEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn FilterObserver>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, event: &str) {
        for observer in &self.observers {
            if observer.should_notify(event) {
                observer.on_event(event);
            }
        }
    }
}

/// Problem 8: Observer with async
/// Async observer (simulated)
pub trait AsyncObserver {
    fn update(&self, event: &str);
}

pub struct AsyncEventSystem {
    observers: Vec<Box<dyn AsyncObserver>>,
}

impl AsyncEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn AsyncObserver>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, event: &str) {
        for observer in &self.observers {
            observer.update(event);
        }
    }
}

/// Problem 9: Observer with history
/// Keep event history
pub struct HistoryEventSystem {
    observers: Vec<Box<dyn Observer>>,
    history: Vec<String>,
}

impl HistoryEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    pub fn notify(&mut self, event: &str) {
        self.history.push(event.to_string());
        for observer in &self.observers {
            observer.update(event);
        }
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }
}

/// Problem 10: Observer with unsubscribe
/// Support unsubscribe
pub struct UnsubscribeEventSystem {
    observers: HashMap<usize, Box<dyn Observer>>,
    next_id: usize,
}

impl UnsubscribeEventSystem {
    pub fn new() -> Self {
        Self {
            observers: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn Observer>) -> usize {
        let id = self.next_id;
        self.observers.insert(id, observer);
        self.next_id += 1;
        id
    }

    pub fn unsubscribe(&mut self, id: usize) {
        self.observers.remove(&id);
    }

    pub fn notify(&self, event: &str) {
        for observer in self.observers.values() {
            observer.update(event);
        }
    }
}

/// Problem 11: Observer with channel
/// Use channels for observer
pub struct ChannelEventSystem {
    senders: Vec<std::sync::mpsc::Sender<String>>,
}

impl ChannelEventSystem {
    pub fn new() -> Self {
        Self {
            senders: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> std::sync::mpsc::Receiver<String> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.senders.push(tx);
        rx
    }

    pub fn notify(&self, event: &str) {
        for sender in &self.senders {
            let _ = sender.send(event.to_string());
        }
    }
}

/// Problem 12: Observer with mutex
/// Thread-safe observer
pub struct MutexEventSystem {
    observers: std::sync::Mutex<Vec<Box<dyn Observer + Send>>>,
}

impl MutexEventSystem {
    pub fn new() -> Self {
        Self {
            observers: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn subscribe(&self, observer: Box<dyn Observer + Send>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }

    pub fn notify(&self, event: &str) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.update(event);
        }
    }
}

/// Problem 13: Observer with closure
/// Use closures as observers
pub struct ClosureEventSystem {
    observers: Vec<Box<dyn Fn(&str)>>,
}

impl ClosureEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn Fn(&str)>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, event: &str) {
        for observer in &self.observers {
            observer(event);
        }
    }
}

/// Problem 14: Observer with trait object
/// Use trait objects
pub struct TraitObjectEventSystem {
    observers: Vec<Box<dyn Observer>>,
}

impl TraitObjectEventSystem {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
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

/// Problem 15: Observer with generic
/// Generic observer system
pub trait GenericObserver<T> {
    fn update(&self, data: &T);
}

impl<T, F: Fn(&T)> GenericObserver<T> for F {
    fn update(&self, data: &T) {
        self(data);
    }
}

pub struct GenericEventSystem<T> {
    observers: Vec<Box<dyn GenericObserver<T>>>,
}

impl<T> GenericEventSystem<T> {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, observer: Box<dyn GenericObserver<T>>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, data: &T) {
        for observer in &self.observers {
            observer.update(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestObserver {
        events: std::sync::Mutex<Vec<String>>,
    }

    impl TestObserver {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn get_events(&self) -> Vec<String> {
let result =             self.events.lock().unwrap().clone(); result
        }
    }

    impl Observer for TestObserver {
        fn update(&self, event: &str) {
            self.events.lock().unwrap().push(event.to_string());
        }
    }

    impl DataObserver for TestObserver {
        fn on_event(&self, event: &Event) {
            self.events.lock().unwrap().push(event.name.clone());
        }
    }

    impl FilterObserver for TestObserver {
        fn should_notify(&self, _event: &str) -> bool {
            true
        }

        fn on_event(&self, event: &str) {
            self.events.lock().unwrap().push(event.to_string());
        }
    }

    #[test]
    fn test_basic_observer() {
        let mut system = EventSystem::new();
        let observer = TestObserver::new();
        system.subscribe(Box::new(observer));
        system.notify("test");
    }

    #[test]
    fn test_observer_with_data() {
        let mut system = DataEventSystem::new();
        let observer = TestObserver::new();
        system.subscribe(Box::new(observer));
        let event = Event {
            name: "test".to_string(),
            data: "data".to_string(),
        };
        system.emit(&event);
    }

    #[test]
    fn test_subscription_manager() {
        let mut manager = SubscriptionManager::new();
        let observer = TestObserver::new();
        manager.subscribe("click", Box::new(observer));
        manager.notify("click");
    }

    #[test]
    fn test_priority_observer() {
        let mut system = PriorityEventSystem::new();
        let observer = TestObserver::new();
        system.subscribe(1, Box::new(observer));
        system.notify("test");
    }

    #[test]
    fn test_filtered_observer() {
        let mut system = FilteredEventSystem::new();
        let observer = TestObserver::new();
        system.subscribe(Box::new(observer));
        system.notify("test");
    }

    #[test]
    fn test_history_observer() {
        let mut system = HistoryEventSystem::new();
        let observer = TestObserver::new();
        system.subscribe(Box::new(observer));
        system.notify("test");
        assert_eq!(system.get_history(), &["test"]);
    }

    #[test]
    fn test_unsubscribe() {
        let mut system = UnsubscribeEventSystem::new();
        let observer = TestObserver::new();
        let id = system.subscribe(Box::new(observer));
        system.unsubscribe(id);
        system.notify("test");
    }

    #[test]
    fn test_channel_observer() {
        let mut system = ChannelEventSystem::new();
        let rx = system.subscribe();
        system.notify("test");
        assert_eq!(rx.recv().unwrap(), "test");
    }

    #[test]
    fn test_closure_observer() {
        let mut system = ClosureEventSystem::new();
        let events = std::sync::Mutex::new(Vec::new());
        system.subscribe(Box::new(move |event| {
            events.lock().unwrap().push(event.to_string());
        }));
        system.notify("test");
    }

    #[test]
    fn test_generic_observer() {
        let mut system = GenericEventSystem::<i32>::new();
        system.subscribe(Box::new(|data: &i32| {
            assert_eq!(*data, 42);
        }));
        system.notify(&42);
    }
}
