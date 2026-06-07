/// Problem: Serialization
///
/// Master Rust's serialization with serde.
///
/// Key Concepts:
/// - Serialize trait
/// - Deserialize trait
/// - JSON serialization
/// - Custom serialization
/// - Container attributes

use serde::{Serialize, Deserialize};

/// Problem 1: Basic serialization
/// Serialize a struct to JSON
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: String,
}

pub fn serialize_user(user: &User) -> String {
    serde_json::to_string(user).unwrap()
}

/// Problem 2: Basic deserialization
/// Deserialize JSON to struct
pub fn deserialize_user(json: &str) -> User {
    serde_json::from_str(json).unwrap()
}

/// Problem 3: Serialization with Option
/// Handle optional fields
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserProfile {
    pub name: String,
    pub bio: Option<String>,
    pub age: Option<u32>,
}

pub fn serialize_profile(profile: &UserProfile) -> String {
    serde_json::to_string(profile).unwrap()
}

/// Problem 4: Serialization with Vec
/// Handle vectors
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Team {
    pub name: String,
    pub members: Vec<String>,
}

pub fn serialize_team(team: &Team) -> String {
    serde_json::to_string(team).unwrap()
}

/// Problem 5: Serialization with HashMap
/// Handle HashMap
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub settings: std::collections::HashMap<String, String>,
}

pub fn serialize_config(config: &Config) -> String {
    serde_json::to_string(config).unwrap()
}

/// Problem 6: Serialization with enum
/// Handle enums
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Account {
    pub username: String,
    pub status: Status,
}

pub fn serialize_account(account: &Account) -> String {
    serde_json::to_string(account).unwrap()
}

/// Problem 7: Serialization with nested structs
/// Handle nested structures
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub country: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Person {
    pub name: String,
    pub address: Address,
}

pub fn serialize_person(person: &Person) -> String {
    serde_json::to_string(person).unwrap()
}

/// Problem 8: Serialization with rename
/// Use rename attribute
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RenamedUser {
    #[serde(rename = "user_name")]
    pub name: String,
    #[serde(rename = "user_age")]
    pub age: u32,
}

pub fn serialize_renamed_user(user: &RenamedUser) -> String {
    serde_json::to_string(user).unwrap()
}

/// Problem 9: Serialization with default
/// Use default attribute
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DefaultUser {
    pub name: String,
    #[serde(default)]
    pub age: u32,
}

pub fn deserialize_default_user(json: &str) -> DefaultUser {
    serde_json::from_str(json).unwrap()
}

/// Problem 10: Serialization with skip
/// Skip fields
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SkipUser {
    pub name: String,
    #[serde(skip)]
    pub password: String,
}

pub fn serialize_skip_user(user: &SkipUser) -> String {
    serde_json::to_string(user).unwrap()
}

/// Problem 11: Serialization with flatten
/// Flatten nested structs
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FlatUser {
    pub name: String,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

pub fn serialize_flat_user(user: &FlatUser) -> String {
    serde_json::to_string(user).unwrap()
}

/// Problem 12: Serialization with custom
/// Custom serialization
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CustomUser {
    pub name: String,
    #[serde(serialize_with = "serialize_uppercase")]
    pub email: String,
}

fn serialize_uppercase<S>(email: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&email.to_uppercase())
}

pub fn serialize_custom_user(user: &CustomUser) -> String {
    serde_json::to_string(user).unwrap()
}

/// Problem 13: Serialization with tagged enum
/// Use tagged enums
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Message {
    Text { content: String },
    Image { url: String },
    Video { url: String, duration: u32 },
}

pub fn serialize_message(message: &Message) -> String {
    serde_json::to_string(message).unwrap()
}

/// Problem 14: Serialization with container attributes
/// Use container attributes
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CamelCaseUser {
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub fn serialize_camel_case_user(user: &CamelCaseUser) -> String {
    serde_json::to_string(user).unwrap()
}

/// Problem 15: Pretty print JSON
/// Pretty print JSON
pub fn pretty_print<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_user() {
        let user = User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        };
        let json = serialize_user(&user);
        assert!(json.contains("Alice"));
    }

    #[test]
    fn test_deserialize_user() {
        let json = r#"{"name":"Alice","age":30,"email":"alice@example.com"}"#;
        let user = deserialize_user(json);
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_serialize_profile() {
        let profile = UserProfile {
            name: "Alice".to_string(),
            bio: Some("Hello".to_string()),
            age: None,
        };
        let json = serialize_profile(&profile);
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_serialize_team() {
        let team = Team {
            name: "Engineering".to_string(),
            members: vec!["Alice".to_string(), "Bob".to_string()],
        };
        let json = serialize_team(&team);
        assert!(json.contains("Engineering"));
    }

    #[test]
    fn test_serialize_config() {
        let mut settings = std::collections::HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        let config = Config { settings };
        let json = serialize_config(&config);
        assert!(json.contains("key"));
    }

    #[test]
    fn test_serialize_account() {
        let account = Account {
            username: "alice".to_string(),
            status: Status::Active,
        };
        let json = serialize_account(&account);
        assert!(json.contains("Active"));
    }

    #[test]
    fn test_serialize_person() {
        let person = Person {
            name: "Alice".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
                country: "US".to_string(),
            },
        };
        let json = serialize_person(&person);
        assert!(json.contains("123 Main St"));
    }

    #[test]
    fn test_serialize_renamed_user() {
        let user = RenamedUser {
            name: "Alice".to_string(),
            age: 30,
        };
        let json = serialize_renamed_user(&user);
        assert!(json.contains("user_name"));
    }

    #[test]
    fn test_deserialize_default_user() {
        let json = r#"{"name":"Alice"}"#;
        let user = deserialize_default_user(json);
        assert_eq!(user.age, 0);
    }

    #[test]
    fn test_serialize_skip_user() {
        let user = SkipUser {
            name: "Alice".to_string(),
            password: "secret".to_string(),
        };
        let json = serialize_skip_user(&user);
        assert!(!json.contains("secret"));
    }

    #[test]
    fn test_serialize_flat_user() {
        let mut extra = std::collections::HashMap::new();
        extra.insert("age".to_string(), serde_json::json!(30));
        let user = FlatUser {
            name: "Alice".to_string(),
            extra,
        };
        let json = serialize_flat_user(&user);
        assert!(json.contains("age"));
    }

    #[test]
    fn test_serialize_custom_user() {
        let user = CustomUser {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let json = serialize_custom_user(&user);
        assert!(json.contains("ALICE@EXAMPLE.COM"));
    }

    #[test]
    fn test_serialize_message() {
        let message = Message::Text {
            content: "Hello".to_string(),
        };
        let json = serialize_message(&message);
        assert!(json.contains("Text"));
    }

    #[test]
    fn test_serialize_camel_case_user() {
        let user = CamelCaseUser {
            first_name: "Alice".to_string(),
            last_name: "Smith".to_string(),
            email_address: "alice@example.com".to_string(),
        };
        let json = serialize_camel_case_user(&user);
        assert!(json.contains("firstName"));
    }

    #[test]
    fn test_pretty_print() {
        let user = User {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        };
        let json = pretty_print(&user);
        assert!(json.contains('\n'));
    }
}
