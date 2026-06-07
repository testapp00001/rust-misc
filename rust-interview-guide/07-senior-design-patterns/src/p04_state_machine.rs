/// Problem: State Machine
///
/// Master state machines in Rust.
///
/// Key Concepts:
/// - State enum
/// - Transition functions
/// - State guards
/// - State actions
/// - Typestate pattern

/// Problem 1: Basic state machine
/// Create basic state machine
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct Connection {
    state: ConnectionState,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        match self.state {
            ConnectionState::Disconnected => {
                self.state = ConnectionState::Connecting;
                Ok(())
            }
            _ => Err("Already connected or connecting".to_string()),
        }
    }

    pub fn established(&mut self) -> Result<(), String> {
        match self.state {
            ConnectionState::Connecting => {
                self.state = ConnectionState::Connected;
                Ok(())
            }
            _ => Err("Not connecting".to_string()),
        }
    }

    pub fn disconnect(&mut self) -> Result<(), String> {
        match self.state {
            ConnectionState::Connected => {
                self.state = ConnectionState::Disconnected;
                Ok(())
            }
            _ => Err("Not connected".to_string()),
        }
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }
}

/// Problem 2: State machine with data
/// State machine with associated data
#[derive(Debug, Clone)]
pub enum OrderState {
    Created { items: Vec<String> },
    Processing { items: Vec<String>, assigned_to: String },
    Shipped { tracking_number: String },
    Delivered,
}

pub struct Order {
    state: OrderState,
}

impl Order {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            state: OrderState::Created { items },
        }
    }

    pub fn process(&mut self, assigned_to: &str) -> Result<(), String> {
        match &self.state {
            OrderState::Created { items } => {
                self.state = OrderState::Processing {
                    items: items.clone(),
                    assigned_to: assigned_to.to_string(),
                };
                Ok(())
            }
            _ => Err("Cannot process".to_string()),
        }
    }

    pub fn ship(&mut self, tracking: &str) -> Result<(), String> {
        match &self.state {
            OrderState::Processing { .. } => {
                self.state = OrderState::Shipped {
                    tracking_number: tracking.to_string(),
                };
                Ok(())
            }
            _ => Err("Cannot ship".to_string()),
        }
    }

    pub fn deliver(&mut self) -> Result<(), String> {
        match &self.state {
            OrderState::Shipped { .. } => {
                self.state = OrderState::Delivered;
                Ok(())
            }
            _ => Err("Cannot deliver".to_string()),
        }
    }

    pub fn state(&self) -> &OrderState {
        &self.state
    }
}

/// Problem 3: Typestate pattern
/// Use typestate for compile-time safety
pub struct Locked;
pub struct Unlocked;

pub struct Door<State> {
    _state: std::marker::PhantomData<State>,
}

impl Door<Locked> {
    pub fn new() -> Self {
        Self {
            _state: std::marker::PhantomData,
        }
    }

    pub fn unlock(self) -> Door<Unlocked> {
        Door {
            _state: std::marker::PhantomData,
        }
    }
}

impl Door<Unlocked> {
    pub fn lock(self) -> Door<Locked> {
        Door {
            _state: std::marker::PhantomData,
        }
    }

    pub fn open(&self) -> String {
        "Door is open".to_string()
    }
}

/// Problem 4: State machine with guards
/// Add guards to transitions
#[derive(Debug, Clone, PartialEq)]
pub enum TrafficLight {
    Red,
    Yellow,
    Green,
}

pub struct TrafficController {
    light: TrafficLight,
    timer: u32,
}

impl TrafficController {
    pub fn new() -> Self {
        Self {
            light: TrafficLight::Red,
            timer: 0,
        }
    }

    pub fn tick(&mut self) {
        self.timer += 1;
        match self.light {
            TrafficLight::Red if self.timer >= 60 => {
                self.light = TrafficLight::Green;
                self.timer = 0;
            }
            TrafficLight::Green if self.timer >= 45 => {
                self.light = TrafficLight::Yellow;
                self.timer = 0;
            }
            TrafficLight::Yellow if self.timer >= 10 => {
                self.light = TrafficLight::Red;
                self.timer = 0;
            }
            _ => {}
        }
    }

    pub fn light(&self) -> &TrafficLight {
        &self.light
    }
}

/// Problem 5: State machine with actions
/// Execute actions on transitions
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    Idle,
    Running,
    Jumping,
    Falling,
}

pub struct Player {
    state: PlayerState,
    position: f64,
}

