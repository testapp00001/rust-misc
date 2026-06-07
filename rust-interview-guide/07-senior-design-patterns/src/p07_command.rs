/// Problem: Command Pattern
///
/// Master the command pattern in Rust.
///
/// Key Concepts:
/// - Command trait
/// - Invoker
/// - Receiver
/// - Undo/Redo
/// - Command history

/// Problem 1: Basic command
/// Create basic command
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

/// Problem 2: Command with history
/// Track command history
pub struct CommandHistory {
    history: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            undone: Vec::new(),
        }
    }

    pub fn execute(&mut self, command: Box<dyn Command>) -> String {
        let result = command.execute();
        self.history.push(command);
        self.undone.clear();
        result
    }

    pub fn undo(&mut self) -> Option<String> {
        if let Some(command) = self.history.pop() {
            let result = command.undo();
            self.undone.push(command);
            Some(result)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<String> {
        if let Some(command) = self.undone.pop() {
            let result = command.execute();
            self.history.push(command);
            Some(result)
        } else {
            None
        }
    }
}

/// Problem 3: Macro command
/// Combine multiple commands
pub struct MacroCommand {
    commands: Vec<Box<dyn Command>>,
}

impl MacroCommand {
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self { commands }
    }
}

impl Command for MacroCommand {
    fn execute(&self) -> String {
        let mut results = Vec::new();
        for command in &self.commands {
            results.push(command.execute());
        }
        results.join("; ")
    }

    fn undo(&self) -> String {
        let mut results = Vec::new();
        for command in self.commands.iter().rev() {
            results.push(command.undo());
        }
        results.join("; ")
    }
}

/// Problem 4: Command with parameters
/// Parameterized commands
pub struct SetCommand {
    key: String,
    value: String,
    store: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
    old_value: std::sync::Mutex<Option<String>>,
}

impl SetCommand {
    pub fn new(
        key: String,
        value: String,
        store: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
    ) -> Self {
        Self {
            key,
            value,
            store,
            old_value: std::sync::Mutex::new(None),
        }
    }
}

impl Command for SetCommand {
    fn execute(&self) -> String {
        let mut store = self.store.lock().unwrap();
        let old = store.insert(self.key.clone(), self.value.clone());
        *self.old_value.lock().unwrap() = old;
        format!("Set {} = {}", self.key, self.value)
    }

    fn undo(&self) -> String {
        let mut store = self.store.lock().unwrap();
        let old_value = self.old_value.lock().unwrap().take();
        if let Some(old) = old_value {
            store.insert(self.key.clone(), old.clone());
            format!("Undid set {} = {}", self.key, old)
        } else {
            store.remove(&self.key);
            format!("Undid set {} (removed)", self.key)
        }
    }
}

/// Problem 5: Command with validation
/// Validate before execution
pub struct ValidatedCommand {
    value: i32,
    receiver: std::sync::Arc<std::sync::Mutex<i32>>,
}

impl ValidatedCommand {
    pub fn new(value: i32, receiver: std::sync::Arc<std::sync::Mutex<i32>>) -> Self {
        Self { value, receiver }
    }
}

impl Command for ValidatedCommand {
    fn execute(&self) -> String {
        let mut num = self.receiver.lock().unwrap();
        if *num + self.value < 0 {
            return "Cannot go below zero".to_string();
        }
        *num += self.value;
        format!("Added {}", self.value)
    }

    fn undo(&self) -> String {
        let mut num = self.receiver.lock().unwrap();
        *num -= self.value;
        format!("Undid add {}", self.value)
    }
}

/// Problem 6: Command with logging
/// Log command execution
pub struct LoggedCommand {
    inner: Box<dyn Command>,
    log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl LoggedCommand {
    pub fn new(command: Box<dyn Command>, log: std::sync::Arc<std::sync::Mutex<Vec<String>>>) -> Self {
        Self {
            inner: command,
            log,
        }
    }
}

impl Command for LoggedCommand {
    fn execute(&self) -> String {
        let result = self.inner.execute();
        self.log.lock().unwrap().push(format!("Execute: {}", result));
        result
    }

