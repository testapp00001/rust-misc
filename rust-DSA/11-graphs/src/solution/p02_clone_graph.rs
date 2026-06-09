// ============================================================================
// Problem: Clone Graph (LeetCode #133)
// ============================================================================
// Given a reference of a node in a connected undirected graph, return a
// deep copy (clone) of the graph.
//
// ============================================================================
// APPROACH: DFS + HashMap (O(V+E) time, O(V) space)
// ============================================================================
//
// Use a HashMap to map original nodes to cloned nodes:
// 1. If the node is already cloned, return the clone.
// 2. Otherwise, create a clone and recursively clone neighbors.
// ============================================================================

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Node {
    pub val: i32,
    pub neighbors: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(val: i32) -> Self {
        Node {
            val,
            neighbors: Vec::new(),
        }
    }
}

pub fn clone_graph(node: Option<Rc<RefCell<Node>>>) -> Option<Rc<RefCell<Node>>> {
    let node = node?;
    let mut visited: HashMap<i32, Rc<RefCell<Node>>> = HashMap::new();
    Some(dfs(&node, &mut visited))
}

fn dfs(node: &Rc<RefCell<Node>>, visited: &mut HashMap<i32, Rc<RefCell<Node>>>) -> Rc<RefCell<Node>> {
    let val = node.borrow().val;

    if let Some(cloned) = visited.get(&val) {
        return Rc::clone(cloned);
    }

    let clone = Rc::new(RefCell::new(Node::new(val)));
    visited.insert(val, Rc::clone(&clone));

    for neighbor in &node.borrow().neighbors {
        let cloned_neighbor = dfs(neighbor, visited);
        clone.borrow_mut().neighbors.push(cloned_neighbor);
    }

    clone
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_graph(adj_list: &[Vec<i32>]) -> Option<Rc<RefCell<Node>>> {
        if adj_list.is_empty() {
            return None;
        }

        let nodes: Vec<Rc<RefCell<Node>>> = (1..=adj_list.len())
            .map(|i| Rc::new(RefCell::new(Node::new(i as i32))))
            .collect();

        for (i, neighbors) in adj_list.iter().enumerate() {
            for &neighbor_idx in neighbors {
                nodes[i]
                    .borrow_mut()
                    .neighbors
                    .push(Rc::clone(&nodes[neighbor_idx as usize - 1]));
            }
        }

        Some(Rc::clone(&nodes[0]))
    }

    #[test]
    fn test_basic() {
        let graph = create_graph(&[vec![2, 4], vec![1, 3], vec![2, 4], vec![1, 3]]);
        let cloned = clone_graph(graph);
        assert!(cloned.is_some());
        assert_eq!(cloned.unwrap().borrow().val, 1);
    }

    #[test]
    fn test_single() {
        let graph = create_graph(&[vec![]]);
        let cloned = clone_graph(graph);
        assert!(cloned.is_some());
        assert_eq!(cloned.unwrap().borrow().val, 1);
    }

    #[test]
    fn test_empty() {
        assert!(clone_graph(None).is_none());
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