impl Player {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Idle,
            position: 0.0,
        }
    }

    pub fn run(&mut self) {
        match self.state {
            PlayerState::Idle | PlayerState::Running => {
                self.state = PlayerState::Running;
                self.position += 1.0;
            }
            _ => {}
        }
    }

    pub fn jump(&mut self) {
        match self.state {
            PlayerState::Running => {
                self.state = PlayerState::Jumping;
                self.position += 5.0;
            }
            _ => {}
        }
    }

    pub fn fall(&mut self) {
        match self.state {
            PlayerState::Jumping => {
                self.state = PlayerState::Falling;
            }
            _ => {}
        }
    }

    pub fn land(&mut self) {
        match self.state {
            PlayerState::Falling => {
                self.state = PlayerState::Idle;
            }
            _ => {}
        }
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    pub fn position(&self) -> f64 {
        self.position
    }
}

/// Problem 6: State machine with history
/// Track state history
#[derive(Debug, Clone)]
pub struct StateHistory<T: Clone> {
    current: T,
    history: Vec<T>,
}

impl<T: Clone> StateHistory<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
        }
    }

    pub fn transition(&mut self, new_state: T) {
        self.history.push(self.current.clone());
        self.current = new_state;
    }

    pub fn current(&self) -> &T {
        &self.current
    }

    pub fn history(&self) -> &[T] {
        &self.history
    }
}

/// Problem 7: State machine with callbacks
/// Execute callbacks on transitions
pub struct CallbackStateMachine {
    state: String,
    on_enter: Vec<Box<dyn Fn(&str)>>,
    on_exit: Vec<Box<dyn Fn(&str)>>,
}

impl CallbackStateMachine {
    pub fn new(initial: &str) -> Self {
        Self {
            state: initial.to_string(),
            on_enter: Vec::new(),
            on_exit: Vec::new(),
        }
    }

    pub fn on_enter(&mut self, callback: Box<dyn Fn(&str)>) {
        self.on_enter.push(callback);
    }

    pub fn on_exit(&mut self, callback: Box<dyn Fn(&str)>) {
        self.on_exit.push(callback);
    }

    pub fn transition(&mut self, new_state: &str) {
        for callback in &self.on_exit {
            callback(&self.state);
        }
        self.state = new_state.to_string();
        for callback in &self.on_enter {
            callback(&self.state);
        }
    }

    pub fn state(&self) -> &str {
        &self.state
    }
}

/// Problem 8: State machine with validation
/// Validate transitions
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentState {
    Draft,
    Review,
    Approved,
    Published,
}

pub struct Document {
    state: DocumentState,
}

impl Document {
    pub fn new() -> Self {
        Self {
            state: DocumentState::Draft,
        }
    }

    pub fn submit_for_review(&mut self) -> Result<(), String> {
        match self.state {
            DocumentState::Draft => {
                self.state = DocumentState::Review;
                Ok(())
            }
            _ => Err("Can only submit draft for review".to_string()),
        }
    }

    pub fn approve(&mut self) -> Result<(), String> {
        match self.state {
            DocumentState::Review => {
                self.state = DocumentState::Approved;
                Ok(())
            }
            _ => Err("Can only approve in review".to_string()),
        }
    }

    pub fn publish(&mut self) -> Result<(), String> {
        match self.state {
            DocumentState::Approved => {
                self.state = DocumentState::Published;
                Ok(())
            }
            _ => Err("Can only publish approved".to_string()),
        }
    }

    pub fn state(&self) -> &DocumentState {
        &self.state
    }
}

/// Problem 9: State machine with context
/// Carry context through states
#[derive(Debug, Clone)]
pub enum AuthState {
    Anonymous,
    Authenticating { username: String },
    Authenticated { username: String, token: String },
    Error { message: String },
}

pub struct AuthMachine {
    state: AuthState,
}

impl AuthMachine {
    pub fn new() -> Self {
        Self {
            state: AuthState::Anonymous,
        }
    }

    pub fn login(&mut self, username: &str) {
        self.state = AuthState::Authenticating {
            username: username.to_string(),
        };
    }

    pub fn success(&mut self, token: &str) {
        if let AuthState::Authenticating { username } = &self.state {
            self.state = AuthState::Authenticated {
                username: username.clone(),
                token: token.to_string(),
            };
        }
    }

    pub fn fail(&mut self, message: &str) {
        self.state = AuthState::Error {
            message: message.to_string(),
        };
    }

    pub fn state(&self) -> &AuthState {
        &self.state
    }
}

/// Problem 10: State machine with reset
/// Reset to initial state
pub struct ResettableMachine {
    state: String,
    initial: String,
}

impl ResettableMachine {
    pub fn new(initial: &str) -> Self {
        Self {
            state: initial.to_string(),
            initial: initial.to_string(),
        }
    }

    pub fn transition(&mut self, new_state: &str) {
        self.state = new_state.to_string();
    }

