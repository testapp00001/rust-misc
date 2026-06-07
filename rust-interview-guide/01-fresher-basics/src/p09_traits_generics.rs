/// Problem: Traits and Generics
///
/// Master Rust's trait and generic system.
///
/// Key Concepts:
/// - Trait definition and implementation
/// - Generic functions and structs
/// - Trait bounds
/// - Associated types
/// - Default implementations
/// - Trait objects

/// Problem 1: Basic trait
/// Define a Drawable trait
pub trait Drawable {
    fn draw(&self) -> String;
}

pub struct Circle {
    pub radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) -> String {
        format!("Drawing circle with radius {}", self.radius)
    }
}

pub struct Square {
    pub side: f64,
}

impl Drawable for Square {
    fn draw(&self) -> String {
        format!("Drawing square with side {}", self.side)
    }
}

/// Problem 2: Trait with default implementation
/// Define a Describable trait with default
pub trait Describable {
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("This is a {}", self.name())
    }
}

pub struct Car {
    pub make: String,
    pub model: String,
}

impl Describable for Car {
    fn name(&self) -> String {
        format!("{} {}", self.make, self.model)
    }
}

/// Problem 3: Generic function
/// Find the maximum of two values
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Problem 4: Generic struct
/// A container that holds a value
pub struct Container<T> {
    pub value: T,
}

impl<T> Container<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Problem 5: Trait bounds
/// Print anything that implements Display
pub fn print_value<T: std::fmt::Display>(value: T) -> String {
    format!("{}", value)
}

/// Problem 6: Multiple trait bounds
/// A function that requires both Display and Debug
pub fn print_debug<T: std::fmt::Display + std::fmt::Debug>(value: T) -> String {
    format!("Display: {}, Debug: {:?}", value, value)
}

/// Problem 7: Trait with associated types
/// Define a Container trait with associated type
pub trait MyContainer {
    type Item;
    fn get(&self) -> &Self::Item;
    fn set(&mut self, item: Self::Item);
}

pub struct MyBox<T> {
    value: T,
}

impl<T> MyContainer for MyBox<T> {
    type Item = T;

    fn get(&self) -> &T {
        &self.value
    }

    fn set(&mut self, item: T) {
        self.value = item;
    }
}

/// Problem 8: Implementing Display trait
/// Implement Display for a custom type
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Problem 9: Trait objects
/// Use trait objects for dynamic dispatch
pub fn draw_all(shapes: &[&dyn Drawable]) -> Vec<String> {
    shapes.iter().map(|s| s.draw()).collect()
}

/// Problem 10: Generic with where clause
/// Use where clause for complex bounds
pub fn process<T>(value: T) -> String
where
    T: std::fmt::Display + std::fmt::Debug + Clone,
{
    let cloned = value.clone();
    format!("Original: {}, Cloned: {:?}", value, cloned)
}

/// Problem 11: Implementing From trait
/// Convert between types
pub struct Celsius(pub f64);
pub struct Fahrenheit(pub f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

/// Problem 12: Generic iterator
/// Create a generic range iterator
pub struct MyRange {
    start: i32,
    end: i32,
    current: i32,
}

impl MyRange {
    pub fn new(start: i32, end: i32) -> Self {
        Self { start, end, current: start }
    }
}

impl Iterator for MyRange {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let result = self.current;
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

/// Problem 13: Trait inheritance
/// Define a trait that extends another
pub trait Animal {
    fn name(&self) -> String;
}

pub trait Pet: Animal {
    fn owner(&self) -> String;
}

pub struct Dog {
    pub name: String,
    pub owner: String,
}

impl Animal for Dog {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Pet for Dog {
    fn owner(&self) -> String {
        self.owner.clone()
    }
}

/// Problem 14: Generic struct with multiple types
/// A pair of values
pub struct Pair<T, U> {
    pub first: T,
    pub second: U,
}

impl<T, U> Pair<T, U> {
    pub fn new(first: T, second: U) -> Self {
        Self { first, second }
    }

    pub fn into_tuple(self) -> (T, U) {
        (self.first, self.second)
    }
}

/// Problem 15: Implementing Add trait
/// Implement addition for a custom type
#[derive(Debug, Clone, Copy)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl std::ops::Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawable() {
        let circle = Circle { radius: 5.0 };
        let square = Square { side: 4.0 };
        assert_eq!(circle.draw(), "Drawing circle with radius 5");
        assert_eq!(square.draw(), "Drawing square with side 4");
    }

    #[test]
    fn test_describable() {
        let car = Car {
            make: "Toyota".to_string(),
            model: "Camry".to_string(),
        };
        assert_eq!(car.name(), "Toyota Camry");
        assert_eq!(car.describe(), "This is a Toyota Camry");
    }

    #[test]
    fn test_max() {
        assert_eq!(max(5, 10), 10);
        assert_eq!(max(3.14, 2.72), 3.14);
        assert_eq!(max("apple", "banana"), "banana");
    }

    #[test]
    fn test_container() {
        let container = Container::new(42);
        assert_eq!(*container.get(), 42);
    }

    #[test]
    fn test_print_value() {
        assert_eq!(print_value(42), "42");
        assert_eq!(print_value("hello"), "hello");
    }

    #[test]
    fn test_print_debug() {
        assert_eq!(print_debug(42), "Display: 42, Debug: 42");
    }

    #[test]
    fn test_my_container() {
        let mut container = MyBox { value: 42 };
        assert_eq!(*container.get(), 42);
        container.set(100);
        assert_eq!(*container.get(), 100);
    }

    #[test]
    fn test_point_display() {
        let p = Point { x: 1.0, y: 2.0 };
        assert_eq!(format!("{}", p), "(1, 2)");
    }

    #[test]
    fn test_draw_all() {
        let shapes: Vec<&dyn Drawable> = vec![
            &Circle { radius: 5.0 },
            &Square { side: 4.0 },
        ];
        let result = draw_all(&shapes);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_process() {
        assert_eq!(process(42), "Original: 42, Cloned: 42");
    }

    #[test]
    fn test_celsius_to_fahrenheit() {
        let c = Celsius(100.0);
        let f = Fahrenheit::from(c);
        assert!((f.0 - 212.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_my_range() {
        let range: Vec<i32> = MyRange::new(0, 5).collect();
        assert_eq!(range, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_animal_pet() {
        let dog = Dog {
            name: "Buddy".to_string(),
            owner: "Alice".to_string(),
        };
        assert_eq!(dog.name(), "Buddy");
        assert_eq!(dog.owner(), "Alice");
    }

    #[test]
    fn test_pair() {
        let pair = Pair::new(1, "hello");
        let (first, second) = pair.into_tuple();
        assert_eq!(first, 1);
        assert_eq!(second, "hello");
    }

    #[test]
    fn test_vector_add() {
        let v1 = Vector2D { x: 1.0, y: 2.0 };
        let v2 = Vector2D { x: 3.0, y: 4.0 };
        let v3 = v1 + v2;
        assert!((v3.x - 4.0).abs() < f64::EPSILON);
        assert!((v3.y - 6.0).abs() < f64::EPSILON);
    }
}
