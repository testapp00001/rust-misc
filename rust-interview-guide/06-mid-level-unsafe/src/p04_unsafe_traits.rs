/// Problem: Unsafe Traits
///
/// Master unsafe traits in Rust.
///
/// Key Concepts:
/// - Unsafe trait definition
/// - Unsafe trait implementation
/// - Safety invariants
/// - Documentation
/// - Common unsafe traits

/// Problem 1: Define unsafe trait
/// Define an unsafe trait
unsafe trait MyUnsafeTrait {
    fn my_method(&self) -> i32;
}

/// Problem 2: Implement unsafe trait
/// Implement unsafe trait for a type
struct MyStruct {
    value: i32,
}

unsafe impl MyUnsafeTrait for MyStruct {
    fn my_method(&self) -> i32 {
        self.value
    }
}

/// Problem 3: Unsafe trait with unsafe method
/// Define unsafe trait with unsafe method
unsafe trait UnsafeMethod {
    unsafe fn dangerous_method(&self) -> i32;
}

struct SafeStruct {
    value: i32,
}

unsafe impl UnsafeMethod for SafeStruct {
    unsafe fn dangerous_method(&self) -> i32 {
        self.value
    }
}

/// Problem 4: Implement Send trait
/// Implement Send for custom type
#[derive(Debug)]
struct MySendType {
    data: Vec<i32>,
}

unsafe impl Send for MySendType {}

/// Problem 5: Implement Sync trait
/// Implement Sync for custom type
#[derive(Debug)]
struct MySyncType {
    data: std::sync::Mutex<i32>,
}

unsafe impl Sync for MySyncType {}

/// Problem 6: Unsafe trait with lifetime
/// Unsafe trait with lifetime
unsafe trait LifetimeTrait<'a> {
    fn get(&self) -> &'a i32;
}

struct LifetimeStruct<'a> {
    data: &'a i32,
}

unsafe impl<'a> LifetimeTrait<'a> for LifetimeStruct<'a> {
    fn get(&self) -> &'a i32 {
        self.data
    }
}

/// Problem 7: Unsafe trait with generic
/// Unsafe trait with generic
unsafe trait GenericTrait<T> {
    fn get(&self) -> &T;
}

struct GenericStruct<T> {
    data: T,
}

unsafe impl<T> GenericTrait<T> for GenericStruct<T> {
    fn get(&self) -> &T {
        &self.data
    }
}

/// Problem 8: Unsafe trait with associated type
/// Unsafe trait with associated type
unsafe trait AssociatedType {
    type Item;
    fn get(&self) -> Self::Item;
}

struct AssociatedStruct {
    value: i32,
}

unsafe impl AssociatedType for AssociatedStruct {
    type Item = i32;

    fn get(&self) -> Self::Item {
        self.value
    }
}

/// Problem 9: Unsafe trait with default implementation
/// Unsafe trait with default
unsafe trait DefaultTrait {
    fn required(&self) -> i32;

    fn default_method(&self) -> i32 {
        self.required() * 2
    }
}

struct DefaultStruct {
    value: i32,
}

unsafe impl DefaultTrait for DefaultStruct {
    fn required(&self) -> i32 {
        self.value
    }
}

/// Problem 10: Unsafe trait with supertraits
/// Unsafe trait with supertraits
trait SafeTrait {
    fn safe_method(&self) -> i32;
}

unsafe trait SuperTrait: SafeTrait {
    fn unsafe_method(&self) -> i32;
}

struct SuperStruct {
    value: i32,
}

impl SafeTrait for SuperStruct {
    fn safe_method(&self) -> i32 {
        self.value
    }
}

unsafe impl SuperTrait for SuperStruct {
    fn unsafe_method(&self) -> i32 {
        self.value * 2
    }
}

/// Problem 11: Unsafe trait with where clause
/// Unsafe trait with where clause
unsafe trait WhereTrait<T: Clone> {
    fn get(&self) -> T;
}