    pub fn reset(&mut self) {
        self.state = self.initial.clone();
    }

    pub fn state(&self) -> &str {
        &self.state
    }
}

/// Problem 11: State machine with events
/// Event-driven state machine
#[derive(Debug, Clone)]
pub enum Event {
    Start,
    Stop,
    Pause,
    Resume,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerControlState {
    Stopped,
    Playing,
    Paused,
}

pub struct PlayerControl {
    state: PlayerControlState,
}

impl PlayerControl {
    pub fn new() -> Self {
        Self {
            state: PlayerControlState::Stopped,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match (&self.state, event) {
            (PlayerControlState::Stopped, Event::Start) => {
                self.state = PlayerControlState::Playing;
            }
            (PlayerControlState::Playing, Event::Stop) => {
                self.state = PlayerControlState::Stopped;
            }
            (PlayerControlState::Playing, Event::Pause) => {
                self.state = PlayerControlState::Paused;
            }
            (PlayerControlState::Paused, Event::Resume) => {
                self.state = PlayerControlState::Playing;
            }
            (PlayerControlState::Paused, Event::Stop) => {
                self.state = PlayerControlState::Stopped;
            }
            _ => {}
        }
    }

    pub fn state(&self) -> &PlayerControlState {
        &self.state
    }
}

/// Problem 12: State machine with timeout
/// Handle timeouts
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Active,
    Idle,
    Expired,
}

pub struct Session {
    state: SessionState,
    idle_timer: u32,
    max_idle: u32,
}

impl Session {
    pub fn new(max_idle: u32) -> Self {
        Self {
            state: SessionState::Active,
            idle_timer: 0,
            max_idle,
        }
    }

    pub fn activity(&mut self) {
        self.state = SessionState::Active;
        self.idle_timer = 0;
    }

    pub fn tick(&mut self) {
        match self.state {
            SessionState::Active => {
                self.idle_timer += 1;
                if self.idle_timer >= self.max_idle {
                    self.state = SessionState::Idle;
                }
            }
            SessionState::Idle => {
                self.idle_timer += 1;
                if self.idle_timer >= self.max_idle * 2 {
                    self.state = SessionState::Expired;
                }
            }
            _ => {}
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }
}

/// Problem 13: State machine with concurrent states
/// Track multiple state dimensions
#[derive(Debug, Clone)]
pub struct CharacterState {
    pub health: HealthState,
    pub movement: MovementState,
    pub action: ActionState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthState {
    Alive,
    Dead,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementState {
    Idle,
    Walking,
    Running,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionState {
    None,
    Attacking,
    Defending,
}

pub struct Character {
    state: CharacterState,
}

impl Character {
    pub fn new() -> Self {
        Self {
            state: CharacterState {
                health: HealthState::Alive,
                movement: MovementState::Idle,
                action: ActionState::None,
            },
        }
    }

    pub fn walk(&mut self) {
        if self.state.health == HealthState::Alive {
            self.state.movement = MovementState::Walking;
        }
    }

    pub fn attack(&mut self) {
        if self.state.health == HealthState::Alive {
            self.state.action = ActionState::Attacking;
        }
    }

    pub fn die(&mut self) {
        self.state.health = HealthState::Dead;
        self.state.movement = MovementState::Idle;
        self.state.action = ActionState::None;
    }

    pub fn state(&self) -> &CharacterState {
        &self.state
    }
}

/// Problem 14: State machine with persistence
/// Serialize/deserialize state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PersistableState {
    Initial,
    Processing,
    Complete,
    Failed,
}

pub struct PersistableMachine {
    state: PersistableState,
}

impl PersistableMachine {
    pub fn new() -> Self {
        Self {
            state: PersistableState::Initial,
        }
    }

    pub fn transition(&mut self, new_state: PersistableState) {
        self.state = new_state;
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_default()
    }

    pub fn deserialize(&mut self, json: &str) {
        if let Ok(state) = serde_json::from_str(json) {
            self.state = state;
        }
    }

    pub fn state(&self) -> &PersistableState {
        &self.state
    }
}

/// Problem 15: State machine with builder
/// Build state machines
pub struct StateMachineBuilder {
    states: Vec<String>,
    transitions: Vec<(String, String, String)>,
    initial: Option<String>,
}

impl StateMachineBuilder {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            transitions: Vec::new(),
            initial: None,
        }
    }

    pub fn add_state(mut self, state: &str) -> Self {
        self.states.push(state.to_string());
        self
    }

    pub fn add_transition(mut self, from: &str, to: &str, event: &str) -> Self {
        self.transitions.push((
            from.to_string(),
            to.to_string(),
            event.to_string(),
        ));
        self
    }

