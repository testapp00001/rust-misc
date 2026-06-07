//! # Zero-Cost Abstractions
//!
//! Rust's zero-cost abstractions mean that high-level constructs compile down
//! to the same machine code as hand-written low-level code. This module
//! demonstrates how iterators, traits, generics, and compile-time computation
//! achieve this.
//!
//! ## Key Concepts
//! - **Iterators vs loops**: Iterator chains compile to the same code as manual loops
//! - **Monomorphization**: Generic code is specialized for each concrete type
//! - **Compile-time computation**: `const fn` and const generics
//! - **Trait dispatch**: Static dispatch via generics, dynamic dispatch via trait objects

/// Sum using a manual loop.
pub fn sum_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &val in data {
        sum += val;
    }
    sum
}

/// Sum using iterators.
/// The compiler optimizes this to the same assembly as sum_loop.
pub fn sum_iterator(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Sum using iterator with fold (explicit accumulator).
pub fn sum_fold(data: &[i32]) -> i32 {
    data.iter().fold(0, |acc, &x| acc + x)
}

/// Chained iterators: filter + map + sum.
pub fn sum_even_squares(data: &[i32]) -> i32 {
    data.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .sum()
}

/// Manual equivalent of sum_even_squares.
pub fn sum_even_squares_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &val in data {
        if val % 2 == 0 {
            sum += val * val;
        }
    }
    sum
}

/// Demonstrates monomorphization: this generic function is compiled
/// separately for each type it's used with, producing optimal code.
pub fn find_max<T: PartialOrd>(data: &[T]) -> Option<&T> {
    data.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

/// A generic accumulator that works with any numeric type.
pub fn accumulate<T>(data: &[T]) -> T
where
    T: std::ops::Add<Output = T> + Copy + Default,
{
    let mut total = T::default();
    for &val in data {
        total = total + val;
    }
    total
}

/// Const generics: a fixed-size ring buffer whose capacity is a compile-time constant.
pub struct RingBuffer<T, const N: usize> {
    data: [std::mem::MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        RingBuffer {
            data: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let tail = (self.head + self.len) % N;
        let old = if self.len == N {
            let old = unsafe { self.data[self.head].assume_init_read() };
            self.head = (self.head + 1) % N;
            Some(old)
        } else {
            self.len += 1;
            None
        };
        self.data[tail] = std::mem::MaybeUninit::new(value);
        old
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = unsafe { self.data[self.head].assume_init_read() };
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            let idx = (self.head + i) % N;
            unsafe {
                self.data[idx].assume_init_drop();
            }
        }
    }
}

/// Const function: compute a lookup table at compile time.
pub const fn fibonacci_table<const N: usize>() -> [u64; N] {
    let mut table = [0u64; N];
    if N > 0 {
        table[0] = 0;
    }
    if N > 1 {
        table[1] = 1;
    }
    let mut i = 2;
    while i < N {
        table[i] = table[i - 1] + table[i - 2];
        i += 1;
    }
    table
}

/// Compile-time factorial.
pub const fn factorial(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

/// A trait with default implementations that get monomorphized.
pub trait Processor {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Self::Output;

    /// Default implementation using the processor.
    fn process_all(&self, inputs: Vec<Self::Input>) -> Vec<Self::Output> {
        inputs.into_iter().map(|i| self.process(i)).collect()
    }
}

pub struct Doubler;
impl Processor for Doubler {
    type Input = i32;
    type Output = i32;

    fn process(&self, input: i32) -> i32 {
        input * 2
    }
}

pub struct Stringifier;
impl Processor for Stringifier {
    type Input = i32;
    type Output = String;

    fn process(&self, input: i32) -> String {
        input.to_string()
    }
}

/// Demonstrates that trait objects (dyn) vs generics (impl) produce
/// different performance characteristics.
pub fn process_generic<P: Processor>(processor: &P, items: Vec<P::Input>) -> Vec<P::Output> {
    processor.process_all(items)
}

pub fn process_dynamic(
    processor: &dyn Processor<Input = i32, Output = i32>,
    items: Vec<i32>,
) -> Vec<i32> {
    processor.process_all(items)
}

/// Type-level programming: encode values in types for compile-time checking.
pub struct Meter;
pub struct Second;

pub struct Quantity<Unit> {
    pub value: f64,
    _unit: std::marker::PhantomData<Unit>,
}

impl<Unit> Quantity<Unit> {
    pub const fn new(value: f64) -> Self {
        Quantity {
            value,
            _unit: std::marker::PhantomData,
        }
    }
}

impl<Unit> std::ops::Add for Quantity<Unit> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Quantity::new(self.value + rhs.value)
    }
}

