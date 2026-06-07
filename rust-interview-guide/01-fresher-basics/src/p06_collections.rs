/// Problem: Collections
///
/// Master Rust's standard collection types.
///
/// Key Concepts:
/// - Vec (dynamic array)
/// - HashMap (hash table)
/// - String (UTF-8 string)
/// - VecDeque (double-ended queue)
/// - BTreeMap (ordered map)
/// - HashSet (hash set)

use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};

/// Problem 1: Vec basics
/// Create and manipulate a vector
pub fn vec_basics() -> Vec<i32> {
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    v
}

/// Problem 2: Vec with initial values
/// Create a vector with initial values
pub fn vec_with_values() -> Vec<i32> {
    vec![1, 2, 3, 4, 5]
}

/// Problem 3: Vec operations
/// Perform common vector operations
pub fn vec_operations() -> Vec<i32> {
    let mut v = vec![1, 2, 3, 4, 5];

    // Remove last element
    v.pop();

    // Insert at index
    v.insert(0, 0);

    // Remove at index
    v.remove(1);

    v
}

/// Problem 4: Vec iteration
/// Iterate over a vector
pub fn vec_sum(v: &[i32]) -> i32 {
    v.iter().sum()
}

/// Problem 5: Vec transformation
/// Transform a vector using map
pub fn vec_double(v: Vec<i32>) -> Vec<i32> {
    v.into_iter().map(|x| x * 2).collect()
}

/// Problem 6: Vec filtering
/// Filter a vector
pub fn vec_evens(v: Vec<i32>) -> Vec<i32> {
    v.into_iter().filter(|x| x % 2 == 0).collect()
}

/// Problem 7: HashMap basics
/// Create and use a HashMap
pub fn hashmap_basics() -> HashMap<String, i32> {
    let mut map = HashMap::new();
    map.insert("one".to_string(), 1);
    map.insert("two".to_string(), 2);
    map.insert("three".to_string(), 3);
    map
}

/// Problem 8: HashMap lookup
/// Look up values in a HashMap
pub fn hashmap_lookup(map: &HashMap<String, i32>, key: &str) -> Option<i32> {
    map.get(key).copied()
}

/// Problem 9: HashMap entry API
/// Use the entry API for conditional insertion
pub fn hashmap_entry() -> HashMap<String, i32> {
    let mut map = HashMap::new();

    // Insert if not present
    map.entry("one".to_string()).or_insert(1);

    // Increment if present
    *map.entry("one".to_string()).or_insert(0) += 1;

    map
}

/// Problem 10: HashMap frequency count
/// Count frequency of elements
pub fn frequency_count(arr: &[i32]) -> HashMap<i32, usize> {
    let mut map = HashMap::new();
    for &x in arr {
        *map.entry(x).or_insert(0) += 1;
    }
    map
}

/// Problem 11: String basics
/// Create and manipulate strings
pub fn string_basics() -> String {
    let mut s = String::from("Hello");
    s.push(' ');
    s.push_str("World");
    s
}

/// Problem 12: String operations
/// Perform string operations
pub fn string_operations(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

/// Problem 13: String formatting
/// Format strings
pub fn string_format(name: &str, age: u32) -> String {
    format!("{} is {} years old", name, age)
}

/// Problem 14: VecDeque
/// Use a double-ended queue
pub fn vecdeque_basics() -> VecDeque<i32> {
    let mut deque = VecDeque::new();
    deque.push_back(1);
    deque.push_back(2);
    deque.push_front(0);
    deque
}

/// Problem 15: BTreeMap
/// Use an ordered map
pub fn treemap_basics() -> BTreeMap<String, i32> {
    let mut map = BTreeMap::new();
    map.insert("banana".to_string(), 2);
    map.insert("apple".to_string(), 1);
    map.insert("cherry".to_string(), 3);
    map
}

/// Problem 16: HashSet
/// Use a hash set
pub fn hashset_basics() -> HashSet<i32> {
    let mut set = HashSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);
    set.insert(2); // Duplicate, won't be added
    set
}

