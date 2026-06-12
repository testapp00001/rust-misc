//! Entry point for the capstone distributed database demo node.

use capstone_distributed_database::consensus::raft::RaftNode;
use capstone_distributed_database::storage::engine::StorageEngine;
use capstone_distributed_database::clock::hlc::HLC;

fn main() {
    println!("Capstone Distributed Database");
    println!("Initializing node...");

    let mut engine = StorageEngine::new();
    engine.put("greeting".to_string(), "Hello, distributed world!".to_string());
    println!("Stored greeting.");

    let mut hlc = HLC::new(1);
    let ts = hlc.now();
    println!("HLC timestamp: {}", ts);

    let raft = RaftNode::new(1, vec![]);
    println!("Raft node created with id: {}", raft.id);

    println!("Node ready.");
}