/// You CANNOT add meters and seconds — this is a compile error:
/// ```compile_fail
/// let d = Quantity::<Meter>::new(1.0) + Quantity::<Second>::new(1.0);
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_methods_agree() {
        let data: Vec<i32> = (-100..100).collect();
        assert_eq!(sum_loop(&data), sum_iterator(&data));
        assert_eq!(sum_loop(&data), sum_fold(&data));
    }

    #[test]
    fn test_sum_even_squares() {
        let data: Vec<i32> = (1..=10).collect();
        let result = sum_even_squares(&data);
        let expected = sum_even_squares_loop(&data);
        assert_eq!(result, expected);
        // 2^2 + 4^2 + 6^2 + 8^2 + 10^2 = 4 + 16 + 36 + 64 + 100 = 220
        assert_eq!(result, 220);
    }

    #[test]
    fn test_find_max() {
        let ints = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(find_max(&ints), Some(&9));

        let floats = vec![1.5, 2.7, 0.3];
        assert_eq!(find_max(&floats), Some(&2.7));

        let empty: Vec<i32> = vec![];
        assert_eq!(find_max(&empty), None);
    }

    #[test]
    fn test_accumulate() {
        let ints: Vec<i32> = vec![1, 2, 3, 4, 5];
        assert_eq!(accumulate(&ints), 15);

        let floats: Vec<f64> = vec![1.0, 2.0, 3.0];
        assert!((accumulate(&floats) - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buf = RingBuffer::<i32, 4>::new();
        assert_eq!(buf.capacity(), 4);
        assert!(buf.is_empty());

        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);
        assert!(buf.is_full());

        // Overwrite oldest
        let old = buf.push(5);
        assert_eq!(old, Some(1));

        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_ring_buffer_partial() {
        let mut buf = RingBuffer::<i32, 3>::new();
        buf.push(10);
        assert_eq!(buf.pop(), Some(10));
        assert!(buf.is_empty());
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_fibonacci_table() {
        const FIB: [u64; 10] = fibonacci_table::<10>();
        assert_eq!(FIB[0], 0);
        assert_eq!(FIB[1], 1);
        assert_eq!(FIB[2], 1);
        assert_eq!(FIB[3], 2);
        assert_eq!(FIB[4], 3);
        assert_eq!(FIB[5], 5);
        assert_eq!(FIB[6], 8);
        assert_eq!(FIB[7], 13);
        assert_eq!(FIB[8], 21);
        assert_eq!(FIB[9], 34);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_processor_doubler() {
        let doubler = Doubler;
        let result = process_generic(&doubler, vec![1, 2, 3]);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_processor_stringifier() {
        let stringifier = Stringifier;
        let result = process_generic(&stringifier, vec![1, 2, 3]);
        assert_eq!(result, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_process_dynamic() {
        let doubler = Doubler;
        let result = process_dynamic(&doubler, vec![1, 2, 3]);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_quantity_add() {
        let d1 = Quantity::<Meter>::new(3.0);
        let d2 = Quantity::<Meter>::new(4.0);
        let total = d1 + d2;
        assert!((total.value - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_const_fib_table_sizes() {
        const FIB5: [u64; 5] = fibonacci_table::<5>();
        assert_eq!(FIB5.len(), 5);

        const FIB0: [u64; 0] = fibonacci_table::<0>();
        assert_eq!(FIB0.len(), 0);

        const FIB1: [u64; 1] = fibonacci_table::<1>();
        assert_eq!(FIB1, [0]);
    }

    #[test]
    fn test_iterator_chain_compiles() {
        // This test just verifies the iterator chain produces correct results
        let data: Vec<i32> = (0..100).collect();
        let result: i32 = data
            .iter()
            .filter(|&&x| x % 3 == 0)
            .map(|&x| x * x)
            .take(5)
            .sum();
        // First 5 multiples of 3: 0, 3, 6, 9, 12 -> squares: 0, 9, 36, 81, 144
        assert_eq!(result, 0 + 9 + 36 + 81 + 144);
    }
}
