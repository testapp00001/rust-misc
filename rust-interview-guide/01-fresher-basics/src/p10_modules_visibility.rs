/// Problem: Modules and Visibility
///
/// Master Rust's module system.
///
/// Key Concepts:
/// - Module declaration
/// - Visibility (pub, pub(crate), pub(super))
/// - use statements
/// - Re-exports
/// - Nested modules

/// Problem 1: Basic module
/// Define a module with public items
pub mod shapes {
    pub struct Circle {
        pub radius: f64,
    }

    impl Circle {
        pub fn new(radius: f64) -> Self {
            Self { radius }
        }

        pub fn area(&self) -> f64 {
            std::f64::consts::PI * self.radius * self.radius
        }
    }
}

/// Problem 2: Module with private items
/// A module with both public and private items
pub mod math {
    // Private function
    fn validate_positive(n: i32) -> bool {
        n > 0
    }

    // Public function that uses private function
    pub fn factorial(n: i32) -> Option<u64> {
        if !validate_positive(n) {
            return None;
        }
        Some((1..=n as u64).product())
    }
}

/// Problem 3: Re-exporting
/// Re-export items from a module
pub mod geometry {
    pub use super::shapes::Circle;

    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    impl Rectangle {
        pub fn new(width: f64, height: f64) -> Self {
            Self { width, height }
        }

        pub fn area(&self) -> f64 {
            self.width * self.height
        }
    }
}

/// Problem 4: Nested modules
/// A module with nested submodules
pub mod company {
    pub mod employees {
        pub struct Employee {
            pub name: String,
            pub department: String,
        }

        impl Employee {
            pub fn new(name: &str, department: &str) -> Self {
                Self {
                    name: name.to_string(),
                    department: department.to_string(),
                }
            }
        }
    }

    pub mod departments {
        pub struct Department {
            pub name: String,
            pub budget: f64,
        }

        impl Department {
            pub fn new(name: &str, budget: f64) -> Self {
                Self {
                    name: name.to_string(),
                    budget,
                }
            }
        }
    }
}

/// Problem 5: Visibility modifiers
/// Use different visibility levels
pub mod config {
    // Public to everyone
    pub const MAX_RETRIES: u32 = 3;

    // Public only within the crate
    pub(crate) fn get_timeout() -> u64 {
        30
    }

    // Private to this module
    fn validate_config() -> bool {
        true
    }

    // Public function that uses private function
    pub fn load_config() -> String {
        if validate_config() {
            "Config loaded".to_string()
        } else {
            "Config invalid".to_string()
        }
    }
}

/// Problem 6: use statements
/// Use items from modules
pub use geometry::Circle;
pub use geometry::Rectangle;

/// Problem 7: Module with impl blocks
/// Split implementation across files (simulated)
pub mod database {
    pub struct Connection {
        pub url: String,
        pub pool_size: u32,
    }

    // First impl block
    impl Connection {
        pub fn new(url: &str) -> Self {
            Self {
                url: url.to_string(),
                pool_size: 10,
            }
        }
    }

    // Second impl block (simulates separate file)
    impl Connection {
        pub fn with_pool_size(mut self, size: u32) -> Self {
            self.pool_size = size;
            self
        }

        pub fn connect(&self) -> String {
            format!("Connected to {} with pool size {}", self.url, self.pool_size)
        }
    }
}

/// Problem 8: Module with traits
/// Define and implement traits in modules
pub mod traits {
    pub trait Serializable {
        fn serialize(&self) -> String;
    }

    pub trait Deserializable {
        type Error;
        fn deserialize(s: &str) -> Result<Self, Self::Error>
        where
            Self: Sized;
    }
}

/// Problem 9: Module with type aliases
/// Create type aliases in modules
pub mod types {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    pub type Callback = Box<dyn Fn(i32) -> i32>;

    pub fn create_adder(x: i32) -> Callback {
        Box::new(move |y| x + y)
    }
}

/// Problem 10: Module with constants
/// Define constants in modules
pub mod constants {
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;

    pub fn circle_area(radius: f64) -> f64 {
        PI * radius * radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shapes_circle() {
        let circle = shapes::Circle::new(5.0);
        let expected = std::f64::consts::PI * 25.0;
        assert!((circle.area() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_math_factorial() {
        assert_eq!(math::factorial(5), Some(120));
        assert_eq!(math::factorial(-1), None);
    }

    #[test]
    fn test_geometry_re_export() {
        let circle = geometry::Circle::new(3.0);
        let expected = std::f64::consts::PI * 9.0;
        assert!((circle.area() - expected).abs() < f64::EPSILON);

        let rect = geometry::Rectangle::new(4.0, 5.0);
        assert!((rect.area() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_company_modules() {
        let employee = company::employees::Employee::new("Alice", "Engineering");
        assert_eq!(employee.name, "Alice");

        let dept = company::departments::Department::new("Engineering", 100000.0);
        assert_eq!(dept.name, "Engineering");
    }

    #[test]
    fn test_config() {
        assert_eq!(config::MAX_RETRIES, 3);
        assert_eq!(config::load_config(), "Config loaded");
    }

    #[test]
    fn test_re_export() {
        let circle = Circle::new(5.0);
        let expected = std::f64::consts::PI * 25.0;
        assert!((circle.area() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_database_connection() {
        let conn = database::Connection::new("postgres://localhost")
            .with_pool_size(20);
        assert_eq!(conn.connect(), "Connected to postgres://localhost with pool size 20");
    }

    #[test]
    fn test_types() {
        let adder = types::create_adder(5);
        assert_eq!(adder(3), 8);
    }

    #[test]
    fn test_constants() {
        assert!((constants::PI - std::f64::consts::PI).abs() < f64::EPSILON);
        let area = constants::circle_area(5.0);
        let expected = std::f64::consts::PI * 25.0;
        assert!((area - expected).abs() < f64::EPSILON);
    }
}
