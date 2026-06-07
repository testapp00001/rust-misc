# Math and Geometry

## Overview
Math and Geometry problems involve mathematical calculations, matrix operations, and geometric algorithms.

## Key Concepts

### Matrix Operations
```rust
// Rotate 90° clockwise: transpose + reverse rows
fn rotate(matrix: &mut Vec<Vec<i32>>) {
    let n = matrix.len();
    // Transpose
    for i in 0..n {
        for j in i+1..n {
            let temp = matrix[i][j];
            matrix[i][j] = matrix[j][i];
            matrix[j][i] = temp;
        }
    }
    // Reverse rows
    for row in matrix.iter_mut() {
        row.reverse();
    }
}
```

### Fast Exponentiation
```rust
fn pow(mut x: f64, mut n: i32) -> f64 {
    if n < 0 {
        x = 1.0 / x;
        n = -n;
    }

    let mut result = 1.0;
    while n > 0 {
        if n % 2 == 1 {
            result *= x;
        }
        x *= x;
        n /= 2;
    }

    result
}
```

## Common Patterns

### 1. Spiral Matrix
```rust
fn spiral_order(matrix: Vec<Vec<i32>>) -> Vec<i32> {
    let mut result = Vec::new();
    if matrix.is_empty() { return result; }

    let mut top = 0;
    let mut bottom = matrix.len() - 1;
    let mut left = 0;
    let mut right = matrix[0].len() - 1;

    while top <= bottom && left <= right {
        for c in left..=right { result.push(matrix[top][c]); }
        top += 1;

        for r in top..=bottom { result.push(matrix[r][right]); }
        right -= 1;

        if top <= bottom {
            for c in (left..=right).rev() { result.push(matrix[bottom][c]); }
            bottom -= 1;
        }

        if left <= right {
            for r in (top..=bottom).rev() { result.push(matrix[r][left]); }
            left += 1;
        }
    }

    result
}
```

### 2. Set Matrix Zeroes
```rust
fn set_zeroes(matrix: &mut Vec<Vec<i32>>) {
    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut first_row_zero = false;

    // Check first row
    for c in 0..cols {
        if matrix[0][c] == 0 { first_row_zero = true; break; }
    }

    // Use first row/column as markers
    for r in 1..rows {
        for c in 0..cols {
            if matrix[r][c] == 0 {
                matrix[r][0] = 0;
                matrix[0][c] = 0;
            }
        }
    }

    // Set zeros based on markers
    for r in 1..rows {
        for c in 1..cols {
            if matrix[r][0] == 0 || matrix[0][c] == 0 {
                matrix[r][c] = 0;
            }
        }
    }

    // Handle first column
    if matrix[0][0] == 0 {
        for r in 0..rows { matrix[r][0] = 0; }
    }

    // Handle first row
    if first_row_zero {
        for c in 0..cols { matrix[0][c] = 0; }
    }
}
```

### 3. Happy Number (Cycle Detection)
```rust
fn is_happy(n: i32) -> bool {
    let mut slow = n;
    let mut fast = n;

    loop {
        slow = digit_square_sum(slow);
        fast = digit_square_sum(digit_square_sum(fast));

        if fast == 1 { return true; }
        if slow == fast { return false; }
    }
}

fn digit_square_sum(mut n: i32) -> i32 {
    let mut sum = 0;
    while n > 0 {
        let digit = n % 10;
        sum += digit * digit;
        n /= 10;
    }
    sum
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Rotate Image | Medium | Transpose + reverse |
| 2 | Spiral Matrix | Medium | Layer by layer |
| 3 | Set Matrix Zeroes | Medium | Use first row/col as markers |
| 4 | Happy Number | Easy | Cycle detection |
| 5 | Plus One | Easy | Handle carry |
| 6 | Pow(x, n) | Medium | Fast exponentiation |
| 7 | Multiply Strings | Medium | Grade school multiplication |
| 8 | Detect Squares | Medium | Count by diagonal |

## Tips for Rust

1. **`i32` overflow**: Use `i64` for intermediate calculations.
2. **`f64` for floating point**: Use `f64::EPSILON` for comparison.
3. **Matrix indexing**: `matrix[row][col]`.
4. **Edge cases**: Empty matrix, single element, negative numbers.
