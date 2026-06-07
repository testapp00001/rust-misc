/// Problem: Functions and Closures
///
/// Master Rust's function system and closures.
///
/// Key Concepts:
/// - Function signatures
/// - Return types
/// - Closures (anonymous functions)
/// - Closure capture (borrow, move)
/// - Function pointers
/// - Higher-order functions

/// Problem 1: Basic function
/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Problem 2: Function with early return
/// Find the first positive number
pub fn first_positive(arr: &[i32]) -> Option<i32> {
    for &x in arr {
        if x > 0 {
            return Some(x);
        }
    }
    None
}

/// Problem 3: Function returning closure
/// Create an adder function
pub fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

/// Problem 4: Basic closure
/// Use a closure to double a number
pub fn double(x: i32) -> i32 {
    let double = |x| x * 2;
    double(x)
}

/// Problem 5: Closure capturing environment
/// Use a closure that captures a variable
pub fn create_counter() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

/// Problem 6: Closure as parameter
/// Apply a function to each element
pub fn apply_to_vec(v: Vec<i32>, f: impl Fn(i32) -> i32) -> Vec<i32> {
    v.into_iter().map(f).collect()
}

/// Problem 7: Closure returning bool (predicate)
/// Filter a vector using a predicate
pub fn filter_vec(v: Vec<i32>, predicate: impl Fn(&i32) -> bool) -> Vec<i32> {
    v.into_iter().filter(|x| predicate(x)).collect()
}

/// Problem 8: Function pointer
/// Use a function pointer as a parameter
pub fn apply_function(x: i32, f: fn(i32) -> i32) -> i32 {
    f(x)
}

pub fn square(x: i32) -> i32 {
    x * x
}

/// Problem 9: Closure with move
/// Create a closure that owns its captured data
pub fn create_greeting(name: String) -> impl Fn() -> String {
    move || format!("Hello, {}!", name)
}

/// Problem 10: Higher-order function
/// Compose two functions
pub fn compose(f: impl Fn(i32) -> i32, g: impl Fn(i32) -> i32) -> impl Fn(i32) -> i32 {
    move |x| f(g(x))
}

/// Problem 11: Closure with multiple captures
/// Create a closure that uses multiple variables
pub fn create_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor
}

/// Problem 12: Returning closures from match
/// Create different closures based on condition
pub fn create_operation(op: char) -> Box<dyn Fn(i32, i32) -> i32> {
    match op {
        '+' => Box::new(|a, b| a + b),
        '-' => Box::new(|a, b| a - b),
        '*' => Box::new(|a, b| a * b),
        '/' => Box::new(|a, b| a / b),
        _ => panic!("Unknown operation"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_first_positive() {
        assert_eq!(first_positive(&[-1, -2, 3, 4]), Some(3));
        assert_eq!(first_positive(&[-1, -2]), None);
    }

    #[test]
    fn test_make_adder() {
        let add5 = make_adder(5);
        assert_eq!(add5(3), 8);
        assert_eq!(add5(10), 15);
    }

    #[test]
    fn test_double() {
        assert_eq!(double(5), 10);
    }

    #[test]
    fn test_create_counter() {
        let mut counter = create_counter();
        assert_eq!(counter(), 1);
        assert_eq!(counter(), 2);
        assert_eq!(counter(), 3);
    }

    #[test]
    fn test_apply_to_vec() {
        let v = vec![1, 2, 3];
        let result = apply_to_vec(v, |x| x * 2);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_filter_vec() {
        let v = vec![1, 2, 3, 4, 5];
        let result = filter_vec(v, |&x| x % 2 == 0);
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_apply_function() {
        assert_eq!(apply_function(5, square), 25);
    }

    #[test]
    fn test_create_greeting() {
        let greet = create_greeting("Alice".to_string());
        assert_eq!(greet(), "Hello, Alice!");
    }

    #[test]
    fn test_compose() {
        let add_one = |x| x + 1;
        let double = |x| x * 2;
        let add_one_then_double = compose(double, add_one);
        assert_eq!(add_one_then_double(5), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_create_multiplier() {
        let triple = create_multiplier(3);
        assert_eq!(triple(5), 15);
    }

    #[test]
    fn test_create_operation() {
        let add = create_operation('+');
        assert_eq!(add(2, 3), 5);

        let mul = create_operation('*');
        assert_eq!(mul(2, 3), 6);
    }
}
