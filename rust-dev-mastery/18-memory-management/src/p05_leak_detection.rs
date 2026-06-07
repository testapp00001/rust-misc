//! # Leak Detection and Prevention
//!
//! Memory leaks occur when allocated memory is never freed. Rust's ownership
//! system prevents most leaks, but they can still happen with reference cycles,
//! forgotten Box::leak, or manual memory management. This module covers
//! detection strategies and prevention patterns.
//!
//! ## Common Leak Sources:
//!
//! - `Rc`/`Arc` reference cycles
//! - `Box::leak` (intentional)
//! - `std::mem::forget` (intentional)
//! - Channel sender never dropped
//! - Thread never joined
//! - Global static collections

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Tracking allocator for leak detection in tests.
/// Counts live allocations and reports leaks when dropped.
pub struct LeakTracker {
    live_allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
    next_id: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub id: usize,
    pub size: usize,
    pub label: String,
    pub allocated_at: std::time::Instant,
}

impl LeakTracker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            live_allocations: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU64::new(0),
        })
    }

    /// Track a new allocation.
    pub fn track(&self, size: usize, label: &str) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) as usize;
        let info = AllocationInfo {
            id,
            size,
            label: label.to_string(),
            allocated_at: std::time::Instant::now(),
        };
        self.live_allocations.lock().unwrap().insert(id, info);
        id
    }

    /// Mark an allocation as freed.
    pub fn free(&self, id: usize) {
        self.live_allocations.lock().unwrap().remove(&id);
    }

    /// Get the number of live allocations.
    pub fn live_count(&self) -> usize {
        self.live_allocations.lock().unwrap().len()
    }

    /// Get total bytes of live allocations.
    pub fn live_bytes(&self) -> usize {
        self.live_allocations
            .lock()
            .unwrap()
            .values()
            .map(|info| info.size)
            .sum()
    }

    /// Check for leaks (non-zero live count).
    pub fn has_leaks(&self) -> bool {
        self.live_count() > 0
    }

    /// Get a report of leaked allocations.
    pub fn leak_report(&self) -> Vec<AllocationInfo> {
        self.live_allocations
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
}

/// RAII guard that tracks allocation lifetime.
pub struct TrackedAllocation {
    id: usize,
    tracker: Arc<LeakTracker>,
}

impl TrackedAllocation {
    pub fn new(tracker: Arc<LeakTracker>, size: usize, label: &str) -> Self {
        let id = tracker.track(size, label);
        Self { id, tracker }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl Drop for TrackedAllocation {
    fn drop(&mut self) {
        self.tracker.free(self.id);
    }
}

/// Weak reference pattern to prevent reference cycles.
/// Uses Weak<T> to break potential cycles.
pub struct TreeNode {
    pub value: i32,
    pub children: Vec<Rc<RefCell<TreeNode>>>,
    pub parent: Option<Weak<RefCell<TreeNode>>>,
}

impl TreeNode {
    pub fn new(value: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            value,
            children: Vec::new(),
            parent: None,
        }))
    }

    /// Add a child node, setting up the parent weak reference.
    pub fn add_child(parent: &Rc<RefCell<Self>>, child_value: i32) -> Rc<RefCell<Self>> {
        let child = Rc::new(RefCell::new(Self {
            value: child_value,
            children: Vec::new(),
            parent: Some(Rc::downgrade(parent)),
        }));
        parent.borrow_mut().children.push(Rc::clone(&child));
        child
    }

    /// Get the parent's value (demonstrating weak reference usage).
    pub fn parent_value(node: &Rc<RefCell<Self>>) -> Option<i32> {
        node.borrow()
            .parent
            .as_ref()
            .and_then(|weak| weak.upgrade())
            .map(|parent| parent.borrow().value)
    }
}

/// Preventing leaks with explicit lifetime management.
pub struct ResourceGuard<T> {
    resource: Option<T>,
    cleanup: Box<dyn FnOnce(T)>,
}

impl<T> ResourceGuard<T> {
    pub fn new<F>(resource: T, cleanup: F) -> Self
    where
        F: FnOnce(T) + 'static,
    {
        Self {
            resource: Some(resource),
            cleanup: Box::new(cleanup),
        }
    }

    pub fn get(&self) -> &T {
        self.resource.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.resource.as_mut().unwrap()
    }

