// ============================================================================
// Problem: Min Stack (LeetCode #155)
// ============================================================================
// Design a stack that supports push, pop, top, and retrieving the minimum
// element in constant time.
//
// Implement the MinStack class:
// - MinStack() initializes the stack object.
// - push(val) pushes the element val onto the stack.
// - pop() removes the element on the top of the stack.
// - top() gets the top element.
// - get_min() retrieves the minimum element in the stack.
//
// ============================================================================
// APPROACH: Two Stacks (O(1) for all operations)
// ============================================================================
//
// Use two stacks:
// 1. Main stack: stores all values.
// 2. Min stack: stores the minimum value at each level.
//
// When pushing: push to main stack, and push min(val, current_min) to min stack.
// When popping: pop from both stacks.
// get_min: return top of min stack.
//
// This keeps the min stack in sync with the main stack.
// ============================================================================

pub struct MinStack {
    stack: Vec<i32>,
    min_stack: Vec<i32>,
}

impl MinStack {
    pub fn new() -> Self {
        MinStack {
            stack: Vec::new(),
            min_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, val: i32) {
        self.stack.push(val);
        let min_val = if self.min_stack.is_empty() {
            val
        } else {
            val.min(*self.min_stack.last().unwrap())
        };
        self.min_stack.push(min_val);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
        self.min_stack.pop();
    }

    pub fn top(&self) -> i32 {
        *self.stack.last().unwrap()
    }

    pub fn get_min(&self) -> i32 {
        *self.min_stack.last().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut stack = MinStack::new();
        stack.push(-2);
        stack.push(0);
        stack.push(-3);
        assert_eq!(stack.get_min(), -3);
        stack.pop();
        assert_eq!(stack.top(), 0);
        assert_eq!(stack.get_min(), -2);
    }

    #[test]
    fn test_single() {
        let mut stack = MinStack::new();
        stack.push(1);
        assert_eq!(stack.top(), 1);
        assert_eq!(stack.get_min(), 1);
    }

    #[test]
    fn test_ascending() {
        let mut stack = MinStack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.get_min(), 1);
        stack.pop();
        assert_eq!(stack.get_min(), 1);
    }

    #[test]
    fn test_descending() {
        let mut stack = MinStack::new();
        stack.push(3);
        stack.push(2);
        stack.push(1);
        assert_eq!(stack.get_min(), 1);
        stack.pop();
        assert_eq!(stack.get_min(), 2);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}