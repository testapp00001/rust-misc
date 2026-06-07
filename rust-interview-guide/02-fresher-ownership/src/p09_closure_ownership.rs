/// Problem: Closure Ownership
///
/// Master ownership in closures.
///
/// Key Concepts:
/// - Closure capture modes
/// - Move closures
/// - Fn, FnMut, FnOnce traits
/// - Returning closures
/// - Closures and lifetimes

/// Problem 1: Closure that borrows
/// A closure that borrows its environment
pub fn borrow_closure() -> i32 {
    let x = 5;
    let add = |y| x + y; // Borrows x
    add(3)
}

/// Problem 2: Closure that moves
/// A closure that takes ownership
pub fn move_closure() -> String {
    let s = String::from("hello");
    let get = move || s; // Takes ownership
    get()
}

/// Problem 3: Closure that mutates
/// A closure that mutates its environment
pub fn mutate_closure() -> i32 {
    let mut x = 5;
    let mut increment = || x += 1; // Mutates x
    increment();
    increment();
    x
}

/// Problem 4: Fn trait
/// A closure that implements Fn
pub fn fn_closure() -> impl Fn(i32) -> i32 {
    |x| x * 2
}

/// Problem 5: FnMut trait
/// A closure that implements FnMut
pub fn fn_mut_closure() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

/// Problem 6: FnOnce trait
/// A closure that implements FnOnce
pub fn fn_once_closure() -> impl FnOnce() -> String {
    let s = String::from("hello");
    move || s // Consumes s
}

/// Problem 7: Closure with reference
/// A closure that captures a reference
pub fn reference_closure() -> impl Fn() -> usize {
    let s = String::from("hello");
    move || s.len() // Borrows s
}

/// Problem 8: Closure with move and clone
/// Use clone to keep original value
pub fn clone_closure() -> (String, impl FnOnce() -> String) {
    let s = String::from("hello");
    let s_clone = s.clone();
    let get = move || s_clone;
    (s, get)
}

/// Problem 9: Closure returning reference
/// A closure that returns a reference
pub fn return_reference_closure() -> impl Fn() -> &'static str {
    || "hello"
}

/// Problem 10: Closure with multiple captures
/// A closure that captures multiple variables
pub fn multiple_captures() -> impl Fn() -> String {
    let name = String::from("Alice");
    let age = 30;
    move || format!("{} is {} years old", name, age)
}

/// Problem 11: Closure in struct
/// Store a closure in a struct
pub struct ClosureHolder {
    pub closure: Box<dyn Fn(i32) -> i32>,
}

impl ClosureHolder {
    pub fn new(closure: Box<dyn Fn(i32) -> i32>) -> Self {
        Self { closure }
    }

    pub fn apply(&self, x: i32) -> i32 {
        (self.closure)(x)
    }
}

/// Problem 12: Closure with lifetime
/// A closure that captures a reference with lifetime
pub fn lifetime_closure<'a>(s: &'a str) -> impl Fn() -> &'a str + 'a {
    move || s
}

/// Problem 13: Closure and iterators
/// Use closures with iterators
pub fn filter_with_closure(v: Vec<i32>, predicate: impl Fn(&i32) -> bool) -> Vec<i32> {
    v.into_iter().filter(|x| predicate(x)).collect()
}

/// Problem 14: Closure and map
/// Use closures with map
pub fn map_with_closure(v: Vec<i32>, f: impl Fn(i32) -> i32) -> Vec<i32> {
    v.into_iter().map(f).collect()
}

/// Problem 15: Closure composition
/// Compose two closures
pub fn compose_closures(
    f: impl Fn(i32) -> i32,
    g: impl Fn(i32) -> i32,
) -> impl Fn(i32) -> i32 {
    move |x| f(g(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_closure() {
        assert_eq!(borrow_closure(), 8);
    }

    #[test]
    fn test_move_closure() {
        assert_eq!(move_closure(), "hello");
    }

    #[test]
    fn test_mutate_closure() {
        assert_eq!(mutate_closure(), 7);
    }

    #[test]
    fn test_fn_closure() {
        let f = fn_closure();
        assert_eq!(f(5), 10);
    }

    #[test]
    fn test_fn_mut_closure() {
        let mut f = fn_mut_closure();
        assert_eq!(f(), 1);
        assert_eq!(f(), 2);
    }

    #[test]
    fn test_fn_once_closure() {
        let f = fn_once_closure();
        assert_eq!(f(), "hello");
    }

    #[test]
    fn test_reference_closure() {
        let f = reference_closure();
        assert_eq!(f(), 5);
    }

    #[test]
    fn test_clone_closure() {
        let (s, get) = clone_closure();
        assert_eq!(s, "hello");
        assert_eq!(get(), "hello");
    }

    #[test]
    fn test_return_reference_closure() {
        let f = return_reference_closure();
        assert_eq!(f(), "hello");
    }

    #[test]
    fn test_multiple_captures() {
        let f = multiple_captures();
        assert_eq!(f(), "Alice is 30 years old");
    }

    #[test]
    fn test_closure_holder() {
        let holder = ClosureHolder::new(Box::new(|x| x * 2));
        assert_eq!(holder.apply(5), 10);
    }

    #[test]
    fn test_lifetime_closure() {
        let s = String::from("hello");
        let f = lifetime_closure(&s);
        assert_eq!(f(), "hello");
    }

    #[test]
    fn test_filter_with_closure() {
        let v = vec![1, 2, 3, 4, 5];
        let result = filter_with_closure(v, |&x| x % 2 == 0);
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_map_with_closure() {
        let v = vec![1, 2, 3];
        let result = map_with_closure(v, |x| x * 2);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_compose_closures() {
        let add_one = |x| x + 1;
        let double = |x| x * 2;
        let add_one_then_double = compose_closures(double, add_one);
        assert_eq!(add_one_then_double(5), 12); // (5 + 1) * 2
    }
}