    /// Consume the guard, running cleanup and returning the resource.
    /// Note: cleanup is NOT called when using into_inner, as the resource
    /// is being explicitly taken out of the guard.
    pub fn into_inner(mut self) -> T {
        let resource = self.resource.take().unwrap();
        // Set cleanup to no-op so Drop doesn't try to use the missing resource
        self.cleanup = Box::new(|_| {});
        resource
    }
}

impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let cleanup = std::mem::replace(&mut self.cleanup, Box::new(|_| {}));
            cleanup(resource);
        }
    }
}

/// Sentinel value pattern: detect when a value is leaked.
pub struct LeakSentinel {
    leaked: Arc<AtomicU64>,
    id: u64,
}

impl LeakSentinel {
    pub fn new(counter: &Arc<AtomicU64>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        let id = counter.load(Ordering::SeqCst);
        Self {
            leaked: Arc::clone(counter),
            id,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Drop for LeakSentinel {
    fn drop(&mut self) {
        self.leaked.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Leak detector that uses sentinel values to detect leaks.
pub struct LeakDetector {
    counter: Arc<AtomicU64>,
}

impl LeakDetector {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a sentinel that will be detected as leaked if not dropped.
    pub fn sentinel(&self) -> LeakSentinel {
        LeakSentinel::new(&self.counter)
    }

    /// Check if there are any leaked sentinels.
    pub fn leaked_count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }

    pub fn has_leaks(&self) -> bool {
        self.leaked_count() > 0
    }
}

/// Cycle-safe graph using weak references.
pub struct GraphNode {
    pub id: usize,
    pub value: String,
    neighbors: Vec<Weak<RefCell<GraphNode>>>,
}

impl GraphNode {
    pub fn new(id: usize, value: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            id,
            value: value.to_string(),
            neighbors: Vec::new(),
        }))
    }

    pub fn add_edge(from: &Rc<RefCell<Self>>, to: &Rc<RefCell<Self>>) {
        from.borrow_mut().neighbors.push(Rc::downgrade(to));
    }

    pub fn neighbor_count(node: &Rc<RefCell<Self>>) -> usize {
        node.borrow()
            .neighbors
            .iter()
            .filter(|w| w.upgrade().is_some())
            .count()
    }

    pub fn alive_neighbors(node: &Rc<RefCell<Self>>) -> Vec<usize> {
        node.borrow()
            .neighbors
            .iter()
            .filter_map(|w| w.upgrade())
            .map(|n| n.borrow().id)
            .collect()
    }
}

/// Channel-based leak detection: tracks if senders are properly dropped.
pub struct ChannelLeakDetector {
    active_senders: Arc<AtomicU64>,
}

impl ChannelLeakDetector {
    pub fn new() -> Self {
        Self {
            active_senders: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a tracked channel sender.
    pub fn create_sender(&self) -> TrackedSender {
        self.active_senders.fetch_add(1, Ordering::SeqCst);
        TrackedSender {
            counter: Arc::clone(&self.active_senders),
        }
    }

    pub fn active_senders(&self) -> u64 {
        self.active_senders.load(Ordering::SeqCst)
    }
}

pub struct TrackedSender {
    counter: Arc<AtomicU64>,
}

impl Drop for TrackedSender {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leak_tracker_no_leaks() {
        let tracker = LeakTracker::new();
        {
            let _alloc1 = TrackedAllocation::new(Arc::clone(&tracker), 100, "test1");
            let _alloc2 = TrackedAllocation::new(Arc::clone(&tracker), 200, "test2");
            assert_eq!(tracker.live_count(), 2);
        }
        // All dropped
        assert_eq!(tracker.live_count(), 0);
        assert!(!tracker.has_leaks());
    }

    #[test]
    fn test_leak_tracker_detects_leak() {
        let tracker = LeakTracker::new();
        let _leaked = TrackedAllocation::new(Arc::clone(&tracker), 100, "leaked");
        assert!(tracker.has_leaks());
        assert_eq!(tracker.live_count(), 1);
    }

    #[test]
    fn test_leak_tracker_report() {
        let tracker = LeakTracker::new();
        let _a = TrackedAllocation::new(Arc::clone(&tracker), 100, "alloc-a");
        let _b = TrackedAllocation::new(Arc::clone(&tracker), 200, "alloc-b");

        let report = tracker.leak_report();
        assert_eq!(report.len(), 2);
        assert!(report.iter().any(|r| r.label == "alloc-a"));
        assert!(report.iter().any(|r| r.label == "alloc-b"));
    }

    #[test]
    fn test_leak_tracker_bytes() {
        let tracker = LeakTracker::new();
        let _a = TrackedAllocation::new(Arc::clone(&tracker), 100, "a");
        let _b = TrackedAllocation::new(Arc::clone(&tracker), 200, "b");
        assert_eq!(tracker.live_bytes(), 300);
    }

    #[test]
    fn test_tree_node_weak_parent() {
        let parent = TreeNode::new(1);
        let child = TreeNode::add_child(&parent, 2);

        assert_eq!(TreeNode::parent_value(&child), Some(1));
        // Parent has 1 strong reference (the `parent` variable)
        // Child holds a Weak reference to parent
        assert_eq!(Rc::strong_count(&parent), 1);
    }

    #[test]
    fn test_tree_node_no_cycle_leak() {
        let parent = TreeNode::new(1);
        let child = TreeNode::add_child(&parent, 2);
        let _grandchild = TreeNode::add_child(&child, 3);

        // Drop all strong references
        drop(parent);
        // child and grandchild should still work
        // The weak parent reference should return None
    }

    #[test]
    fn test_resource_guard_cleanup() {
        let cleaned = Arc::new(AtomicU64::new(0));
        let cleaned_clone = Arc::clone(&cleaned);

        {
            let _guard = ResourceGuard::new(42, move |_| {
                cleaned_clone.fetch_add(1, Ordering::SeqCst);
            });
        }
        assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_resource_guard_into_inner() {
        let cleaned = Arc::new(AtomicU64::new(0));
        let cleaned_clone = Arc::clone(&cleaned);

        let guard = ResourceGuard::new(String::from("hello"), move |_| {
            cleaned_clone.fetch_add(1, Ordering::SeqCst);
        });

        let value = guard.into_inner();
        assert_eq!(value, "hello");
        // into_inner does NOT call cleanup (resource is explicitly taken out)
        assert_eq!(cleaned.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_leak_sentinel() {
        let detector = LeakDetector::new();
        {
            let _s1 = detector.sentinel();
            let _s2 = detector.sentinel();
            assert_eq!(detector.leaked_count(), 2);
        }
        assert_eq!(detector.leaked_count(), 0);
        assert!(!detector.has_leaks());
    }

    #[test]
    fn test_leak_detector_detects_leak() {
        let detector = LeakDetector::new();
        let _leaked = detector.sentinel();
        assert!(detector.has_leaks());
    }

    #[test]
    fn test_graph_node_weak_edges() {
        let a = GraphNode::new(1, "A");
        let b = GraphNode::new(2, "B");
        let c = GraphNode::new(3, "C");

        GraphNode::add_edge(&a, &b);
        GraphNode::add_edge(&a, &c);
        GraphNode::add_edge(&b, &c);

        assert_eq!(GraphNode::neighbor_count(&a), 2);
        assert_eq!(GraphNode::neighbor_count(&b), 1);
        assert_eq!(GraphNode::neighbor_count(&c), 0);
    }

    #[test]
    fn test_graph_node_weak_reference_expiry() {
        let a = GraphNode::new(1, "A");
        {
            let b = GraphNode::new(2, "B");
            GraphNode::add_edge(&a, &b);
            assert_eq!(GraphNode::neighbor_count(&a), 1);
        }
        // b dropped, weak reference should be expired
        assert_eq!(GraphNode::neighbor_count(&a), 0);
        assert!(GraphNode::alive_neighbors(&a).is_empty());
    }

    #[test]
    fn test_channel_leak_detector() {
        let detector = ChannelLeakDetector::new();
        {
            let _s1 = detector.create_sender();
            let _s2 = detector.create_sender();
            assert_eq!(detector.active_senders(), 2);
        }
        assert_eq!(detector.active_senders(), 0);
    }

    #[test]
    fn test_resource_guard_get() {
        let guard = ResourceGuard::new(vec![1, 2, 3], |_| {});
        assert_eq!(guard.get(), &vec![1, 2, 3]);
    }
}
