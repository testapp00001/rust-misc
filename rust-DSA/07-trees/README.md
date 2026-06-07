# Trees

## Overview
Trees are hierarchical data structures with a root node and children. Binary trees have at most two children per node. Binary Search Trees (BST) have the property that left < root < right.

## Key Concepts

### TreeNode Definition
```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    pub fn new(val: i32) -> Self {
        TreeNode { val, left: None, right: None }
    }
}
```

### Why Rc<RefCell<TreeNode>>?
- `Rc<T>`: Reference counting for shared ownership.
- `RefCell<T>`: Interior mutability (borrow at runtime).
- `Option<Rc<RefCell<TreeNode>>>`: Nullable shared pointer.

```rust
// Create a node
let node = Rc::new(RefCell::new(TreeNode::new(1)));

// Access children
let left = node.borrow().left.clone();
let right = node.borrow().right.clone();

// Modify children
node.borrow_mut().left = Some(Rc::new(RefCell::new(TreeNode::new(2))));
```

## Tree Traversals

### 1. In-order (Left, Root, Right)
```rust
fn inorder(node: &Option<Rc<RefCell<TreeNode>>>, result: &mut Vec<i32>) {
    if let Some(n) = node {
        let borrowed = n.borrow();
        inorder(&borrowed.left, result);
        result.push(borrowed.val);
        inorder(&borrowed.right, result);
    }
}
```

### 2. Pre-order (Root, Left, Right)
```rust
fn preorder(node: &Option<Rc<RefCell<TreeNode>>>, result: &mut Vec<i32>) {
    if let Some(n) = node {
        let borrowed = n.borrow();
        result.push(borrowed.val);
        preorder(&borrowed.left, result);
        preorder(&borrowed.right, result);
    }
}
```

### 3. Post-order (Left, Right, Root)
```rust
fn postorder(node: &Option<Rc<RefCell<TreeNode>>>, result: &mut Vec<i32>) {
    if let Some(n) = node {
        let borrowed = n.borrow();
        postorder(&borrowed.left, result);
        postorder(&borrowed.right, result);
        result.push(borrowed.val);
    }
}
```

### 4. Level-order (BFS)
```rust
fn level_order(root: &Option<Rc<RefCell<TreeNode>>>) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    if root.is_none() { return result; }

    let mut queue = VecDeque::new();
    queue.push_back(root.clone().unwrap());

    while !queue.is_empty() {
        let level_size = queue.len();
        let mut level = Vec::new();

        for _ in 0..level_size {
            let node = queue.pop_front().unwrap();
            let borrowed = node.borrow();
            level.push(borrowed.val);

            if let Some(ref left) = borrowed.left {
                queue.push_back(Rc::clone(left));
            }
            if let Some(ref right) = borrowed.right {
                queue.push_back(Rc::clone(right));
            }
        }

        result.push(level);
    }

    result
}
```

## Common Patterns

### 1. Recursive DFS
```rust
fn dfs(node: &Option<Rc<RefCell<TreeNode>>>) -> i32 {
    match node {
        None => 0,
        Some(n) => {
            let borrowed = n.borrow();
            1 + dfs(&borrowed.left).max(dfs(&borrowed.right))
        }
    }
}
```

### 2. Validate BST
```rust
fn is_valid_bst(node: &Option<Rc<RefCell<TreeNode>>>, min: i64, max: i64) -> bool {
    match node {
        None => true,
        Some(n) => {
            let borrowed = n.borrow();
            let val = borrowed.val as i64;
            val > min && val < max
                && is_valid_bst(&borrowed.left, min, val)
                && is_valid_bst(&borrowed.right, val, max)
        }
    }
}
```

### 3. Lowest Common Ancestor
```rust
fn lca(node: &Option<Rc<RefCell<TreeNode>>>, p: i32, q: i32) -> Option<Rc<RefCell<TreeNode>>> {
    match node {
        None => None,
        Some(n) => {
            let borrowed = n.borrow();
            if borrowed.val == p || borrowed.val == q {
                return Some(Rc::clone(n));
            }

            let left = lca(&borrowed.left, p, q);
            let right = lca(&borrowed.right, p, q);

            match (left, right) {
                (Some(_), Some(_)) => Some(Rc::clone(n)),
                (Some(l), None) => Some(l),
                (None, Some(r)) => Some(r),
                (None, None) => None,
            }
        }
    }
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Invert Tree | Easy | Swap children recursively |
| 2 | Max Depth | Easy | 1 + max(left, right) |
| 3 | Diameter | Easy | left_depth + right_depth |
| 4 | Balanced Tree | Easy | Check height difference |
| 5 | Same Tree | Easy | Compare recursively |
| 6 | Subtree of Another Tree | Easy | Check each node |
| 7 | Lowest Common Ancestor | Medium | Both sides return non-None |
| 8 | Level Order Traversal | Medium | BFS with queue |
| 9 | Validate BST | Medium | Pass min/max bounds |
| 10 | Kth Smallest in BST | Medium | In-order traversal |
| 11 | Build Tree from Traversals | Medium | Preorder root + inorder split |
| 12 | Serialize/Deserialize | Hard | Pre-order with null markers |

## Tips for Rust

1. **`Rc::clone()`**: Cheap reference count increment.
2. **`.borrow()`**: Get shared reference (panics if already mutably borrowed).
3. **`.borrow_mut()`**: Get mutable reference (panics if already borrowed).
4. **`Rc::ptr_eq()`**: Check if two Rc point to the same allocation.
