//! # Observer Pattern
//!
//! The observer pattern defines a one-to-many dependency between objects so that
//! when one object changes state, all dependents are notified. In Rust, this is
//! implemented with callback closures, trait objects, or event channels.
//!
//! ## Key Concepts
//! - **Event system**: Central event bus that dispatches events
//! - **Subscriber lists**: Maintaining weak references to avoid reference cycles
//! - **Callbacks**: Closure-based event handlers
//! - **Type-safe events**: Using enums for event types

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

/// An event emitter that manages subscribers and dispatches events.
pub struct EventEmitter<E: Clone> {
    subscribers: Vec<Subscriber<E>>,
    next_id: u64,
}

struct Subscriber<E> {
    id: u64,
    handler: Box<dyn Fn(&E) + Send + Sync>,
}

impl<E: Clone> EventEmitter<E> {
    pub fn new() -> Self {
        EventEmitter {
            subscribers: Vec::new(),
            next_id: 1,
        }
    }

    /// Registers a callback and returns a subscription ID for unsubscribing.
    pub fn subscribe(&mut self, handler: impl Fn(&E) + Send + Sync + 'static) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.subscribers.push(Subscriber {
            id,
            handler: Box::new(handler),
        });
        id
    }

    /// Removes a subscriber by ID.
    pub fn unsubscribe(&mut self, id: u64) {
        self.subscribers.retain(|s| s.id != id);
    }

    /// Dispatches an event to all subscribers.
    pub fn emit(&self, event: &E) {
        for subscriber in &self.subscribers {
            (subscriber.handler)(event);
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

/// An event system with typed events and topic-based routing.
#[derive(Debug, Clone)]
pub enum AppEvent {
    UserLogin { user_id: u64 },
    UserLogout { user_id: u64 },
    PageView { url: String },
    Error { message: String },
}

pub struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn Fn(&AppEvent) + Send + Sync>>>,
    next_id: u64,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            handlers: HashMap::new(),
            next_id: 1,
        }
    }

    /// Subscribes to events matching a topic prefix.
    pub fn on(&mut self, topic: &str, handler: impl Fn(&AppEvent) + Send + Sync + 'static) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers
            .entry(topic.to_string())
            .or_default()
            .push(Box::new(handler));
        id
    }

    /// Publishes an event to all matching subscribers.
    pub fn emit(&self, event: &AppEvent) {
        let topic = match event {
            AppEvent::UserLogin { .. } => "user.login",
            AppEvent::UserLogout { .. } => "user.logout",
            AppEvent::PageView { .. } => "page.view",
            AppEvent::Error { .. } => "error",
        };

        // Exact match
        if let Some(handlers) = self.handlers.get(topic) {
            for handler in handlers {
                handler(event);
            }
        }

        // Wildcard subscribers
        if let Some(handlers) = self.handlers.get("*") {
            for handler in handlers {
                handler(event);
            }
        }
    }
}

/// A property observer that notifies when a value changes.
pub struct Observable<T: Clone> {
    value: T,
    observers: Vec<Box<dyn Fn(&T, &T) + Send + Sync>>,
}

