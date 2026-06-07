//! # Command Pattern
//!
//! The command pattern encapsulates a request as an object, enabling undo/redo,
//! command queues, and macro commands. In Rust, commands are typically trait objects
//! or enum variants.
//!
//! ## Key Concepts
//! - **Command trait**: Execute and undo operations
//! - **Command history**: Stack of executed commands for undo
//! - **Macro command**: Composite command that groups multiple commands
//! - **Command queue**: Deferred execution of commands

use std::collections::VecDeque;

/// A command that can be executed and undone.
pub trait Command {
    fn execute(&mut self) -> Result<(), CommandError>;
    fn undo(&mut self) -> Result<(), CommandError>;
    fn description(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    CannotUndo,
    ExecutionFailed(String),
}

/// A text document that supports command-based editing.
#[derive(Debug, Clone)]
pub struct Document {
    pub content: String,
    pub cursor: usize,
}

impl Document {
    pub fn new() -> Self {
        Document {
            content: String::new(),
            cursor: 0,
        }
    }
}

/// Inserts text at the cursor position.
pub struct InsertCommand {
    document: std::rc::Rc<std::cell::RefCell<Document>>,
    text: String,
    position: usize,
    position_set: bool,
}

impl InsertCommand {
    pub fn new(doc: std::rc::Rc<std::cell::RefCell<Document>>, text: impl Into<String>) -> Self {
        InsertCommand {
            document: doc,
            text: text.into(),
            position: 0,
            position_set: false,
        }
    }
}

impl Command for InsertCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        let mut doc = self.document.borrow_mut();
        if !self.position_set {
            self.position = doc.cursor;
            self.position_set = true;
        }
        if self.position <= doc.content.len() {
            doc.content.insert_str(self.position, &self.text);
            doc.cursor = self.position + self.text.len();
            Ok(())
        } else {
            Err(CommandError::ExecutionFailed("Position out of bounds".into()))
        }
    }

    fn undo(&mut self) -> Result<(), CommandError> {
        let mut doc = self.document.borrow_mut();
        let end = self.position + self.text.len();
        if end <= doc.content.len() {
            doc.content.drain(self.position..end);
            doc.cursor = self.position;
            Ok(())
        } else {
            Err(CommandError::CannotUndo)
        }
    }

    fn description(&self) -> &str {
        "Insert"
    }
}

/// Deletes text from a range.
pub struct DeleteCommand {
    document: std::rc::Rc<std::cell::RefCell<Document>>,
    start: usize,
    end: usize,
    deleted_text: Option<String>,
}

impl DeleteCommand {
    pub fn new(
        doc: std::rc::Rc<std::cell::RefCell<Document>>,
        start: usize,
        end: usize,
    ) -> Self {
        DeleteCommand {
            document: doc,
            start,
            end,
            deleted_text: None,
        }
    }
}

impl Command for DeleteCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        let mut doc = self.document.borrow_mut();
        if self.start <= self.end && self.end <= doc.content.len() {
            self.deleted_text = Some(doc.content[self.start..self.end].to_string());
            doc.content.drain(self.start..self.end);
            doc.cursor = self.start;
            Ok(())
        } else {
            Err(CommandError::ExecutionFailed("Invalid range".into()))
        }
    }

    fn undo(&mut self) -> Result<(), CommandError> {
        if let Some(ref text) = self.deleted_text {
            let mut doc = self.document.borrow_mut();
            doc.content.insert_str(self.start, text);
            doc.cursor = self.start + text.len();
            Ok(())
        } else {
            Err(CommandError::CannotUndo)
        }
    }

    fn description(&self) -> &str {
        "Delete"
    }
}

/// A command history with undo/redo support.
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    pub fn new() -> Self {
        CommandHistory {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>) -> Result<(), CommandError> {
        cmd.execute()?;
        self.undo_stack.push(cmd);
        self.redo_stack.clear(); // New command invalidates redo
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), CommandError> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo()?;
            self.redo_stack.push(cmd);
            Ok(())
        } else {
            Err(CommandError::CannotUndo)
        }
    }

    pub fn redo(&mut self) -> Result<(), CommandError> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute()?;
            self.undo_stack.push(cmd);
            Ok(())
        } else {
            Err(CommandError::CannotUndo)
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
}

/// A macro command that groups multiple commands into one.
pub struct MacroCommand {
    commands: Vec<Box<dyn Command>>,
    description: String,
}

impl MacroCommand {
    pub fn new(description: impl Into<String>) -> Self {
        MacroCommand {
            commands: Vec::new(),
            description: description.into(),
        }
    }

    pub fn add(&mut self, cmd: Box<dyn Command>) {
        self.commands.push(cmd);
    }
}