    fn undo(&self) -> String {
        let result = self.inner.undo();
        self.log.lock().unwrap().push(format!("Undo: {}", result));
        result
    }
}

/// Problem 7: Command with timing
/// Time command execution
pub struct TimedCommand {
    inner: Box<dyn Command>,
    duration: std::sync::Mutex<Option<std::time::Duration>>,
}

impl TimedCommand {
    pub fn new(command: Box<dyn Command>) -> Self {
        Self {
            inner: command,
            duration: std::sync::Mutex::new(None),
        }
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        let result = *self.duration.lock().unwrap(); result
    }
}

impl Command for TimedCommand {
    fn execute(&self) -> String {
        let start = std::time::Instant::now();
        let result = self.inner.execute();
        *self.duration.lock().unwrap() = Some(start.elapsed());
        result
    }

    fn undo(&self) -> String {
        self.inner.undo()
    }
}

/// Problem 8: Command with rollback
/// Rollback on failure
pub struct RollbackCommand {
    commands: Vec<Box<dyn Command>>,
    executed: std::sync::Mutex<Vec<Box<dyn Command>>>,
}

impl RollbackCommand {
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self {
            commands,
            executed: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Command for RollbackCommand {
    fn execute(&self) -> String {
        let mut executed = self.executed.lock().unwrap();
        for command in &self.commands {
            let result = command.execute();
            executed.push(Box::new(AddCommand::new(0, std::sync::Arc::new(std::sync::Mutex::new(0)))));
            if result.contains("error") {
                // Rollback
                for cmd in executed.iter().rev() {
                    cmd.undo();
                }
                return "Rolled back".to_string();
            }
        }
        "All executed".to_string()
    }

    fn undo(&self) -> String {
        let executed = self.executed.lock().unwrap();
        for command in executed.iter().rev() {
            command.undo();
        }
        "All undone".to_string()
    }
}

/// Problem 9: Command with conditional
/// Conditional execution
pub struct ConditionalCommand {
    condition: Box<dyn Fn() -> bool>,
    command: Box<dyn Command>,
}

impl ConditionalCommand {
    pub fn new(condition: Box<dyn Fn() -> bool>, command: Box<dyn Command>) -> Self {
        Self { condition, command }
    }
}

impl Command for ConditionalCommand {
    fn execute(&self) -> String {
        if (self.condition)() {
            self.command.execute()
        } else {
            "Condition not met".to_string()
        }
    }

    fn undo(&self) -> String {
        self.command.undo()
    }
}

/// Problem 10: Command with serialization
/// Serialize commands
pub trait SerializableCommand: Command {
    fn serialize(&self) -> String;
}

/// Problem 11: Command with async
/// Async commands (simulated)
pub struct AsyncCommand {
    value: i32,
}

impl AsyncCommand {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl Command for AsyncCommand {
    fn execute(&self) -> String {
        format!("Async executed: {}", self.value)
    }

    fn undo(&self) -> String {
        format!("Async undone: {}", self.value)
    }
}

/// Problem 12: Command with retry
/// Retry on failure
pub struct RetryCommand {
    inner: Box<dyn Command>,
    max_retries: u32,
}

impl RetryCommand {
    pub fn new(command: Box<dyn Command>, max_retries: u32) -> Self {
        Self {
            inner: command,
            max_retries,
        }
    }
}

impl Command for RetryCommand {
    fn execute(&self) -> String {
        for attempt in 0..self.max_retries {
            let result = self.inner.execute();
            if !result.contains("error") {
                return result;
            }
            if attempt == self.max_retries - 1 {
                return format!("Failed after {} retries", self.max_retries);
            }
        }
        "Failed".to_string()
    }

    fn undo(&self) -> String {
        self.inner.undo()
    }
}

/// Problem 13: Command with composition
/// Compose commands
pub struct ComposedCommand {
    first: Box<dyn Command>,
    second: Box<dyn Command>,
}

impl ComposedCommand {
    pub fn new(first: Box<dyn Command>, second: Box<dyn Command>) -> Self {
        Self { first, second }
    }
}

impl Command for ComposedCommand {
    fn execute(&self) -> String {
        let result1 = self.first.execute();
        let result2 = self.second.execute();
        format!("{}; {}", result1, result2)
    }

    fn undo(&self) -> String {
        let result2 = self.second.undo();
        let result1 = self.first.undo();
        format!("{}; {}", result2, result1)
    }
}

/// Problem 14: Command with state
/// Stateful commands
pub struct StatefulCommand {
    state: std::sync::Mutex<String>,
    value: String,
}

impl StatefulCommand {
    pub fn new(value: String) -> Self {
        Self {
            state: std::sync::Mutex::new(String::new()),
            value,
        }
    }
}

impl Command for StatefulCommand {
    fn execute(&self) -> String {
        let mut state = self.state.lock().unwrap();
        *state = self.value.clone();
        format!("State set to: {}", self.value)
    }

    fn undo(&self) -> String {
        let mut state = self.state.lock().unwrap();
        state.clear();
        "State cleared".to_string()
    }
}

/// Problem 15: Command with transaction
/// Transactional commands
pub struct TransactionCommand {
    commands: Vec<Box<dyn Command>>,
    committed: std::sync::Mutex<bool>,
}

impl TransactionCommand {
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self {
            commands,
            committed: std::sync::Mutex::new(false),
        }
    }
}

impl Command for TransactionCommand {
    fn execute(&self) -> String {
        let mut results = Vec::new();
        for command in &self.commands {
            results.push(command.execute());
        }
        *self.committed.lock().unwrap() = true;
        format!("Transaction: {}", results.join("; "))
    }

    fn undo(&self) -> String {
        if *self.committed.lock().unwrap() {
            let mut results = Vec::new();
            for command in self.commands.iter().rev() {
                results.push(command.undo());
            }
            *self.committed.lock().unwrap() = false;
            format!("Rolled back: {}", results.join("; "))
        } else {
            "Nothing to undo".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        let command = AddCommand::new(5, receiver.clone());
        command.execute();
        assert_eq!(*receiver.lock().unwrap(), 5);
        command.undo();
        assert_eq!(*receiver.lock().unwrap(), 0);
    }

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new();
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        history.execute(Box::new(AddCommand::new(5, receiver.clone())));
        assert_eq!(*receiver.lock().unwrap(), 5);
        history.undo();
        assert_eq!(*receiver.lock().unwrap(), 0);
    }

    #[test]
    fn test_macro_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(AddCommand::new(5, receiver.clone())),
            Box::new(AddCommand::new(3, receiver.clone())),
        ];
        let macro_cmd = MacroCommand::new(commands);
        macro_cmd.execute();
        assert_eq!(*receiver.lock().unwrap(), 8);
    }