impl<T: Clone + PartialEq> Observable<T> {
    pub fn new(initial: T) -> Self {
        Observable {
            value: initial,
            observers: Vec::new(),
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    /// Sets the value and notifies observers if it changed.
    pub fn set(&mut self, new_value: T) {
        if self.value != new_value {
            let old = self.value.clone();
            self.value = new_value;
            for observer in &self.observers {
                observer(&old, &self.value);
            }
        }
    }

    pub fn observe(&mut self, handler: impl Fn(&T, &T) + Send + Sync + 'static) {
        self.observers.push(Box::new(handler));
    }
}

/// A weak-reference-based observer to prevent reference cycles.
pub trait Observer<T>: Send + Sync {
    fn on_change(&self, old: &T, new: &T);
}

pub struct Observed<T> {
    value: T,
    observers: Vec<Weak<dyn Observer<T>>>,
}

impl<T: Clone + PartialEq> Observed<T> {
    pub fn new(initial: T) -> Self {
        Observed {
            value: initial,
            observers: Vec::new(),
        }
    }

    pub fn add_observer(&mut self, observer: Weak<dyn Observer<T>>) {
        self.observers.push(observer);
    }

    pub fn set(&mut self, new_value: T) {
        if self.value != new_value {
            let old = self.value.clone();
            self.value = new_value.clone();

            // Clean up dead observers and notify live ones
            self.observers.retain(|weak| {
                if let Some(observer) = weak.upgrade() {
                    observer.on_change(&old, &self.value);
                    true
                } else {
                    false // Observer was dropped
                }
            });
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

/// A change tracker that records all value changes.
pub struct ChangeTracker<T: Clone> {
    history: Arc<Mutex<Vec<(T, T)>>>,
}

impl<T: Clone + Send + Sync + 'static> ChangeTracker<T> {
    pub fn new() -> Self {
        ChangeTracker {
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn observer(&self) -> impl Fn(&T, &T) + Send + Sync {
        let history = self.history.clone();
        move |old: &T, new: &T| {
            history.lock().unwrap().push((old.clone(), new.clone()));
        }
    }

    pub fn history(&self) -> Vec<(T, T)> {
        self.history.lock().unwrap().clone()
    }

    pub fn change_count(&self) -> usize {
        self.history.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emitter_basic() {
        let mut emitter = EventEmitter::<i32>::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let r = received.clone();

        emitter.subscribe(move |event: &i32| {
            r.lock().unwrap().push(*event);
        });

        emitter.emit(&1);
        emitter.emit(&2);
        emitter.emit(&3);

        assert_eq!(*received.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_event_emitter_unsubscribe() {
        let mut emitter = EventEmitter::<i32>::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let r = received.clone();

        let id = emitter.subscribe(move |event: &i32| {
            r.lock().unwrap().push(*event);
        });

        emitter.emit(&1);
        emitter.unsubscribe(id);
        emitter.emit(&2);

        assert_eq!(*received.lock().unwrap(), vec![1]);
    }

    #[test]
    fn test_event_emitter_multiple_subscribers() {
        let mut emitter = EventEmitter::<String>::new();

        let r1 = Arc::new(Mutex::new(Vec::new()));
        let r1c = r1.clone();
        emitter.subscribe(move |e: &String| r1c.lock().unwrap().push(e.clone()));

        let r2 = Arc::new(Mutex::new(Vec::new()));
        let r2c = r2.clone();
        emitter.subscribe(move |e: &String| r2c.lock().unwrap().push(e.clone()));

        emitter.emit(&"test".to_string());

        assert_eq!(*r1.lock().unwrap(), vec!["test"]);
        assert_eq!(*r2.lock().unwrap(), vec!["test"]);
    }

    #[test]
    fn test_event_bus() {
        let mut bus = EventBus::new();
        let logins = Arc::new(Mutex::new(Vec::new()));
        let l = logins.clone();

        bus.on("user.login", move |event| {
            if let AppEvent::UserLogin { user_id } = event {
                l.lock().unwrap().push(*user_id);
            }
        });

        bus.emit(&AppEvent::UserLogin { user_id: 1 });
        bus.emit(&AppEvent::PageView {
            url: "/home".into(),
        });
        bus.emit(&AppEvent::UserLogin { user_id: 2 });

        assert_eq!(*logins.lock().unwrap(), vec![1, 2]);
    }

    #[test]
    fn test_event_bus_wildcard() {
        let mut bus = EventBus::new();
        let all = Arc::new(Mutex::new(0u32));
        let count = all.clone();

        bus.on("*", move |_| {
            *count.lock().unwrap() += 1;
        });

        bus.emit(&AppEvent::UserLogin { user_id: 1 });
        bus.emit(&AppEvent::Error {
            message: "oops".into(),
        });

        assert_eq!(*all.lock().unwrap(), 2);
    }

    #[test]
    fn test_observable() {
        let mut obs = Observable::new(0);

        let changes = Arc::new(Mutex::new(Vec::new()));
        let c = changes.clone();
        obs.observe(move |old, new| {
            c.lock().unwrap().push((*old, *new));
        });

        obs.set(10);
        obs.set(20);
        obs.set(20); // No change, no notification

        let entries = changes.lock().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], (0, 10));
        assert_eq!(entries[1], (10, 20));
    }

    #[test]
    fn test_change_tracker() {
        let tracker = ChangeTracker::<i32>::new();
        let mut obs = Observable::new(0);

        obs.observe(tracker.observer());

        obs.set(1);
        obs.set(2);
        obs.set(3);

        assert_eq!(tracker.change_count(), 3);
        let history = tracker.history();
        assert_eq!(history[0], (0, 1));
        assert_eq!(history[1], (1, 2));
        assert_eq!(history[2], (2, 3));
    }
}
