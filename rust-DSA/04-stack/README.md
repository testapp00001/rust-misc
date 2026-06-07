# Stack

## Overview
Stack is a LIFO (Last In, First Out) data structure. In Rust, `Vec<T>` can be used as a stack with `push()` and `pop()`.

## Key Concepts

### Stack Operations
```rust
let mut stack: Vec<i32> = Vec::new();

// Push
stack.push(1);
stack.push(2);

// Pop
let top = stack.pop(); // Some(2)

// Peek
let top = stack.last(); // Some(&1)

// Check empty
if stack.is_empty() { /* ... */ }
```

### When to Use Stack
1. **Matching/balancing**: Parentheses, brackets.
2. **Expression evaluation**: Postfix, infix to postfix.
3. **Monotonic stack**: Next greater/smaller element.
4. **DFS traversal**: Explicit stack instead of recursion.
5. **Undo operations**: Browser history, text editor.

## Common Patterns

### 1. Valid Parentheses
```rust
fn is_valid(s: &str) -> bool {
    let mut stack = Vec::new();

    for c in s.chars() {
        match c {
            '(' | '{' | '[' => stack.push(c),
            ')' => { if stack.pop() != Some('(') { return false; } }
            '}' => { if stack.pop() != Some('{') { return false; } }
            ']' => { if stack.pop() != Some('[') { return false; } }
            _ => {}
        }
    }

    stack.is_empty()
}
```

### 2. Monotonic Stack (Next Greater Element)
```rust
fn next_greater(nums: &[i32]) -> Vec<i32> {
    let mut result = vec![-1; nums.len()];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..nums.len() {
        while !stack.is_empty() && nums[i] > nums[*stack.last().unwrap()] {
            result[stack.pop().unwrap()] = nums[i];
        }
        stack.push(i);
    }

    result
}
```

### 3. Min Stack
```rust
struct MinStack {
    stack: Vec<i32>,
    min_stack: Vec<i32>,
}

impl MinStack {
    fn push(&mut self, val: i32) {
        self.stack.push(val);
        let min_val = val.min(*self.min_stack.last().unwrap_or(&val));
        self.min_stack.push(min_val);
    }

    fn pop(&mut self) {
        self.stack.pop();
        self.min_stack.pop();
    }

    fn get_min(&self) -> i32 {
        *self.min_stack.last().unwrap()
    }
}
```

### 4. Evaluate Reverse Polish Notation
```rust
fn eval_rpn(tokens: &[&str]) -> i32 {
    let mut stack = Vec::new();

    for token in tokens {
        match token {
            "+" | "-" | "*" | "/" => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let result = match *token {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => a / b,
                    _ => unreachable!(),
                };
                stack.push(result);
            }
            _ => stack.push(token.parse().unwrap()),
        }
    }

    stack.pop().unwrap()
}
```

## Monotonic Stack

A monotonic stack maintains elements in sorted order. It's useful for:
- **Next greater element**: Decreasing stack (pop when current > top)
- **Next smaller element**: Increasing stack (pop when current < top)
- **Largest rectangle in histogram**: Increasing stack

```rust
// Daily Temperatures (next warmer day)
fn daily_temperatures(temps: &[i32]) -> Vec<i32> {
    let mut result = vec![0; temps.len()];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..temps.len() {
        while !stack.is_empty() && temps[i] > temps[*stack.last().unwrap()] {
            let j = stack.pop().unwrap();
            result[j] = (i - j) as i32;
        }
        stack.push(i);
    }

    result
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Valid Parentheses | Easy | Match opening/closing |
| 2 | Min Stack | Medium | Two stacks |
| 3 | Evaluate Reverse Polish | Medium | Stack for operands |
| 4 | Daily Temperatures | Medium | Monotonic stack |
| 5 | Car Fleet | Medium | Sort + stack |
| 6 | Largest Rectangle in Histogram | Hard | Monotonic stack |

## Tips for Rust

1. **`Vec` as stack**: Use `push()` and `pop()`.
2. **`last()` returns `Option<&T>`**: Use `.unwrap_or()` for defaults.
3. **`pop()` returns `Option<T>`**: Handle `None` case.
4. **Pattern matching**: Use `match` for cleaner code.