    #[test]
    fn test_validated_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(10));
        let command = ValidatedCommand::new(-5, receiver.clone());
        command.execute();
        assert_eq!(*receiver.lock().unwrap(), 5);
    }

    #[test]
    fn test_conditional_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        let command = ConditionalCommand::new(
            Box::new(|| true),
            Box::new(AddCommand::new(5, receiver.clone())),
        );
        command.execute();
        assert_eq!(*receiver.lock().unwrap(), 5);
    }

    #[test]
    fn test_composed_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        let command = ComposedCommand::new(
            Box::new(AddCommand::new(5, receiver.clone())),
            Box::new(AddCommand::new(3, receiver.clone())),
        );
        command.execute();
        assert_eq!(*receiver.lock().unwrap(), 8);
    }

    #[test]
    fn test_stateful_command() {
        let command = StatefulCommand::new("test".to_string());
        command.execute();
        assert_eq!(*command.state.lock().unwrap(), "test");
    }

    #[test]
    fn test_transaction_command() {
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(0));
        let command = TransactionCommand::new(vec![
            Box::new(AddCommand::new(5, receiver.clone())),
        ]);
        command.execute();
        assert_eq!(*receiver.lock().unwrap(), 5);
        command.undo();
        assert_eq!(*receiver.lock().unwrap(), 0);
    }
}