impl Command for MacroCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        for cmd in &mut self.commands {
            cmd.execute()?;
        }
        Ok(())
    }

    fn undo(&mut self) -> Result<(), CommandError> {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo()?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// A command queue for deferred execution.
pub struct CommandQueue {
    queue: VecDeque<Box<dyn Command>>,
    history: Vec<Box<dyn Command>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        CommandQueue {
            queue: VecDeque::new(),
            history: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, cmd: Box<dyn Command>) {
        self.queue.push_back(cmd);
    }

    pub fn execute_next(&mut self) -> Result<bool, CommandError> {
        if let Some(mut cmd) = self.queue.pop_front() {
            cmd.execute()?;
            self.history.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn execute_all(&mut self) -> Result<usize, CommandError> {
        let mut count = 0;
        while self.execute_next()? {
            count += 1;
        }
        Ok(count)
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }
}

/// A simple counter with command-based operations.
#[derive(Debug)]
pub struct Counter {
    pub value: i64,
}

impl Counter {
    pub fn new() -> Self {
        Counter { value: 0 }
    }
}

pub struct IncrementCommand {
    counter: std::rc::Rc<std::cell::RefCell<Counter>>,
    amount: i64,
}

impl IncrementCommand {
    pub fn new(counter: std::rc::Rc<std::cell::RefCell<Counter>>, amount: i64) -> Self {
        IncrementCommand { counter, amount }
    }
}

impl Command for IncrementCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        self.counter.borrow_mut().value += self.amount;
        Ok(())
    }

    fn undo(&mut self) -> Result<(), CommandError> {
        self.counter.borrow_mut().value -= self.amount;
        Ok(())
    }

    fn description(&self) -> &str {
        "Increment"
    }
}

pub struct SetValueCommand {
    counter: std::rc::Rc<std::cell::RefCell<Counter>>,
    new_value: i64,
    old_value: Option<i64>,
}

impl SetValueCommand {
    pub fn new(counter: std::rc::Rc<std::cell::RefCell<Counter>>, value: i64) -> Self {
        SetValueCommand {
            counter,
            new_value: value,
            old_value: None,
        }
    }
}

impl Command for SetValueCommand {
    fn execute(&mut self) -> Result<(), CommandError> {
        let mut counter = self.counter.borrow_mut();
        self.old_value = Some(counter.value);
        counter.value = self.new_value;
        Ok(())
    }

    fn undo(&mut self) -> Result<(), CommandError> {
        if let Some(old) = self.old_value {
            self.counter.borrow_mut().value = old;
            Ok(())
        } else {
            Err(CommandError::CannotUndo)
        }
    }

    fn description(&self) -> &str {
        "SetValue"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_command() {
        let doc = std::rc::Rc::new(std::cell::RefCell::new(Document::new()));
        let mut cmd = InsertCommand::new(doc.clone(), "Hello");
        cmd.execute().unwrap();
        assert_eq!(doc.borrow().content, "Hello");
        cmd.undo().unwrap();
        assert_eq!(doc.borrow().content, "");
    }

    #[test]
    fn test_delete_command() {
        let doc = std::rc::Rc::new(std::cell::RefCell::new(Document {
            content: "Hello World".to_string(),
            cursor: 0,
        }));
        let mut cmd = DeleteCommand::new(doc.clone(), 5, 11);
        cmd.execute().unwrap();
        assert_eq!(doc.borrow().content, "Hello");
        cmd.undo().unwrap();
        assert_eq!(doc.borrow().content, "Hello World");
    }

    #[test]
    fn test_command_history_undo_redo() {
        let doc = std::rc::Rc::new(std::cell::RefCell::new(Document::new()));
        let mut history = CommandHistory::new();

        history
            .execute(Box::new(InsertCommand::new(doc.clone(), "Hello")))
            .unwrap();
        history
            .execute(Box::new(InsertCommand::new(doc.clone(), " World")))
            .unwrap();

        assert_eq!(doc.borrow().content, "Hello World");
        assert!(history.can_undo());

        history.undo().unwrap();
        assert_eq!(doc.borrow().content, "Hello");

        history.redo().unwrap();
        assert_eq!(doc.borrow().content, "Hello World");
    }

    #[test]
    fn test_macro_command() {
        let doc = std::rc::Rc::new(std::cell::RefCell::new(Document::new()));
        let mut macro_cmd = MacroCommand::new("Insert greeting");

        macro_cmd.add(Box::new(InsertCommand::new(doc.clone(), "Hello")));
        macro_cmd.add(Box::new(InsertCommand::new(doc.clone(), " World")));

        macro_cmd.execute().unwrap();
        assert_eq!(doc.borrow().content, "Hello World");

        macro_cmd.undo().unwrap();
        assert_eq!(doc.borrow().content, "");
    }

    #[test]
    fn test_command_queue() {
        let counter = std::rc::Rc::new(std::cell::RefCell::new(Counter::new()));
        let mut queue = CommandQueue::new();

        queue.enqueue(Box::new(IncrementCommand::new(counter.clone(), 10)));
        queue.enqueue(Box::new(IncrementCommand::new(counter.clone(), 20)));

        assert_eq!(queue.pending(), 2);
        queue.execute_all().unwrap();
        assert_eq!(counter.borrow().value, 30);
    }

    #[test]
    fn test_counter_set_value() {
        let counter = std::rc::Rc::new(std::cell::RefCell::new(Counter::new()));
        let mut history = CommandHistory::new();

        history
            .execute(Box::new(SetValueCommand::new(counter.clone(), 42)))
            .unwrap();
        assert_eq!(counter.borrow().value, 42);

        history.undo().unwrap();
        assert_eq!(counter.borrow().value, 0);
    }

    #[test]
    fn test_increment_command() {
        let counter = std::rc::Rc::new(std::cell::RefCell::new(Counter::new()));
        let mut cmd = IncrementCommand::new(counter.clone(), 5);

        cmd.execute().unwrap();
        assert_eq!(counter.borrow().value, 5);

        cmd.execute().unwrap();
        assert_eq!(counter.borrow().value, 10);

        cmd.undo().unwrap();
        assert_eq!(counter.borrow().value, 5);
    }
}
