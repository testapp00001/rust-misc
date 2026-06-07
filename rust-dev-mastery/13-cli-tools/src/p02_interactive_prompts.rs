//! # Interactive Prompts
//!
//! Building interactive CLIs that gather user input through prompts, selections,
//! and confirmations. This lesson covers patterns for terminal-based user
//! interaction without external dependencies, demonstrating the underlying
//! concepts used by libraries like dialoguer and inquire.
//!
//! ## Key Concepts
//! - Reading from stdin with validation
//! - Confirmation prompts (yes/no)
//! - Selection menus
//! - Fuzzy matching
//! - Password input (hidden)
//! - Multi-step wizards

// (imports added as needed)

// ---------------------------------------------------------------------------
// 1. Prompt Builder
// ---------------------------------------------------------------------------

/// A builder for creating interactive prompts.
#[derive(Debug)]
pub struct PromptBuilder {
    message: String,
    default: Option<String>,
    required: bool,
    validator: Option<fn(&str) -> Result<(), String>>,
}

impl PromptBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            required: true,
            validator: None,
        }
    }

    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn validate(mut self, validator: fn(&str) -> Result<(), String>) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Build the prompt message string.
    pub fn format_message(&self) -> String {
        let mut msg = self.message.clone();
        if let Some(ref default) = self.default {
            msg.push_str(&format!(" [{default}]"));
        }
        if !self.required {
            msg.push_str(" (optional)");
        }
        msg.push_str(": ");
        msg
    }

    /// Simulate prompting for input (for testing, we pass input as parameter).
    pub fn process_input(&self, input: &str) -> Result<Option<String>, String> {
        let value = if input.trim().is_empty() {
            if let Some(ref default) = self.default {
                default.clone()
            } else if self.required {
                return Err("input is required".into());
            } else {
                return Ok(None);
            }
        } else {
            input.trim().to_string()
        };

        if let Some(validator) = self.validator {
            validator(&value)?;
        }

        Ok(Some(value))
    }
}

// ---------------------------------------------------------------------------
// 2. Confirmation Prompt
// ---------------------------------------------------------------------------

/// A yes/no confirmation prompt.
#[derive(Debug)]
pub struct ConfirmPrompt {
    message: String,
    default: bool,
}

