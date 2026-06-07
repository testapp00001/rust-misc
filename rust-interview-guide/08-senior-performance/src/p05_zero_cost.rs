/// Problem: Zero-Cost Abstractions
///
/// Master zero-cost abstractions in Rust.
///
/// Key Concepts:
/// - Iterators
/// - Closures
/// - Generics
/// - Traits
/// - Monomorphization

/// Problem 1: Iterator as zero-cost
/// Iterators compile to same code as loops
pub fn iterator_sum(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 2: Closure as zero-cost
/// Closures are inlined
pub fn closure_map(data: &[i32]) -> Vec<i32> {
    data.iter().map(|&x| x * 2).collect()
}

/// Problem 3: Generic as zero-cost
/// Generics are monomorphized
pub fn generic_add<T: std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T {
    a + b
}

/// Problem 4: Trait as zero-cost
/// Trait dispatch is static
pub trait Processor {
    fn process(&self, x: i32) -> i32;
}

pub struct Double;

impl Processor for Double {
    fn process(&self, x: i32) -> i32 {
        x * 2
    }
}

pub fn use_processor(p: &impl Processor, x: i32) -> i32 {
    p.process(x)
}

/// Problem 5: Enum as zero-cost
/// Enums are optimized
#[derive(Debug)]
pub enum Option2<T> {
    Some(T),
    None,
}

impl<T> Option2<T> {
    pub fn unwrap(self) -> T {
        match self {
            Option2::Some(v) => v,
            Option2::None => panic!("None"),
        }
    }
}

/// Problem 6: Box as zero-cost (nearly)
/// Box is a thin pointer
pub fn box_example() -> Box<i32> {
    Box::new(42)
}

/// Problem 7: Vec as zero-cost (nearly)
/// Vec is three words
pub fn vec_example() -> Vec<i32> {
    vec![1, 2, 3]
}

/// Problem 8: String as zero-cost (nearly)
/// String is three words
pub fn string_example() -> String {
    "hello".to_string()
}

/// Problem 9: Result as zero-cost
/// Result uses niche optimization
pub fn result_example() -> Result<i32, String> {
    Ok(42)
}

/// Problem 10: Future as zero-cost
/// Futures are state machines
pub async fn future_example() -> i32 {
    42
}

/// Problem 11: Iterator chain as zero-cost
/// Chains compile to single loop
pub fn iterator_chain(data: &[i32]) -> Vec<i32> {
    data.iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .collect()
}

/// Problem 12: Closure capture as zero-cost
/// Captures are optimized
pub fn closure_capture(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

/// Problem 13: Generic struct as zero-cost
/// Generic structs are monomorphized
pub struct Wrapper<T> {
    value: T,
}

impl<T> Wrapper<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Problem 14: Trait object vs enum
/// Enum is often faster
#[derive(Debug)]
pub enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
        }
    }
}

/// Problem 15: Zero-cost error handling
/// ? operator is zero-cost
pub fn error_handling() -> Result<i32, String> {
    let x: i32 = "42".parse().map_err(|e| format!("{}", e))?;
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_sum() {
        assert_eq!(iterator_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_closure_map() {
        assert_eq!(closure_map(&[1, 2, 3]), vec![2, 4, 6]);
    }

    #[test]
    fn test_generic_add() {
        assert_eq!(generic_add(20, 22), 42);
    }

    #[test]
    fn test_use_processor() {
        let p = Double;
        assert_eq!(use_processor(&p, 21), 42);
    }

    #[test]
    fn test_enum() {
        let opt = Option2::Some(42);
        assert_eq!(opt.unwrap(), 42);
    }

    #[test]
    fn test_box() {
        assert_eq!(*box_example(), 42);
    }

    #[test]
    fn test_vec() {
        assert_eq!(vec_example(), vec![1, 2, 3]);
    }

    #[test]
    fn test_string() {
        assert_eq!(string_example(), "hello");
    }

    #[test]
    fn test_result() {
        assert_eq!(result_example(), Ok(42));
    }

    #[tokio::test]
    async fn test_future() {
        assert_eq!(future_example().await, 42);
    }

    #[test]
    fn test_iterator_chain() {
        assert_eq!(iterator_chain(&[1, -2, 3, -4, 5]), vec![2, 6, 10]);
    }

    #[test]
    fn test_closure_capture() {
        let f = closure_capture(20);
        assert_eq!(f(22), 42);
    }

    #[test]
    fn test_wrapper() {
        let w = Wrapper::new(42);
        assert_eq!(*w.get(), 42);
    }

    #[test]
    fn test_shape() {
        let circle = Shape::Circle(5.0);
        let expected = std::f64::consts::PI * 25.0;
        assert!((circle.area() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_error_handling() {
        assert_eq!(error_handling(), Ok(42));
    }
}
