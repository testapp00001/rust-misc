# Linked List

## Overview
Linked Lists are fundamental data structures where each node contains data and a pointer to the next node. In Rust, linked lists are tricky due to ownership rules.

## Key Concepts

### ListNode Definition
```rust
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    pub fn new(val: i32) -> Self {
        ListNode { val, next: None }
    }
}
```

### Ownership in Rust Linked Lists
```rust
// Box<T>: Heap-allocated pointer with single ownership
// Option<Box<ListNode>>: Nullable pointer

// Take ownership and leave None
let next = node.next.take();

// Get mutable reference
if let Some(ref mut next_node) = node.next {
    // Can modify next_node
}
```

## Common Patterns

### 1. Reverse a Linked List
```rust
fn reverse_list(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    let mut prev = None;
    let mut current = head;

    while let Some(mut node) = current {
        let next = node.next.take();
        node.next = prev;
        prev = Some(node);
        current = next;
    }

    prev
}
```

### 2. Two Pointers (Slow/Fast)
```rust
fn find_middle(head: &Option<Box<ListNode>>) -> &Option<Box<ListNode>> {
    let mut slow = head;
    let mut fast = head;

    while fast.is_some() && fast.as_ref().unwrap().next.is_some() {
        slow = &slow.as_ref().unwrap().next;
        fast = &fast.as_ref().unwrap().next.as_ref().unwrap().next;
    }

    slow
}
```

### 3. Merge Two Sorted Lists
```rust
fn merge_two_lists(
    l1: Option<Box<ListNode>>,
    l2: Option<Box<ListNode>>,
) -> Option<Box<ListNode>> {
    let mut dummy = ListNode::new(0);
    let mut tail = &mut dummy;
    let mut l1 = l1;
    let mut l2 = l2;

    while let (Some(mut n1), Some(mut n2)) = (l1, l2) {
        if n1.val <= n2.val {
            l1 = n1.next.take();
            l2 = Some(n2);
            tail.next = Some(n1);
        } else {
            l2 = n2.next.take();
            l1 = Some(n1);
            tail.next = Some(n2);
        }
        tail = tail.next.as_mut().unwrap();
    }

    tail.next = l1.or(l2);
    dummy.next
}
```

### 4. Detect Cycle (Floyd's Algorithm)
```rust
fn has_cycle(head: &Option<Box<ListNode>>) -> bool {
    let mut slow = head.as_ref();
    let mut fast = head.as_ref();

    while let (Some(s), Some(f)) = (slow, fast) {
        slow = s.next.as_ref();
        fast = f.next.as_ref().and_then(|n| n.next.as_ref());

        if let (Some(s), Some(f)) = (slow, fast) {
            if std::ptr::eq(s, f) {
                return true;
            }
        }
    }

    false
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Reverse Linked List | Easy | Three pointers |
| 2 | Merge Two Sorted Lists | Easy | Dummy head |
| 3 | Linked List Cycle | Easy | Floyd's algorithm |
| 4 | Remove Nth From End | Medium | Two pointers with gap |
| 5 | Reorder List | Medium | Find middle + reverse + merge |
| 6 | Add Two Numbers | Medium | Grade-school addition |
| 7 | Copy List with Random Pointer | Medium | HashMap for mapping |

## Tips for Rust

1. **Use `Box<T>`**: For heap allocation with single ownership.
2. **Use `Option<T>`**: For nullable values (replaces null pointer).
3. **`.take()`**: Takes ownership and leaves None.
4. **`.as_mut()`**: Gets mutable reference to inner value.
5. **Dummy head**: Simplifies edge cases (empty list, remove first node).