struct WhereStruct {
    value: i32,
}

unsafe impl WhereTrait<i32> for WhereStruct {
    fn get(&self) -> i32 {
        self.value
    }
}

/// Problem 12: Unsafe trait with const
/// Unsafe trait with const
unsafe trait ConstTrait {
    const VALUE: i32;
}

struct ConstStruct;

unsafe impl ConstTrait for ConstStruct {
    const VALUE: i32 = 42;
}

/// Problem 13: Unsafe trait with static method
/// Unsafe trait with static method
unsafe trait StaticTrait {
    fn new() -> Self;
}

struct StaticStruct {
    value: i32,
}

unsafe impl StaticTrait for StaticStruct {
    fn new() -> Self {
        Self { value: 42 }
    }
}

/// Problem 14: Unsafe trait with unsafe implementation
/// Unsafe trait with unsafe implementation details
unsafe trait UnsafeImpl {
    fn safe_wrapper(&self) -> i32;

    unsafe fn unsafe_inner(&self) -> i32;
}

struct UnsafeImplStruct {
    value: i32,
}

unsafe impl UnsafeImpl for UnsafeImplStruct {
    fn safe_wrapper(&self) -> i32 {
        unsafe { self.unsafe_inner() }
    }

    unsafe fn unsafe_inner(&self) -> i32 {
        self.value
    }
}

/// Problem 15: Safe wrapper around unsafe trait
/// Create safe wrapper
trait SafeWrapper {
    fn safe_method(&self) -> i32;
}

struct Wrapper<T: MyUnsafeTrait> {
    inner: T,
}

impl<T: MyUnsafeTrait> SafeWrapper for Wrapper<T> {
    fn safe_method(&self) -> i32 {
        self.inner.my_method()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_unsafe_trait() {
        let s = MyStruct { value: 42 };
        assert_eq!(s.my_method(), 42);
    }

    #[test]
    fn test_unsafe_method() {
        let s = SafeStruct { value: 42 };
        unsafe {
            assert_eq!(s.dangerous_method(), 42);
        }
    }

    #[test]
    fn test_send_trait() {
        let s = MySendType { data: vec![1, 2, 3] };
        assert!(std::mem::size_of_val(&s) > 0);
    }

    #[test]
    fn test_sync_trait() {
        let s = MySyncType { data: std::sync::Mutex::new(42) };
        drop(s.data.lock().unwrap());
    }

    #[test]
    fn test_lifetime_trait() {
        let x = 42;
        let s = LifetimeStruct { data: &x };
        assert_eq!(*s.get(), 42);
    }

    #[test]
    fn test_generic_trait() {
        let s = GenericStruct { data: 42 };
        assert_eq!(*s.get(), 42);
    }

    #[test]
    fn test_associated_type() {
        let s = AssociatedStruct { value: 42 };
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_default_trait() {
        let s = DefaultStruct { value: 21 };
        assert_eq!(s.default_method(), 42);
    }

    #[test]
    fn test_super_trait() {
        let s = SuperStruct { value: 21 };
        assert_eq!(s.safe_method(), 21);
        assert_eq!(s.unsafe_method(), 42);
    }

    #[test]
    fn test_where_trait() {
        let s = WhereStruct { value: 42 };
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn test_const_trait() {
        assert_eq!(ConstStruct::VALUE, 42);
    }

    #[test]
    fn test_static_trait() {
        let s = StaticStruct::new();
        assert_eq!(s.value, 42);
    }

    #[test]
    fn test_unsafe_impl() {
        let s = UnsafeImplStruct { value: 42 };
        assert_eq!(s.safe_wrapper(), 42);
    }

    #[test]
    fn test_safe_wrapper() {
        let inner = MyStruct { value: 42 };
        let wrapper = Wrapper { inner };
        assert_eq!(wrapper.safe_method(), 42);
    }
}
