/// Problem: Structs and Enums
///
/// Master Rust's struct and enum system.
///
/// Key Concepts:
/// - Struct definition and instantiation
/// - Methods and associated functions
/// - Enum variants
/// - Option and Result
/// - Pattern matching with enums

/// Problem 1: Basic struct
/// Define a Point struct
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Problem 2: Struct with methods
/// Define a Rectangle struct
#[derive(Debug)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Calculate area
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Check if it's a square
    pub fn is_square(&self) -> bool {
        (self.width - self.height).abs() < f64::EPSILON
    }

    /// Create a square
    pub fn square(size: f64) -> Self {
        Self::new(size, size)
    }
}

/// Problem 3: Basic enum
/// Define a Direction enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Get the opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Get the next direction (clockwise)
    pub fn next_clockwise(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }
}

/// Problem 4: Enum with data
/// Define a Shape enum
#[derive(Debug)]
pub enum Shape {
    Circle(f64),           // radius
    Rectangle(f64, f64),   // width, height
    Triangle(f64, f64),    // base, height
}

impl Shape {
    /// Calculate area
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle(b, h) => 0.5 * b * h,
        }
    }

    /// Get shape name
    pub fn name(&self) -> &'static str {
        match self {
            Shape::Circle(_) => "circle",
            Shape::Rectangle(_, _) => "rectangle",
            Shape::Triangle(_, _) => "triangle",
        }
    }
}

/// Problem 5: Option usage
/// Find the maximum value in a slice
pub fn find_max(arr: &[i32]) -> Option<i32> {
    if arr.is_empty() {
        return None;
    }
    Some(*arr.iter().max().unwrap())
}

/// Problem 6: Result usage
/// Parse a string to integer
pub fn parse_int(s: &str) -> Result<i32, String> {
    s.parse().map_err(|_| format!("Failed to parse '{}'", s))
}

/// Problem 7: Enum with multiple data variants
/// Define a Command enum
#[derive(Debug)]
pub enum Command {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
    ChangeColor(u8, u8, u8),
}

impl Command {
    /// Execute the command (return a description)
    pub fn execute(&self) -> String {
        match self {
            Command::Quit => "Quitting".to_string(),
            Command::Echo(msg) => format!("Echo: {}", msg),
            Command::Move { x, y } => format!("Moving to ({}, {})", x, y),
            Command::ChangeColor(r, g, b) => format!("Changing color to ({}, {}, {})", r, g, b),
        }
    }
}

/// Problem 8: Struct with Option field
/// Define a User struct with optional email
#[derive(Debug)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
}

impl User {
    /// Create a new user
    pub fn new(name: String) -> Self {
        Self { name, email: None }
    }

    /// Set email
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Get email or default
    pub fn email_or_default(&self) -> String {
        self.email.clone().unwrap_or_else(|| "no-email@example.com".to_string())
    }
}

/// Problem 9: Enum with methods
/// Define a TrafficLight enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrafficLight {
    Red,
    Yellow,
    Green,
}

impl TrafficLight {
    /// Get the duration in seconds
    pub fn duration(&self) -> u32 {
        match self {
            TrafficLight::Red => 60,
            TrafficLight::Yellow => 10,
            TrafficLight::Green => 45,
        }
    }

    /// Get the next light
    pub fn next(&self) -> Self {
        match self {
            TrafficLight::Red => TrafficLight::Green,
            TrafficLight::Green => TrafficLight::Yellow,
            TrafficLight::Yellow => TrafficLight::Red,
        }
    }
}

/// Problem 10: Nested structs and enums
/// Define a Card struct
#[derive(Debug, Clone)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rank {
    Ace,
    King,
    Queen,
    Jack,
    Number(u8),
}

impl Card {
    /// Create a new card
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    /// Get the card's value in blackjack
    pub fn blackjack_value(&self) -> u8 {
        match &self.rank {
            Rank::Ace => 11,
            Rank::King | Rank::Queen | Rank::Jack => 10,
            Rank::Number(n) => *n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rectangle_area() {
        let rect = Rectangle::new(5.0, 3.0);
        assert!((rect.area() - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rectangle_is_square() {
        let rect = Rectangle::new(5.0, 5.0);
        assert!(rect.is_square());

        let rect = Rectangle::new(5.0, 3.0);
        assert!(!rect.is_square());
    }

    #[test]
    fn test_rectangle_square_constructor() {
        let sq = Rectangle::square(5.0);
        assert!(sq.is_square());
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
    }

    #[test]
    fn test_direction_next_clockwise() {
        assert_eq!(Direction::North.next_clockwise(), Direction::East);
        assert_eq!(Direction::East.next_clockwise(), Direction::South);
    }

    #[test]
    fn test_shape_area() {
        let circle = Shape::Circle(5.0);
        let expected = std::f64::consts::PI * 25.0;
        assert!((circle.area() - expected).abs() < f64::EPSILON);

        let rect = Shape::Rectangle(4.0, 5.0);
        assert!((rect.area() - 20.0).abs() < f64::EPSILON);

        let tri = Shape::Triangle(3.0, 4.0);
        assert!((tri.area() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_max() {
        assert_eq!(find_max(&[1, 2, 3, 4, 5]), Some(5));
        assert_eq!(find_max(&[]), None);
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("42"), Ok(42));
        assert!(parse_int("abc").is_err());
    }

    #[test]
    fn test_command_execute() {
        assert_eq!(Command::Quit.execute(), "Quitting");
        assert_eq!(Command::Echo("hello".to_string()).execute(), "Echo: hello");
        assert_eq!(Command::Move { x: 1, y: 2 }.execute(), "Moving to (1, 2)");
    }

    #[test]
    fn test_user_email() {
        let user = User::new("Alice".to_string());
        assert_eq!(user.email_or_default(), "no-email@example.com");

        let user = user.with_email("alice@example.com".to_string());
        assert_eq!(user.email_or_default(), "alice@example.com");
    }

    #[test]
    fn test_traffic_light() {
        assert_eq!(TrafficLight::Red.duration(), 60);
        assert_eq!(TrafficLight::Red.next(), TrafficLight::Green);
        assert_eq!(TrafficLight::Green.next(), TrafficLight::Yellow);
        assert_eq!(TrafficLight::Yellow.next(), TrafficLight::Red);
    }

    #[test]
    fn test_card_blackjack_value() {
        let card = Card::new(Suit::Hearts, Rank::Ace);
        assert_eq!(card.blackjack_value(), 11);

        let card = Card::new(Suit::Spades, Rank::King);
        assert_eq!(card.blackjack_value(), 10);

        let card = Card::new(Suit::Diamonds, Rank::Number(7));
        assert_eq!(card.blackjack_value(), 7);
    }
}
