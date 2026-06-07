//! # Actor Model for Concurrent Systems
//!
//! The actor model treats concurrent entities (actors) as isolated units that
//! communicate exclusively through messages. Each actor has its own mailbox and
//! processes messages sequentially, eliminating shared-state concurrency issues.
//!
//! ## Key Concepts:
//!
//! - **Actor**: An entity with state, behavior, and a mailbox
//! - **Message**: Typed data sent between actors
//! - **Mailbox**: Queue of incoming messages for an actor
//! - **Supervision**: Hierarchical error handling (parent monitors children)
//! - **Lifecycle**: Actors can be started, stopped, and restarted
//!
//! ## Actor System Design:
//!
//! ```text
//! Supervisor
//!   ├── WorkerActor1
//!   ├── WorkerActor2
//!   └── CoordinatorActor
//!         ├── SubWorker1
//!         └── SubWorker2
//! ```

use crossbeam::channel::{self, Receiver, Sender};
use std::collections::HashMap;
use std::thread;

/// Unique identifier for an actor in the system.
pub type ActorId = u64;

/// Messages that can be sent to any actor.
#[derive(Debug, Clone)]
pub enum ActorMessage {
    /// Regular application message
    Custom(String),
    /// Request to stop the actor
    Stop,
    /// Health check ping
    Ping,
    /// Response to a ping
    Pong,
}

/// Result of processing a message.
#[derive(Debug)]
pub enum ActorAction {
    /// Continue processing messages
    Continue,
    /// Stop the actor
    Stop,
    /// Restart the actor with new state
    Restart,
}

/// Trait that all actors must implement.
pub trait Actor: Send + 'static {
    /// Called when the actor starts.
    fn on_start(&mut self) {}

    /// Process an incoming message.
    fn handle_message(&mut self, message: ActorMessage) -> ActorAction;

    /// Called when the actor is stopping.
    fn on_stop(&mut self) {}

    /// Called when the actor is restarting (after failure).
    fn on_restart(&mut self) {}
}

/// Handle to communicate with a running actor.
pub struct ActorHandle {
    pub id: ActorId,
    sender: Sender<ActorMessage>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ActorHandle {
    /// Send a message to the actor.
    pub fn tell(&self, message: ActorMessage) -> Result<(), String> {
        self.sender
            .send(message)
            .map_err(|_| "Actor mailbox closed".to_string())
    }

    /// Send a message and wait for a response (simplified).
    pub fn ask(&self, message: ActorMessage) -> Result<ActorMessage, String> {
        // In a real system, this would use a response channel
        self.tell(message)?;
        // Simplified: just return Pong
        Ok(ActorMessage::Pong)
    }

    /// Stop the actor.
    pub fn stop(&self) -> Result<(), String> {
        self.tell(ActorMessage::Stop)
    }
}

impl Drop for ActorHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(ActorMessage::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Actor system that manages actor lifecycle and message routing.
pub struct ActorSystem {
    next_id: ActorId,
    actors: HashMap<ActorId, ActorHandle>,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            actors: HashMap::new(),
        }
    }

    /// Spawn a new actor in the system.
    pub fn spawn<A: Actor>(&mut self, mut actor: A) -> ActorId {
        let id = self.next_id;
        self.next_id += 1;

        let (tx, rx) = channel::unbounded();
        actor.on_start();

        let handle = thread::Builder::new()
            .name(format!("actor-{}", id))
            .spawn(move || {
                Self::run_actor_loop(&mut actor, rx);
            })
            .expect("Failed to spawn actor");

        self.actors.insert(
            id,
            ActorHandle {
                id,
                sender: tx,
                handle: Some(handle),
            },
        );

        id
    }

    fn run_actor_loop<A: Actor>(actor: &mut A, rx: Receiver<ActorMessage>) {
        for message in rx.iter() {
            match message {
                ActorMessage::Stop => {
                    actor.on_stop();
                    break;
                }
                ActorMessage::Ping => {
                    // Pong is handled by the ask pattern
                    let _ = actor.handle_message(ActorMessage::Ping);
                }
                msg => {
                    match actor.handle_message(msg) {
                        ActorAction::Continue => {}
                        ActorAction::Stop => {
                            actor.on_stop();
                            break;
                        }
                        ActorAction::Restart => {
                            actor.on_restart();
                        }
                    }
                }
            }
        }
    }

    /// Get a handle to an actor by ID.
    pub fn get_handle(&self, id: ActorId) -> Option<&ActorHandle> {
        self.actors.get(&id)
    }

