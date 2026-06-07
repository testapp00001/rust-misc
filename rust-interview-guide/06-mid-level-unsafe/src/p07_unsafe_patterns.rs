/// Problem: Unsafe Patterns
///
/// Master common unsafe patterns in Rust.
///
/// Key Concepts:
/// - Type punning
/// - Interior mutability
/// - Zero-cost abstractions
/// - Optimization patterns
/// - Platform-specific code

/// Problem 1: Type punning with union
/// Use union for type punning
pub union TypePun {
    pub i: i32,
    pub f: f32,
}

pub fn type_pun_int_to_float(i: i32) -> f32 {
    let pun = TypePun { i };
    unsafe { pun.f }
}

/// Problem 2: Interior mutability pattern
/// Use Cell for interior mutability
pub struct InteriorMutability {
    data: std::cell::Cell<i32>,
}

impl InteriorMutability {
    pub fn new(value: i32) -> Self {
        Self {
            data: std::cell::Cell::new(value),
        }
    }

    pub fn get(&self) -> i32 {
        self.data.get()
    }

    pub fn set(&self, value: i32) {
        self.data.set(value);
    }
}

/// Problem 3: RefCell pattern
/// Use RefCell for interior mutability
pub struct RefCellPattern {
    data: std::cell::RefCell<Vec<i32>>,
}

