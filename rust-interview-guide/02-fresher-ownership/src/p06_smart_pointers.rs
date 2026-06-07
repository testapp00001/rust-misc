/// Problem: Smart Pointers
///
/// Master Rust's smart pointer types.
///
/// Key Concepts:
/// - Box<T> for heap allocation
/// - Rc<T> for reference counting
/// - RefCell<T> for interior mutability
/// - Arc<T> for thread-safe reference counting
/// - Deref and Drop traits

use std::rc::Rc;
use std::cell::RefCell;

/// Problem 1: Box for heap allocation
/// Allocate a value on the heap
pub fn box_example() -> Box<i32> {
    let b = Box::new(5);
    b
}

/// Problem 2: Box for recursive types
/// Use Box for recursive data structures
#[derive(Debug)]
pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

pub fn create_list() -> List {
    List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))))
}

/// Problem 3: Rc for multiple ownership
/// Use Rc for multiple owners
pub fn rc_example() -> (Rc<String>, Rc<String>) {
    let s = Rc::new(String::from("hello"));
    let s2 = Rc::clone(&s);
    (s, s2)
}

/// Problem 4: Rc reference count
/// Check the reference count
pub fn rc_count() -> usize {
    let s = Rc::new(String::from("hello"));
    let s2 = Rc::clone(&s);
    let s3 = Rc::clone(&s);
    Rc::strong_count(&s) // Returns 3
}

/// Problem 5: RefCell for interior mutability
/// Modify data through a shared reference
pub fn refcell_example() -> i32 {
    let x = RefCell::new(5);
    *x.borrow_mut() += 1;
    let result = *x.borrow();
    result
}

/// Problem 6: Rc<RefCell<T>> for shared mutability
/// Combine Rc and RefCell for shared mutable data
pub fn rc_refcell_example() -> (Rc<RefCell<i32>>, Rc<RefCell<i32>>) {
    let x = Rc::new(RefCell::new(5));
    let y = Rc::clone(&x);
    *y.borrow_mut() += 1;
    (x, y)
}

/// Problem 7: Box with trait objects
/// Use Box for trait objects
pub trait Animal {
    fn speak(&self) -> String;
}

pub struct Dog;
pub struct Cat;

impl Animal for Dog {
    fn speak(&self) -> String {
        "Woof!".to_string()
    }
}

impl Animal for Cat {
    fn speak(&self) -> String {
        "Meow!".to_string()
    }
}

pub fn create_animals() -> Vec<Box<dyn Animal>> {
    vec![Box::new(Dog), Box::new(Cat)]
}

/// Problem 8: Deref trait
/// Implement Deref for a custom type
pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(x: T) -> Self {
        MyBox(x)
    }
}

impl<T> std::ops::Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

pub fn deref_example() -> i32 {
    let x = MyBox::new(5);
    *x // Deref allows us to use * operator
}

/// Problem 9: Drop trait
/// Implement Drop for cleanup
pub struct MyResource {
    pub name: String,
}

impl Drop for MyResource {
    fn drop(&mut self) {
        // Cleanup code here
    }
}

pub fn drop_example() -> String {
    let r = MyResource {
        name: "resource".to_string(),
    };
    r.name.clone()
}

/// Problem 10: Rc for shared ownership in graph
/// Use Rc for graph nodes
#[derive(Debug)]
pub struct Node {
    pub value: i32,
    pub neighbors: Vec<Rc<Node>>,
}

pub fn create_graph() -> Rc<Node> {
    let node1 = Rc::new(Node {
        value: 1,
        neighbors: vec![],
    });
    let node2 = Rc::new(Node {
        value: 2,
        neighbors: vec![Rc::clone(&node1)],
    });
    node2
}

/// Problem 11: RefCell with borrow checking
/// Demonstrate runtime borrow checking
pub fn refcell_borrow_check() -> Result<i32, String> {
    let x = RefCell::new(5);
    let r1 = x.borrow();
    let r2 = x.borrow(); // Multiple immutable borrows OK
    Ok(*r1 + *r2)
}

/// Problem 12: Box for large data
/// Use Box to avoid stack overflow
pub fn box_large_data() -> Box<[i32; 1000]> {
    Box::new([0; 1000])
}

/// Problem 13: Rc for circular references
/// Demonstrate circular references with Rc
pub fn circular_reference() -> Rc<RefCell<i32>> {
    let x = Rc::new(RefCell::new(5));
    let y = Rc::clone(&x);
    *y.borrow_mut() += 1;
    x
}

/// Problem 14: Box with Option
/// Use Box in Option for nullable heap data
pub fn box_option() -> Option<Box<i32>> {
    Some(Box::new(42))
}

/// Problem 15: Rc with Vec
/// Share a Vec with multiple owners
pub fn rc_vec() -> (Rc<Vec<i32>>, Rc<Vec<i32>>) {
    let v = Rc::new(vec![1, 2, 3]);
    let v2 = Rc::clone(&v);
    (v, v2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_example() {
        let b = box_example();
        assert_eq!(*b, 5);
    }

    #[test]
    fn test_create_list() {
        let list = create_list();
        match list {
            List::Cons(1, next) => match *next {
                List::Cons(2, next) => match *next {
                    List::Nil => assert!(true),
                    _ => assert!(false),
                },
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn test_rc_example() {
        let (s1, s2) = rc_example();
        assert_eq!(*s1, "hello");
        assert_eq!(*s2, "hello");
    }

    #[test]
    fn test_rc_count() {
        assert_eq!(rc_count(), 3);
    }

    #[test]
    fn test_refcell_example() {
        assert_eq!(refcell_example(), 6);
    }

    #[test]
    fn test_rc_refcell_example() {
        let (x, y) = rc_refcell_example();
        assert_eq!(*x.borrow(), 6);
        assert_eq!(*y.borrow(), 6);
    }

    #[test]
    fn test_create_animals() {
        let animals = create_animals();
        assert_eq!(animals.len(), 2);
        assert_eq!(animals[0].speak(), "Woof!");
        assert_eq!(animals[1].speak(), "Meow!");
    }

    #[test]
    fn test_deref_example() {
        assert_eq!(deref_example(), 5);
    }

    #[test]
    fn test_drop_example() {
        assert_eq!(drop_example(), "resource");
    }

    #[test]
    fn test_create_graph() {
        let node = create_graph();
        assert_eq!(node.value, 2);
        assert_eq!(node.neighbors.len(), 1);
        assert_eq!(node.neighbors[0].value, 1);
    }

    #[test]
    fn test_refcell_borrow_check() {
        assert_eq!(refcell_borrow_check(), Ok(10));
    }

    #[test]
    fn test_box_large_data() {
        let data = box_large_data();
        assert_eq!(data.len(), 1000);
    }

    #[test]
    fn test_circular_reference() {
        let x = circular_reference();
        assert_eq!(*x.borrow(), 6);
    }

    #[test]
    fn test_box_option() {
        let opt = box_option();
        assert!(opt.is_some());
        assert_eq!(*opt.unwrap(), 42);
    }

    #[test]
    fn test_rc_vec() {
        let (v1, v2) = rc_vec();
        assert_eq!(*v1, vec![1, 2, 3]);
        assert_eq!(*v2, vec![1, 2, 3]);
    }
}
