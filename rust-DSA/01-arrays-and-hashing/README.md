# Arrays and Hashing

## Overview
Arrays and Hashing are fundamental to almost every algorithm problem. This topic covers problems that use arrays, hash maps, and hash sets as primary data structures.

## Key Concepts

### Arrays in Rust
- `Vec<T>` is the standard dynamic array.
- Access by index: O(1)
- Push/Pop from end: O(1) amortized
- Insert/Remove from middle: O(n)

```rust
let mut v = Vec::new();
v.push(1);
v.push(2);
let first = v[0]; // 1
let last = v.pop(); // Some(2)
```

### HashMap in Rust
- `HashMap<K, V>` for key-value pairs.
- Insert/Lookup: O(1) average

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key", 42);
if let Some(&value) = map.get("key") {
    println!("{}", value);
}

// Entry API for insert-or-update
*map.entry("counter").or_insert(0) += 1;
```

### HashSet in Rust
- `HashSet<T>` for unique elements.
- Insert/Contains: O(1) average

```rust
use std::collections::HashSet;

let mut set = HashSet::new();
set.insert(1);
if set.contains(&1) {
    println!("Found!");
}
// insert() returns false if element already exists
if !set.insert(1) {
    println!("Already there!");
}
```

## Common Patterns

### 1. Frequency Counting
```rust
let mut freq: HashMap<i32, i32> = HashMap::new();
for &num in &nums {
    *freq.entry(num).or_insert(0) += 1;
}
```

### 2. Two Sum Pattern
```rust
let mut seen: HashMap<i32, usize> = HashMap::new();
for (i, &num) in nums.iter().enumerate() {
    let complement = target - num;
    if let Some(&j) = seen.get(&complement) {
        return vec![j as i32, i as i32];
    }
    seen.insert(num, i);
}
```

### 3. Grouping with Entry API
```rust
let mut groups: HashMap<String, Vec<String>> = HashMap::new();
for s in strings {
    let key = make_key(&s);
    groups.entry(key).or_default().push(s);
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Two Sum | Easy | HashMap lookup |
| 2 | Contains Duplicate | Easy | HashSet insert returns bool |
| 3 | Valid Anagram | Easy | Character frequency count |
| 4 | Group Anagrams | Medium | Sorted key as HashMap key |
| 5 | Top K Frequent Elements | Medium | Bucket sort by frequency |
| 6 | Product of Array Except Self | Medium | Prefix and suffix products |
| 7 | Encode and Decode Strings | Medium | Length-prefix encoding |
| 8 | Longest Consecutive Sequence | Hard | HashSet + sequence start detection |

## Tips for Rust

1. **Use `entry()` API**: It's more idiomatic than checking `contains_key()` first.
2. **`HashMap::with_capacity()`**: Pre-allocate when you know the size.
3. **`iter().enumerate()`**: Gives (index, &value) pairs.
4. **`HashSet::insert()` returns bool**: Use it to detect duplicates efficiently.