impl RefCellPattern {
    pub fn new() -> Self {
        Self {
            data: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn push(&self, value: i32) {
        self.data.borrow_mut().push(value);
    }

    pub fn get(&self, index: usize) -> Option<i32> {
        self.data.borrow().get(index).copied()
    }
}

/// Problem 4: UnsafeCell pattern
/// Use UnsafeCell for interior mutability
pub struct UnsafeCellPattern {
    data: std::cell::UnsafeCell<i32>,
}

impl UnsafeCellPattern {
    pub fn new(value: i32) -> Self {
        Self {
            data: std::cell::UnsafeCell::new(value),
        }
    }

    pub fn get(&self) -> i32 {
        unsafe { *self.data.get() }
    }

    pub fn set(&self, value: i32) {
        unsafe { *self.data.get() = value; }
    }
}

/// Problem 5: Static mut pattern
/// Use static mut (dangerous)
pub fn static_mut_pattern() -> i32 {
    static mut COUNTER: i32 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

/// Problem 6: Lazy initialization pattern
/// Use lazy initialization
pub struct LazyInit {
    data: std::sync::Once,
    value: std::cell::UnsafeCell<Option<i32>>,
}

impl LazyInit {
    pub fn new() -> Self {
        Self {
            data: std::sync::Once::new(),
            value: std::cell::UnsafeCell::new(None),
        }
    }

    pub fn get(&self) -> i32 {
        self.data.call_once(|| {
            unsafe { *self.value.get() = Some(42); }
        });
        unsafe { (*self.value.get()).unwrap() }
    }
}

/// Problem 7: Transmute pattern
/// Use transmute for type conversion
pub fn transmute_pattern(x: i32) -> u32 {
    unsafe { std::mem::transmute(x) }
}

/// Problem 8: Transmute copy pattern
/// Use transmute_copy for zero-cost conversion
pub fn transmute_copy_pattern(x: &i32) -> u32 {
    unsafe { std::mem::transmute_copy(x) }
}

/// Problem 9: MaybeUninit pattern
/// Use MaybeUninit for uninitialized memory
pub fn maybe_uninit_pattern() -> i32 {
    let mut x = std::mem::MaybeUninit::<i32>::uninit();
    unsafe {
        x.as_mut_ptr().write(42);
        x.assume_init()
    }
}

/// Problem 10: ManuallyDrop pattern
/// Use ManuallyDrop for custom drop
pub struct ManuallyDropPattern {
    data: std::mem::ManuallyDrop<Vec<i32>>,
}

impl ManuallyDropPattern {
    pub fn new() -> Self {
        Self {
            data: std::mem::ManuallyDrop::new(Vec::new()),
        }
    }

    pub fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    pub fn into_vec(mut self) -> Vec<i32> {
        let data = std::mem::take(&mut self.data);
        std::mem::ManuallyDrop::into_inner(data)
    }
}

/// Problem 11: Pin pattern
/// Use Pin for self-referential structs
pub struct PinPattern {
    data: std::pin::Pin<Box<i32>>,
}

impl PinPattern {
    pub fn new(value: i32) -> Self {
        Self {
            data: Box::pin(value),
        }
    }

    pub fn get(&self) -> i32 {
        *self.data
    }
}

/// Problem 12: PhantomData pattern
/// Use PhantomData for type safety
pub struct PhantomPattern<T> {
    data: *mut u8,
    _marker: std::marker::PhantomData<T>,
}

impl<T> PhantomPattern<T> {
    pub fn new() -> Self {
        Self {
            data: std::ptr::null_mut(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Problem 13: Null pointer optimization
/// Use null pointer optimization
pub fn null_pointer_optimization() -> Option<Box<i32>> {
    let b = Box::new(42);
    Some(b)
}

/// Problem 14: Niche optimization
/// Use niche optimization
pub fn niche_optimization() -> Option<i32> {
    Some(42)
}

/// Problem 15: Zero-sized type pattern
/// Use zero-sized types
pub struct ZeroSizedType;

pub fn zero_sized_type_pattern() -> usize {
    std::mem::size_of::<ZeroSizedType>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_pun() {
        let f = type_pun_int_to_float(42);
        assert!(f.is_finite());
    }

    #[test]
    fn test_interior_mutability() {
        let x = InteriorMutability::new(42);
        assert_eq!(x.get(), 42);
        x.set(100);
        assert_eq!(x.get(), 100);
    }

    #[test]
    fn test_refcell_pattern() {
        let x = RefCellPattern::new();
        x.push(42);
        assert_eq!(x.get(0), Some(42));
    }

    #[test]
    fn test_unsafe_cell_pattern() {
        let x = UnsafeCellPattern::new(42);
        assert_eq!(x.get(), 42);
        x.set(100);
        assert_eq!(x.get(), 100);
    }

    #[test]
    fn test_static_mut_pattern() {
        assert_eq!(static_mut_pattern(), 1);
    }

    #[test]
    fn test_lazy_init() {
        let x = LazyInit::new();
        assert_eq!(x.get(), 42);
    }

    #[test]
    fn test_transmute_pattern() {
        let x: i32 = -1;
        let y: u32 = transmute_pattern(x);
        assert_eq!(y, u32::MAX);
    }

    #[test]
    fn test_transmute_copy_pattern() {
        let x: i32 = 42;
        let y: u32 = transmute_copy_pattern(&x);
        assert_eq!(y, 42);
    }

    #[test]
    fn test_maybe_uninit_pattern() {
        assert_eq!(maybe_uninit_pattern(), 42);
    }

    #[test]
    fn test_manually_drop_pattern() {
        let mut x = ManuallyDropPattern::new();
        x.push(42);
        let v = x.into_vec();
        assert_eq!(v, vec![42]);
    }

    #[test]
    fn test_pin_pattern() {
        let x = PinPattern::new(42);
        assert_eq!(x.get(), 42);
    }

    #[test]
    fn test_phantom_pattern() {
        let x: PhantomPattern<i32> = PhantomPattern::new();
        assert!(x.data.is_null());
    }

    #[test]
    fn test_null_pointer_optimization() {
        let x = null_pointer_optimization();
        assert!(x.is_some());
    }

    #[test]
    fn test_niche_optimization() {
        let x = niche_optimization();
        assert!(x.is_some());
    }

    #[test]
    fn test_zero_sized_type() {
        assert_eq!(zero_sized_type_pattern(), 0);
    }
}
