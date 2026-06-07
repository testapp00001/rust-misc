// ============================================================================
// Problem: Serialize and Deserialize Binary Tree (LeetCode #297)
// ============================================================================
// Design an algorithm to serialize and deserialize a binary tree.
//
// Serialization: Convert tree to a string.
// Deserialization: Convert string back to the original tree.
//
// ============================================================================
// APPROACH: Pre-order Traversal with Null Markers
// ============================================================================
//
// Serialize: Pre-order traversal, using "#" for null nodes.
//   "1,2,#,#,3,4,#,#,5,#,#"
//
// Deserialize: Read values one by one, building the tree pre-order.
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use super::tree_node::TreeNode;

type TreeLink = Option<Rc<RefCell<TreeNode>>>;

pub struct Codec;

impl Codec {
    pub fn new() -> Self {
        Codec
    }

    pub fn serialize(&self, root: &TreeLink) -> String {
        let mut result = String::new();
        self.serialize_helper(root, &mut result);
        result
    }

    fn serialize_helper(&self, node: &TreeLink, result: &mut String) {
        if let Some(n) = node {
            let borrowed = n.borrow();
            result.push_str(&borrowed.val.to_string());
            result.push(',');
            self.serialize_helper(&borrowed.left, result);
            self.serialize_helper(&borrowed.right, result);
        } else {
            result.push_str("#,");
        }
    }

    pub fn deserialize(&self, data: &str) -> TreeLink {
        let mut vals: Vec<&str> = data.trim_end_matches(',').split(',').collect();
        vals.reverse(); // So we can pop from the front efficiently
        self.deserialize_helper(&mut vals)
    }

    fn deserialize_helper(&self, vals: &mut Vec<&str>) -> TreeLink {
        let val = vals.pop()?;
        if val == "#" {
            return None;
        }

        let node = Rc::new(RefCell::new(TreeNode::new(val.parse().unwrap())));
        node.borrow_mut().left = self.deserialize_helper(vals);
        node.borrow_mut().right = self.deserialize_helper(vals);

        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tree_node::TreeNode;

    #[test]
    fn test_basic() {
        let codec = Codec::new();
        let root = TreeNode::from_level_order(&[
            Some(1), Some(2), Some(3), None, None, Some(4), Some(5),
        ]);
        let serialized = codec.serialize(&root);
        let deserialized = codec.deserialize(&serialized);
        assert_eq!(
            TreeNode::to_level_order(&root),
            TreeNode::to_level_order(&deserialized)
        );
    }

    #[test]
    fn test_empty() {
        let codec = Codec::new();
        let root: TreeLink = None;
        let serialized = codec.serialize(&root);
        let deserialized = codec.deserialize(&serialized);
        assert_eq!(
            TreeNode::to_level_order(&root),
            TreeNode::to_level_order(&deserialized)
        );
    }

    #[test]
    fn test_single() {
        let codec = Codec::new();
        let root = TreeNode::from_level_order(&[Some(42)]);
        let serialized = codec.serialize(&root);
        let deserialized = codec.deserialize(&serialized);
        assert_eq!(
            TreeNode::to_level_order(&root),
            TreeNode::to_level_order(&deserialized)
        );
    }
}