    /// Stop all actors in the system.
    pub fn shutdown(&mut self) {
        for (_, mut actor) in self.actors.drain() {
            let _ = actor.stop();
            if let Some(handle) = actor.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ActorSystem {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Supervisor strategy for handling actor failures.
#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    /// Stop the failed actor
    Stop,
    /// Restart the failed actor
    Restart,
    /// Escalate the failure to the parent
    Escalate,
    /// Resume processing (ignore the failure)
    Resume,
}

/// Supervised actor with error handling.
pub struct Supervisor<A: Actor> {
    strategy: SupervisionStrategy,
    max_restarts: u32,
    restart_count: u32,
    child_factory: Box<dyn Fn() -> A + Send>,
    children: Vec<ActorId>,
}

impl<A: Actor> Supervisor<A> {
    pub fn new<F>(strategy: SupervisionStrategy, max_restarts: u32, factory: F) -> Self
    where
        F: Fn() -> A + Send + 'static,
    {
        Self {
            strategy,
            max_restarts,
            restart_count: 0,
            child_factory: Box::new(factory),
            children: Vec::new(),
        }
    }

    pub fn spawn_child(&mut self, system: &mut ActorSystem) -> ActorId {
        let actor = (self.child_factory)();
        let id = system.spawn(actor);
        self.children.push(id);
        id
    }

    pub fn handle_failure(&mut self, system: &mut ActorSystem, failed_id: ActorId) {
        self.children.retain(|&id| id != failed_id);

        match self.strategy.clone() {
            SupervisionStrategy::Stop => {
                // Don't restart
            }
            SupervisionStrategy::Restart => {
                if self.restart_count < self.max_restarts {
                    self.restart_count += 1;
                    self.spawn_child(system);
                }
            }
            SupervisionStrategy::Escalate => {
                // In a real system, this would escalate to the parent supervisor
            }
            SupervisionStrategy::Resume => {
                // Re-add the actor
                self.spawn_child(system);
            }
        }
    }
}

/// Example: Worker actor that processes tasks.
pub struct WorkerActor {
    name: String,
    tasks_processed: u64,
    output: Sender<String>,
}

impl WorkerActor {
    pub fn new(name: &str, output: Sender<String>) -> Self {
        Self {
            name: name.into(),
            tasks_processed: 0,
            output,
        }
    }
}

impl Actor for WorkerActor {
    fn on_start(&mut self) {
        let _ = self.output.send(format!("[{}] Started", self.name));
    }

    fn handle_message(&mut self, message: ActorMessage) -> ActorAction {
        match message {
            ActorMessage::Custom(task) => {
                self.tasks_processed += 1;
                let _ = self.output.send(format!(
                    "[{}] Processed task '{}' (total: {})",
                    self.name, task, self.tasks_processed
                ));
                ActorAction::Continue
            }
            ActorMessage::Ping => {
                let _ = self
                    .output
                    .send(format!("[{}] Pong (processed: {})", self.name, self.tasks_processed));
                ActorAction::Continue
            }
            ActorMessage::Stop => ActorAction::Stop,
            _ => ActorAction::Continue,
        }
    }

    fn on_stop(&mut self) {
        let _ = self.output.send(format!(
            "[{}] Stopping after {} tasks",
            self.name, self.tasks_processed
        ));
    }
}

/// Router actor that distributes work across worker actors.
pub struct RouterActor {
    workers: Vec<ActorId>,
    current: usize,
    output: Sender<String>,
}

impl RouterActor {
    pub fn new(workers: Vec<ActorId>, output: Sender<String>) -> Self {
        Self {
            workers,
            current: 0,
            output,
        }
    }
}

impl Actor for RouterActor {
    fn handle_message(&mut self, message: ActorMessage) -> ActorAction {
        match message {
            ActorMessage::Custom(task) => {
                if !self.workers.is_empty() {
                    let worker_id = self.workers[self.current % self.workers.len()];
                    self.current += 1;
                    let _ = self.output.send(format!(
                        "Routing task '{}' to worker {}",
                        task, worker_id
                    ));
                }
                ActorAction::Continue
            }
            ActorMessage::Stop => ActorAction::Stop,
            _ => ActorAction::Continue,
        }
    }
}

/// Pub-sub actor that broadcasts messages to subscribers.
pub struct PubSubActor {
    subscribers: HashMap<String, Vec<Sender<String>>>,
}

impl PubSubActor {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, topic: &str) -> Receiver<String> {
        let (tx, rx) = channel::unbounded();
        self.subscribers
            .entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(tx);
        rx
    }
}

impl Actor for PubSubActor {
    fn handle_message(&mut self, message: ActorMessage) -> ActorAction {
        // In a real system, messages would include topic information
        ActorAction::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;
    use std::time::Duration;

    #[test]
    fn test_worker_actor_processes_messages() {
        let (tx, rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let worker = WorkerActor::new("test-worker", tx);
        let id = system.spawn(worker);

        system
            .get_handle(id)
            .unwrap()
            .tell(ActorMessage::Custom("task1".into()))
            .unwrap();
        system
            .get_handle(id)
            .unwrap()
            .tell(ActorMessage::Custom("task2".into()))
            .unwrap();

        thread::sleep(Duration::from_millis(50));

        let messages: Vec<String> = rx.try_iter().collect();
        assert!(messages.iter().any(|m| m.contains("task1")));
        assert!(messages.iter().any(|m| m.contains("task2")));
        assert!(messages.iter().any(|m| m.contains("Started")));
    }

    #[test]
    fn test_worker_actor_ping_pong() {
        let (tx, rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let worker = WorkerActor::new("ping-test", tx);
        let id = system.spawn(worker);

        system
            .get_handle(id)
            .unwrap()
            .tell(ActorMessage::Ping)
            .unwrap();

        thread::sleep(Duration::from_millis(50));

        let messages: Vec<String> = rx.try_iter().collect();
        assert!(messages.iter().any(|m| m.contains("Pong")));
    }

    #[test]
    fn test_actor_system_spawn_and_stop() {
        let (tx, _rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let worker = WorkerActor::new("lifecycle", tx);
        let id = system.spawn(worker);

        assert!(system.get_handle(id).is_some());
        system.get_handle(id).unwrap().stop().unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn test_actor_system_shutdown() {
        let (tx1, _rx1) = channel::unbounded();
        let (tx2, _rx2) = channel::unbounded();
        let mut system = ActorSystem::new();

        system.spawn(WorkerActor::new("w1", tx1));
        system.spawn(WorkerActor::new("w2", tx2));

        system.shutdown();
        // Should complete without hanging
    }

    #[test]
    fn test_multiple_actors_communication() {
        let (tx, rx) = channel::unbounded();
        let mut system = ActorSystem::new();

        let mut ids = Vec::new();
        for i in 0..5 {
            let tx = tx.clone();
            ids.push(system.spawn(WorkerActor::new(&format!("worker-{}", i), tx)));
        }

        for id in &ids {
            system
                .get_handle(*id)
                .unwrap()
                .tell(ActorMessage::Custom(format!("task-for-{}", id)))
                .unwrap();
        }

        thread::sleep(Duration::from_millis(100));

        let messages: Vec<String> = rx.try_iter().collect();
        assert!(messages.len() >= 5); // At least 5 task messages + starts
    }

    #[test]
    fn test_supervisor_restart_strategy() {
        let (tx, _rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let mut supervisor = Supervisor::new(
            SupervisionStrategy::Restart,
            3,
            move || WorkerActor::new("supervised", tx.clone()),
        );

        let id = supervisor.spawn_child(&mut system);
        assert!(system.get_handle(id).is_some());

        // Simulate failure and restart
        supervisor.handle_failure(&mut system, id);
        assert_eq!(supervisor.children.len(), 1);
    }

    #[test]
    fn test_supervisor_stop_strategy() {
        let (tx, _rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let mut supervisor = Supervisor::new(
            SupervisionStrategy::Stop,
            3,
            move || WorkerActor::new("supervised", tx.clone()),
        );

        supervisor.spawn_child(&mut system);
        supervisor.handle_failure(&mut system, 999); // Non-existent ID
        // Stop strategy: don't restart
    }

    #[test]
    fn test_router_actor() {
        let (tx, rx) = channel::unbounded();
        let mut system = ActorSystem::new();

        let router = RouterActor::new(vec![1, 2, 3], tx);
        let id = system.spawn(router);

        system
            .get_handle(id)
            .unwrap()
            .tell(ActorMessage::Custom("work-item".into()))
            .unwrap();

        thread::sleep(Duration::from_millis(50));

        let messages: Vec<String> = rx.try_iter().collect();
        assert!(messages.iter().any(|m| m.contains("Routing")));
    }

    #[test]
    fn test_actor_handle_tell() {
        let (tx, _rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let id = system.spawn(WorkerActor::new("handle-test", tx));

        let handle = system.get_handle(id).unwrap();
        assert!(handle.tell(ActorMessage::Custom("msg".into())).is_ok());
    }

    #[test]
    fn test_actor_handle_stop() {
        let (tx, rx) = channel::unbounded();
        let mut system = ActorSystem::new();
        let id = system.spawn(WorkerActor::new("stop-test", tx));

        system.get_handle(id).unwrap().stop().unwrap();
        thread::sleep(Duration::from_millis(50));

        let messages: Vec<String> = rx.try_iter().collect();
        assert!(messages.iter().any(|m| m.contains("Stopping")));
    }
}