    pub fn initial(mut self, state: &str) -> Self {
        self.initial = Some(state.to_string());
        self
    }

    pub fn build(self) -> BuiltStateMachine {
        BuiltStateMachine {
            states: self.states,
            transitions: self.transitions,
            current: self.initial.unwrap_or_default(),
        }
    }
}

pub struct BuiltStateMachine {
    states: Vec<String>,
    transitions: Vec<(String, String, String)>,
    current: String,
}

impl BuiltStateMachine {
    pub fn transition(&mut self, event: &str) -> Result<(), String> {
        for (from, to, evt) in &self.transitions {
            if *from == self.current && *evt == event {
                self.current = to.clone();
                return Ok(());
            }
        }
        Err("Invalid transition".to_string())
    }

    pub fn state(&self) -> &str {
        &self.current
    }
}

use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state_machine() {
        let mut conn = Connection::new();
        assert_eq!(*conn.state(), ConnectionState::Disconnected);
        conn.connect().unwrap();
        assert_eq!(*conn.state(), ConnectionState::Connecting);
        conn.established().unwrap();
        assert_eq!(*conn.state(), ConnectionState::Connected);
    }

    #[test]
    fn test_order_state_machine() {
        let mut order = Order::new(vec!["item1".to_string()]);
        order.process("worker1").unwrap();
        order.ship("TRACK123").unwrap();
        order.deliver().unwrap();
        assert!(matches!(order.state(), OrderState::Delivered));
    }

    #[test]
    fn test_typestate() {
        let door = Door::new();
        let door = door.unlock();
        assert_eq!(door.open(), "Door is open");
        let _door = door.lock();
    }

    #[test]
    fn test_traffic_light() {
        let mut controller = TrafficController::new();
        assert_eq!(*controller.light(), TrafficLight::Red);
        for _ in 0..60 {
            controller.tick();
        }
        assert_eq!(*controller.light(), TrafficLight::Green);
    }

    #[test]
    fn test_player_state() {
        let mut player = Player::new();
        player.run();
        assert_eq!(*player.state(), PlayerState::Running);
        player.jump();
        assert_eq!(*player.state(), PlayerState::Jumping);
    }

    #[test]
    fn test_state_history() {
        let mut history = StateHistory::new("initial");
        history.transition("state1");
        history.transition("state2");
        assert_eq!(history.current(), &"state2");
        assert_eq!(history.history().len(), 2);
    }

    #[test]
    fn test_document_state() {
        let mut doc = Document::new();
        doc.submit_for_review().unwrap();
        doc.approve().unwrap();
        doc.publish().unwrap();
        assert_eq!(*doc.state(), DocumentState::Published);
    }

    #[test]
    fn test_auth_machine() {
        let mut auth = AuthMachine::new();
        auth.login("user");
        auth.success("token123");
        if let AuthState::Authenticated { username, .. } = auth.state() {
            assert_eq!(username, "user");
        }
    }

    #[test]
    fn test_resettable_machine() {
        let mut machine = ResettableMachine::new("initial");
        machine.transition("other");
        assert_eq!(machine.state(), "other");
        machine.reset();
        assert_eq!(machine.state(), "initial");
    }

    #[test]
    fn test_player_control() {
        let mut player = PlayerControl::new();
        player.handle_event(&Event::Start);
        assert_eq!(*player.state(), PlayerControlState::Playing);
        player.handle_event(&Event::Pause);
        assert_eq!(*player.state(), PlayerControlState::Paused);
    }

    #[test]
    fn test_session_timeout() {
        let mut session = Session::new(10);
        for _ in 0..10 {
            session.tick();
        }
        assert_eq!(*session.state(), SessionState::Idle);
    }

    #[test]
    fn test_character_state() {
        let mut character = Character::new();
        character.walk();
        assert_eq!(character.state().movement, MovementState::Walking);
        character.die();
        assert_eq!(character.state().health, HealthState::Dead);
    }

    #[test]
    fn test_persistable_machine() {
        let mut machine = PersistableMachine::new();
        machine.transition(PersistableState::Processing);
        let json = machine.serialize();
        let mut new_machine = PersistableMachine::new();
        new_machine.deserialize(&json);
        assert!(matches!(new_machine.state(), PersistableState::Processing));
    }

    #[test]
    fn test_state_machine_builder() {
        let mut sm = StateMachineBuilder::new()
            .add_state("idle")
            .add_state("running")
            .add_transition("idle", "running", "start")
            .add_transition("running", "idle", "stop")
            .initial("idle")
            .build();
        sm.transition("start").unwrap();
        assert_eq!(sm.state(), "running");
    }
}