impl ConfirmPrompt {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: false,
        }
    }

    pub fn default_yes(mut self) -> Self {
        self.default = true;
        self
    }

    /// Format the prompt message.
    pub fn format_message(&self) -> String {
        let hint = if self.default { "Y/n" } else { "y/N" };
        format!("{} [{}]: ", self.message, hint)
    }

    /// Parse the user's response.
    pub fn parse_response(&self, input: &str) -> bool {
        let trimmed = input.trim().to_lowercase();
        if trimmed.is_empty() {
            self.default
        } else {
            matches!(trimmed.as_str(), "y" | "yes" | "true" | "1")
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Selection Menu
// ---------------------------------------------------------------------------

/// A selection menu that lets users pick from a list of options.
#[derive(Debug)]
pub struct SelectPrompt<T: std::fmt::Display> {
    message: String,
    options: Vec<T>,
    default: Option<usize>,
}

impl<T: std::fmt::Display> SelectPrompt<T> {
    pub fn new(message: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            message: message.into(),
            options,
            default: None,
        }
    }

    pub fn default_index(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    /// Format the menu for display.
    pub fn format_menu(&self) -> String {
        let mut output = format!("{}\n", self.message);
        for (i, option) in self.options.iter().enumerate() {
            let marker = if self.default == Some(i) {
                ">"
            } else {
                " "
            };
            output.push_str(&format!("  {marker} {}. {}\n", i + 1, option));
        }
        output.push_str("Enter number: ");
        output
    }

    /// Parse a selection from user input.
    pub fn parse_selection(&self, input: &str) -> Result<&T, String> {
        let trimmed = input.trim();
        let index = if trimmed.is_empty() {
            self.default.ok_or("no default and no input provided")?
        } else {
            trimmed
                .parse::<usize>()
                .map_err(|_| format!("'{trimmed}' is not a valid number"))?
                .checked_sub(1)
                .ok_or("selection must be >= 1")?
        };

        self.options
            .get(index)
            .ok_or(format!("selection {index} out of range"))
    }

    pub fn options(&self) -> &[T] {
        &self.options
    }
}

// ---------------------------------------------------------------------------
// 4. Multi-Select Prompt
// ---------------------------------------------------------------------------

/// A multi-select prompt where users can pick multiple options.
#[derive(Debug)]
pub struct MultiSelectPrompt<T: std::fmt::Display + Clone> {
    message: String,
    options: Vec<T>,
    defaults: Vec<bool>,
}

impl<T: std::fmt::Display + Clone> MultiSelectPrompt<T> {
    pub fn new(message: impl Into<String>, options: Vec<T>) -> Self {
        let len = options.len();
        Self {
            message: message.into(),
            options,
            defaults: vec![false; len],
        }
    }

    pub fn with_defaults(mut self, defaults: Vec<bool>) -> Self {
        self.defaults = defaults;
        self
    }

    /// Format the menu for display.
    pub fn format_menu(&self) -> String {
        let mut output = format!("{} (comma-separated numbers)\n", self.message);
        for (i, option) in self.options.iter().enumerate() {
            let marker = if self.defaults.get(i).copied().unwrap_or(false) {
                "[x]"
            } else {
                "[ ]"
            };
            output.push_str(&format!("  {marker} {}. {}\n", i + 1, option));
        }
        output.push_str("Enter selections: ");
        output
    }

    /// Parse multiple selections from comma-separated input.
    pub fn parse_selections(&self, input: &str) -> Result<Vec<&T>, String> {
        let mut selected = Vec::new();

        if input.trim().is_empty() {
            // Use defaults
            for (i, &default) in self.defaults.iter().enumerate() {
                if default {
                    selected.push(&self.options[i]);
                }
            }
            return Ok(selected);
        }

        for part in input.split(',') {
            let trimmed = part.trim();
            let index = trimmed
                .parse::<usize>()
                .map_err(|_| format!("'{trimmed}' is not a valid number"))?
                .checked_sub(1)
                .ok_or("selection must be >= 1")?;

            let option = self
                .options
                .get(index)
                .ok_or(format!("selection {} out of range", index + 1))?;
            selected.push(option);
        }

        Ok(selected)
    }
}

// ---------------------------------------------------------------------------
// 5. Password Prompt
// ---------------------------------------------------------------------------

/// A password prompt that validates strength.
#[derive(Debug)]
pub struct PasswordPrompt {
    message: String,
    min_length: usize,
    require_uppercase: bool,
    require_digit: bool,
}

impl PasswordPrompt {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            min_length: 8,
            require_uppercase: true,
            require_digit: true,
        }
    }

    pub fn min_length(mut self, len: usize) -> Self {
        self.min_length = len;
        self
    }

    pub fn no_uppercase_requirement(mut self) -> Self {
        self.require_uppercase = false;
        self
    }

    pub fn no_digit_requirement(mut self) -> Self {
        self.require_digit = false;
        self
    }

    pub fn format_message(&self) -> String {
        format!("{}: ", self.message)
    }

    /// Validate password strength.
    pub fn validate(&self, password: &str) -> Result<(), String> {
        if password.len() < self.min_length {
            return Err(format!(
                "password must be at least {} characters",
                self.min_length
            ));
        }
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err("password must contain at least one uppercase letter".into());
        }
        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err("password must contain at least one digit".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Input Validators
// ---------------------------------------------------------------------------

/// Common input validation functions.
pub mod validators {
    pub fn email(input: &str) -> Result<(), String> {
        if input.contains('@') && input.contains('.') && !input.starts_with('@') {
            Ok(())
        } else {
            Err("invalid email format".into())
        }
    }

    pub fn non_empty(input: &str) -> Result<(), String> {
        if input.trim().is_empty() {
            Err("input cannot be empty".into())
        } else {
            Ok(())
        }
    }

    pub fn numeric(input: &str) -> Result<(), String> {
        if input.chars().all(|c| c.is_ascii_digit()) {
            Ok(())
        } else {
            Err("input must be numeric".into())
        }
    }

    pub fn alphanumeric(input: &str) -> Result<(), String> {
        if input.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            Ok(())
        } else {
            Err("input must be alphanumeric (underscores and hyphens allowed)".into())
        }
    }

    pub fn min_length(min: usize) -> Box<dyn Fn(&str) -> Result<(), String>> {
        Box::new(move |input: &str| {
            if input.len() >= min {
                Ok(())
            } else {
                Err(format!("must be at least {min} characters"))
            }
        })
    }

    pub fn max_length(max: usize) -> Box<dyn Fn(&str) -> Result<(), String>> {
        Box::new(move |input: &str| {
            if input.len() <= max {
                Ok(())
            } else {
                Err(format!("must be at most {max} characters"))
            }
        })
    }
}

// ---------------------------------------------------------------------------
// 7. Wizard Pattern
// ---------------------------------------------------------------------------

/// A step in a setup wizard.
#[derive(Debug, Clone)]
pub struct WizardStep {
    pub id: String,
    pub prompt: String,
    pub default: Option<String>,
    pub validator: Option<fn(&str) -> Result<(), String>>,
    pub skip_if: Option<fn(&std::collections::HashMap<String, String>) -> bool>,
}

/// A multi-step wizard that collects information across multiple prompts.
#[derive(Debug)]
pub struct Wizard {
    steps: Vec<WizardStep>,
    results: std::collections::HashMap<String, String>,
}

impl Wizard {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            results: std::collections::HashMap::new(),
        }
    }

    pub fn add_step(mut self, step: WizardStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Get the steps that should be shown based on current results.
    pub fn visible_steps(&self) -> Vec<&WizardStep> {
        self.steps
            .iter()
            .filter(|step| {
                if let Some(skip_if) = step.skip_if {
                    !skip_if(&self.results)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Process input for a specific step.
    pub fn process_step(&mut self, step_id: &str, input: &str) -> Result<(), String> {
        let step = self
            .steps
            .iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| format!("unknown step: {step_id}"))?;

        let value = if input.trim().is_empty() {
            step.default.clone().ok_or("input is required")?
        } else {
            input.trim().to_string()
        };

        if let Some(validator) = step.validator {
            validator(&value)?;
        }

        self.results.insert(step_id.into(), value);
        Ok(())
    }

    pub fn get_result(&self, key: &str) -> Option<&String> {
        self.results.get(key)
    }

    pub fn results(&self) -> &std::collections::HashMap<String, String> {
        &self.results
    }

    pub fn is_complete(&self) -> bool {
        self.visible_steps()
            .iter()
            .all(|step| self.results.contains_key(&step.id))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_format() {
        let prompt = PromptBuilder::new("Enter name").default("Alice");
        assert_eq!(prompt.format_message(), "Enter name [Alice]: ");
    }

    #[test]
    fn test_prompt_builder_optional() {
        let prompt = PromptBuilder::new("Enter bio").optional();
        assert!(prompt.format_message().contains("(optional)"));
    }

    #[test]
    fn test_prompt_process_input_with_value() {
        let prompt = PromptBuilder::new("Name");
        let result = prompt.process_input("Alice").unwrap();
        assert_eq!(result, Some("Alice".into()));
    }

    #[test]
    fn test_prompt_process_input_empty_with_default() {
        let prompt = PromptBuilder::new("Name").default("Bob");
        let result = prompt.process_input("").unwrap();
        assert_eq!(result, Some("Bob".into()));
    }

    #[test]
    fn test_prompt_process_input_empty_required() {
        let prompt = PromptBuilder::new("Name");
        assert!(prompt.process_input("").is_err());
    }

    #[test]
    fn test_prompt_process_input_empty_optional() {
        let prompt = PromptBuilder::new("Name").optional();
        let result = prompt.process_input("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_prompt_with_validation() {
        let prompt = PromptBuilder::new("Email").validate(validators::email);
        assert!(prompt.process_input("user@example.com").is_ok());
        assert!(prompt.process_input("not-an-email").is_err());
    }

    #[test]
    fn test_confirm_prompt_yes() {
        let prompt = ConfirmPrompt::new("Continue?");
        assert_eq!(prompt.format_message(), "Continue? [y/N]: ");
        assert!(prompt.parse_response("y"));
        assert!(prompt.parse_response("yes"));
        assert!(prompt.parse_response("YES"));
        assert!(!prompt.parse_response("n"));
        assert!(!prompt.parse_response("no"));
    }

    #[test]
    fn test_confirm_prompt_default() {
        let prompt = ConfirmPrompt::new("Proceed?").default_yes();
        assert_eq!(prompt.format_message(), "Proceed? [Y/n]: ");
        assert!(prompt.parse_response("")); // empty = default = yes
    }

    #[test]
    fn test_confirm_prompt_default_no() {
        let prompt = ConfirmPrompt::new("Delete?");
        assert!(!prompt.parse_response("")); // empty = default = no
    }

    #[test]
    fn test_select_prompt() {
        let prompt = SelectPrompt::new("Choose color", vec!["Red", "Green", "Blue"]);
        let menu = prompt.format_menu();
        assert!(menu.contains("1. Red"));
        assert!(menu.contains("2. Green"));
        assert!(menu.contains("3. Blue"));
    }

    #[test]
    fn test_select_prompt_parse() {
        let prompt = SelectPrompt::new("Choose", vec!["A", "B", "C"]);
        assert_eq!(*prompt.parse_selection("2").unwrap(), "B");
        assert_eq!(*prompt.parse_selection("1").unwrap(), "A");
        assert!(prompt.parse_selection("0").is_err());
        assert!(prompt.parse_selection("4").is_err());
        assert!(prompt.parse_selection("abc").is_err());
    }

    #[test]
    fn test_select_prompt_with_default() {
        let prompt = SelectPrompt::new("Choose", vec!["A", "B", "C"]).default_index(1);
        assert_eq!(*prompt.parse_selection("").unwrap(), "B");
    }

    #[test]
    fn test_multi_select_prompt() {
        let prompt =
            MultiSelectPrompt::new("Pick toppings", vec!["Cheese", "Pepperoni", "Mushrooms"]);
        let menu = prompt.format_menu();
        assert!(menu.contains("[ ]"));
        assert!(menu.contains("1. Cheese"));
    }

    #[test]
    fn test_multi_select_parse() {
        let prompt = MultiSelectPrompt::new("Pick", vec!["A", "B", "C", "D"]);
        let selected = prompt.parse_selections("1, 3").unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(*selected[0], "A");
        assert_eq!(*selected[1], "C");
    }

    #[test]
    fn test_multi_select_with_defaults() {
        let prompt =
            MultiSelectPrompt::new("Pick", vec!["A", "B", "C"]).with_defaults(vec![true, false, true]);
        let selected = prompt.parse_selections("").unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(*selected[0], "A");
        assert_eq!(*selected[1], "C");
    }

    #[test]
    fn test_password_prompt_validation() {
        let prompt = PasswordPrompt::new("Enter password");
        assert!(prompt.validate("SecureP4ss").is_ok());
        assert!(prompt.validate("short").is_err()); // too short
        assert!(prompt.validate("nouppercase1").is_err()); // no uppercase
        assert!(prompt.validate("NoDigits").is_err()); // no digit
    }

    #[test]
    fn test_password_prompt_custom_requirements() {
        let prompt = PasswordPrompt::new("PIN")
            .min_length(4)
            .no_uppercase_requirement()
            .no_digit_requirement();
        assert!(prompt.validate("abcd").is_ok());
        assert!(prompt.validate("abc").is_err());
    }

    #[test]
    fn test_validators_email() {
        assert!(validators::email("user@example.com").is_ok());
        assert!(validators::email("bad").is_err());
        assert!(validators::email("@domain.com").is_err());
    }

    #[test]
    fn test_validators_non_empty() {
        assert!(validators::non_empty("hello").is_ok());
        assert!(validators::non_empty("").is_err());
        assert!(validators::non_empty("   ").is_err());
    }

    #[test]
    fn test_validators_numeric() {
        assert!(validators::numeric("12345").is_ok());
        assert!(validators::numeric("12a45").is_err());
    }

    #[test]
    fn test_validators_alphanumeric() {
        assert!(validators::alphanumeric("hello_world-123").is_ok());
        assert!(validators::alphanumeric("hello world").is_err());
    }

    #[test]
    fn test_wizard_basic() {
        let wizard = Wizard::new()
            .add_step(WizardStep {
                id: "name".into(),
                prompt: "Enter name".into(),
                default: None,
                validator: None,
                skip_if: None,
            })
            .add_step(WizardStep {
                id: "email".into(),
                prompt: "Enter email".into(),
                default: None,
                validator: None,
                skip_if: None,
            });

        let mut wizard = wizard;
        assert!(!wizard.is_complete());

        wizard.process_step("name", "Alice").unwrap();
        assert!(!wizard.is_complete());

        wizard.process_step("email", "alice@example.com").unwrap();
        assert!(wizard.is_complete());

        assert_eq!(wizard.get_result("name").unwrap(), "Alice");
        assert_eq!(wizard.get_result("email").unwrap(), "alice@example.com");
    }

    #[test]
    fn test_wizard_with_defaults() {
        let mut wizard = Wizard::new().add_step(WizardStep {
            id: "lang".into(),
            prompt: "Language".into(),
            default: Some("en".into()),
            validator: None,
            skip_if: None,
        });

        wizard.process_step("lang", "").unwrap();
        assert_eq!(wizard.get_result("lang").unwrap(), "en");
    }

    #[test]
    fn test_wizard_with_validation() {
        let mut wizard = Wizard::new().add_step(WizardStep {
            id: "email".into(),
            prompt: "Email".into(),
            default: None,
            validator: Some(validators::email),
            skip_if: None,
        });

        assert!(wizard.process_step("email", "bad").is_err());
        assert!(wizard.process_step("email", "ok@test.com").is_ok());
    }

    #[test]
    fn test_wizard_unknown_step() {
        let mut wizard = Wizard::new();
        assert!(wizard.process_step("unknown", "val").is_err());
    }

    #[test]
    fn test_prompt_whitespace_trimming() {
        let prompt = PromptBuilder::new("Name");
        let result = prompt.process_input("  Alice  ").unwrap();
        assert_eq!(result, Some("Alice".into()));
    }
}
