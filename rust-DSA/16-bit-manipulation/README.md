# Bit Manipulation

## Overview
Bit manipulation uses bitwise operators to solve problems efficiently. It's often used for optimization and solving problems with O(1) space.

## Key Concepts

### Bitwise Operators
```rust
let a = 5;  // 0101 in binary
let b = 3;  // 0011 in binary

a & b   // AND:  0001 (1)
a | b   // OR:   0111 (7)
a ^ b   // XOR:  0110 (6)
!a      // NOT:  1010 (-6 in two's complement)
a << 1  // Left shift:  1010 (10)
a >> 1  // Right shift: 0010 (2)
```

### Common Bit Tricks
```rust
// Check if bit is set
let is_set = (n >> i) & 1 == 1;

// Set bit
n |= 1 << i;

// Clear bit
n &= !(1 << i);

// Toggle bit
n ^= 1 << i;

// Get lowest set bit
let lowest = n & (-n);

// Clear lowest set bit
n &= n - 1;
```

## Common Patterns

### 1. Single Number (XOR)
```rust
fn single_number(nums: &[i32]) -> i32 {
    nums.iter().fold(0, |acc, &x| acc ^ x)
}
```

### 2. Number of 1 Bits (Brian Kernighan's)
```rust
fn hamming_weight(mut n: u32) -> i32 {
    let mut count = 0;
    while n != 0 {
        n &= n - 1;  // Clear lowest set bit
        count += 1;
    }
    count
}
```

### 3. Counting Bits (DP)
```rust
fn count_bits(n: i32) -> Vec<i32> {
    let n = n as usize;
    let mut dp = vec![0; n + 1];

    for i in 1..=n {
        dp[i] = dp[i >> 1] + (i & 1) as i32;
    }

    dp
}
```

### 4. Reverse Bits
```rust
fn reverse_bits(mut n: u32) -> u32 {
    let mut result = 0;
    for _ in 0..32 {
        result = (result << 1) | (n & 1);
        n >>= 1;
    }
    result
}
```

### 5. Missing Number (XOR)
```rust
fn missing_number(nums: &[i32]) -> i32 {
    let mut result = nums.len() as i32;
    for (i, &num) in nums.iter().enumerate() {
        result ^= i as i32 ^ num;
    }
    result
}
```

### 6. Sum of Two Integers (Bit Manipulation)
```rust
fn get_sum(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let carry = (a & b) << 1;
        a = a ^ b;
        b = carry;
    }
    a
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Single Number | Easy | XOR cancels pairs |
| 2 | Number of 1 Bits | Easy | Clear lowest set bit |
| 3 | Counting Bits | Easy | DP with bit shift |
| 4 | Reverse Bits | Easy | Process bit by bit |
| 5 | Missing Number | Easy | XOR or math |
| 6 | Sum of Two Integers | Medium | XOR + AND + shift |

## Tips for Rust

1. **`u32` for unsigned**: Use `u32` for bit operations on 32-bit integers.
2. **`i32` for signed**: Be careful with sign extension.
3. **`.count_ones()`**: Built-in method for counting set bits.
4. **`.leading_zeros()` / `.trailing_zeros()`**: Built-in methods.
5. **Overflow**: Use `.wrapping_add()` for safe overflow.