/// Problem 17: HashSet operations
/// Perform set operations
pub fn set_union(a: &HashSet<i32>, b: &HashSet<i32>) -> HashSet<i32> {
    a.union(b).cloned().collect()
}

/// Problem 18: Vec sorting
/// Sort a vector
pub fn vec_sort(mut v: Vec<i32>) -> Vec<i32> {
    v.sort();
    v
}

/// Problem 19: Vec dedup
/// Remove duplicates from a sorted vector
pub fn vec_dedup(mut v: Vec<i32>) -> Vec<i32> {
    v.sort();
    v.dedup();
    v
}

/// Problem 20: Collect into HashMap
/// Collect pairs into a HashMap
pub fn pairs_to_map(pairs: Vec<(String, i32)>) -> HashMap<String, i32> {
    pairs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_basics() {
        let v = vec_basics();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_vec_with_values() {
        let v = vec_with_values();
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_vec_operations() {
        let v = vec_operations();
        assert_eq!(v, vec![0, 2, 3, 4]);
    }

    #[test]
    fn test_vec_sum() {
        assert_eq!(vec_sum(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_vec_double() {
        let v = vec![1, 2, 3];
        assert_eq!(vec_double(v), vec![2, 4, 6]);
    }

    #[test]
    fn test_vec_evens() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(vec_evens(v), vec![2, 4]);
    }

    #[test]
    fn test_hashmap_basics() {
        let map = hashmap_basics();
        assert_eq!(map.get("one"), Some(&1));
        assert_eq!(map.get("two"), Some(&2));
    }

    #[test]
    fn test_hashmap_lookup() {
        let map = hashmap_basics();
        assert_eq!(hashmap_lookup(&map, "one"), Some(1));
        assert_eq!(hashmap_lookup(&map, "four"), None);
    }

    #[test]
    fn test_hashmap_entry() {
        let map = hashmap_entry();
        assert_eq!(map.get("one"), Some(&2)); // 1 + 1
    }

    #[test]
    fn test_frequency_count() {
        let arr = vec![1, 2, 2, 3, 3, 3];
        let freq = frequency_count(&arr);
        assert_eq!(freq.get(&1), Some(&1));
        assert_eq!(freq.get(&2), Some(&2));
        assert_eq!(freq.get(&3), Some(&3));
    }

    #[test]
    fn test_string_basics() {
        let s = string_basics();
        assert_eq!(s, "Hello World");
    }

    #[test]
    fn test_string_operations() {
        let words = string_operations("Hello World");
        assert_eq!(words, vec!["Hello", "World"]);
    }

    #[test]
    fn test_string_format() {
        let s = string_format("Alice", 30);
        assert_eq!(s, "Alice is 30 years old");
    }

    #[test]
    fn test_vecdeque_basics() {
        let deque = vecdeque_basics();
        assert_eq!(deque[0], 0);
        assert_eq!(deque[1], 1);
        assert_eq!(deque[2], 2);
    }

    #[test]
    fn test_treemap_basics() {
        let map = treemap_basics();
        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_hashset_basics() {
        let set = hashset_basics();
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
    }

    #[test]
    fn test_set_union() {
        let a: HashSet<i32> = vec![1, 2, 3].into_iter().collect();
        let b: HashSet<i32> = vec![3, 4, 5].into_iter().collect();
        let union = set_union(&a, &b);
        assert_eq!(union.len(), 5);
    }

    #[test]
    fn test_vec_sort() {
        let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(vec_sort(v), vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_vec_dedup() {
        let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(vec_dedup(v), vec![1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_pairs_to_map() {
        let pairs = vec![
            ("one".to_string(), 1),
            ("two".to_string(), 2),
        ];
        let map = pairs_to_map(pairs);
        assert_eq!(map.get("one"), Some(&1));
    }
}
