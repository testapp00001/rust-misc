//! # Exercise: Mini Blockchain
//!
//! ## Theory
//!
//! A **blockchain** is a distributed ledger where transactions are organized
//! into **blocks**, each linked to the previous block via a cryptographic hash.
//! This creates an immutable, append-only data structure.
//!
//! In a Byzantine fault-tolerant blockchain:
//! - Nodes maintain a copy of the blockchain.
//! - Transactions are grouped into blocks by a leader (or miner).
//! - The block must be validated and committed by a quorum of nodes.
//! - The chain is secured by the hash linkage: altering any block invalidates
//!   all subsequent blocks.
//!
//! Key properties:
//! 1. **Immutability**: Once a block is committed, it cannot be altered without
//!    breaking the hash chain.
//! 2. **Decentralization**: No single node controls the ledger.
//! 3. **BFT Consensus**: The system tolerates Byzantine faults using a protocol
//!    like PBFT or Tendermint.
//! 4. **Finality**: In BFT-based blockchains, committed blocks are final
//!    (unlike Bitcoin's probabilistic finality).
//!
//! ## Proof / Intuition
//!
//! The hash chain provides integrity: if an attacker modifies block B_i, then
//! the hash of B_i changes, which invalidates the "previous hash" pointer in
//! B_{i+1}, and so on. An attacker must re-mine (or re-commit) all subsequent
//! blocks, which requires controlling a quorum of the network.
//!
//! With BFT consensus (n >= 3f + 1), an attacker needs f+1 honest nodes to
//! collude to break the chain, which is impossible if fewer than n/3 nodes are
//! Byzantine.
//!
//! ## Implementation Task
//!
//! Implement:
//!
//! - `Transaction` struct with: from, to, amount, hash
//! - `Block` struct with: index, transactions, previous_hash, hash, nonce
//! - `Blockchain` struct with: chain, pending_transactions, difficulty
//! - Methods:
//!   - `Transaction::new(from, to, amount)` - create a transaction
//!   - `Block::new(index, transactions, previous_hash)` - create and hash a block
//!   - `Block::compute_hash(&self) -> String` - compute SHA-256-like hash
//!   - `Blockchain::new(difficulty)` - create a blockchain with a genesis block
//!   - `Blockchain::add_block(transactions)` - mine and add a new block
//!   - `Blockchain::validate()` - validate the entire chain
//!   - `Blockchain::latest_block()` - get the last block
//!
//! ## Verification
//!
//! - Verify genesis block is created correctly
//! - Verify blocks are properly linked via hashes
//! - Verify chain validation passes for a valid chain
//! - Verify chain validation fails when a block is tampered with

use std::fmt;

/// A transaction representing a transfer of value between addresses.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// The sender's address.
    pub from: String,
    /// The recipient's address.
    pub to: String,
    /// The amount transferred.
    pub amount: f64,
    /// Hash of this transaction.
    pub hash: String,
}

impl Transaction {
    /// Create a new transaction with a computed hash.
    pub fn new(from: &str, to: &str, amount: f64) -> Self {
        let content = format!("{from}{to}{amount}");
        let hash = simple_hash(&content);
        Self {
            from: from.to_string(),
            to: to.to_string(),
            amount,
            hash,
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}: {:.2} (hash: {})",
            self.from, self.to, self.amount, self.hash
        )
    }
}

/// A block in the blockchain.
#[derive(Debug, Clone)]
pub struct Block {
    /// The block's position in the chain (0 = genesis).
    pub index: usize,
    /// Transactions included in this block.
    pub transactions: Vec<Transaction>,
    /// Hash of the previous block in the chain.
    pub previous_hash: String,
    /// This block's hash (computed from all block contents).
    pub hash: String,
    /// Nonce used for proof-of-work mining.
    pub nonce: u64,
    /// Timestamp of block creation (simplified as a counter).
    pub timestamp: u64,
}

impl Block {
    /// Create a new block and compute its hash.
    pub fn new(
        index: usize,
        transactions: Vec<Transaction>,
        previous_hash: String,
        difficulty: usize,
    ) -> Self {
        let mut block = Block {
            index,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            timestamp: index as u64,
        };
        block.mine(difficulty);
        block
    }

    /// Compute the hash of this block's contents.
    ///
    /// The hash includes the index, previous hash, transactions, nonce,
    /// and timestamp.
    pub fn compute_hash(&self) -> String {
        let tx_hashes: Vec<String> = self
            .transactions
            .iter()
            .map(|tx| tx.hash.clone())
            .collect();
        let content = format!(
            "{}{}{}{}{}",
            self.index,
            self.previous_hash,
            tx_hashes.join(""),
            self.nonce,
            self.timestamp
        );
        simple_hash(&content)
    }

    /// Mine the block by finding a nonce that produces a hash with the
    /// required number of leading zeros.
    fn mine(&mut self, difficulty: usize) {
        let prefix = "0".repeat(difficulty);
        loop {
            self.hash = self.compute_hash();
            if self.hash.starts_with(&prefix) {
                break;
            }
            self.nonce += 1;
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block #{} [hash: {}, prev: {}, txs: {}, nonce: {}]",
            self.index,
            &self.hash[..8],
            &self.previous_hash[..8],
            self.transactions.len(),
            self.nonce
        )
    }
}

/// A simple blockchain with proof-of-work mining.
#[derive(Debug)]
pub struct Blockchain {
    /// The chain of committed blocks.
    pub chain: Vec<Block>,
    /// Transactions waiting to be included in a block.
    pub pending_transactions: Vec<Transaction>,
    /// Mining difficulty (number of leading zeros in block hash).
    pub difficulty: usize,
}

impl Blockchain {
    /// Create a new blockchain with a genesis block.
    pub fn new(difficulty: usize) -> Self {
        let genesis = Block::new(0, Vec::new(), "0".to_string(), difficulty);
        let chain = vec![genesis];
        Self {
            chain,
            pending_transactions: Vec::new(),
            difficulty,
        }
    }

    /// Get the latest block in the chain.
    pub fn latest_block(&self) -> &Block {
        self.chain
            .last()
            .expect("blockchain should always have at least the genesis block")
    }

    /// Add a transaction to the pending pool.
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    /// Mine a new block containing all pending transactions and add it to the chain.
    ///
    /// Returns a reference to the newly created block.
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> &Block {
        let previous_hash = self.latest_block().hash.clone();
        let index = self.chain.len();
        let block = Block::new(index, transactions, previous_hash, self.difficulty);
        self.chain.push(block);
        self.pending_transactions.clear();
        self.chain.last().unwrap()
    }

    /// Validate the entire blockchain.
    ///
    /// Checks:
    /// 1. The genesis block is valid.
    /// 2. Each block's previous_hash matches the hash of the block before it.
    /// 3. Each block's hash is correctly computed.
    /// 4. Each block's hash satisfies the difficulty requirement.
    pub fn validate(&self) -> bool {
        if self.chain.is_empty() {
            return false;
        }

        let prefix = "0".repeat(self.difficulty);

        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // Check previous hash linkage
            if current.previous_hash != previous.hash {
                return false;
            }

            // Check hash is correctly computed
            if current.hash != current.compute_hash() {
                return false;
            }

            // Check difficulty
            if !current.hash.starts_with(&prefix) {
                return false;
            }
        }

        true
    }

    /// Tamper with a block's transaction amount (for testing validation).
    ///
    /// Modifies the transaction amount and recomputes the transaction hash,
    /// which changes the block's hash, causing validation to fail.
    pub fn tamper_block(&mut self, block_index: usize, tx_index: usize, new_amount: f64) {
        if block_index < self.chain.len() {
            if let Some(block) = self.chain.get_mut(block_index) {
                if tx_index < block.transactions.len() {
                    block.transactions[tx_index].amount = new_amount;
                    // Recompute the transaction hash to reflect the tampered amount
                    let tx = &mut block.transactions[tx_index];
                    let content = format!("{}{}{}", tx.from, tx.to, tx.amount);
                    tx.hash = simple_hash(&content);
                }
            }
        }
    }
}

/// Compute a simple hash string from input data.
///
/// This is a simplified hash function for demonstration purposes.
/// In a real blockchain, SHA-256 or similar would be used.
fn simple_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    // Format as hex string, zero-padded to 16 characters
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_block_created() {
        let blockchain = Blockchain::new(2);
        assert_eq!(blockchain.chain.len(), 1, "should have genesis block");
        assert_eq!(blockchain.latest_block().index, 0);
        assert!(
            blockchain.latest_block().transactions.is_empty(),
            "genesis block should have no transactions"
        );
        assert_eq!(
            blockchain.latest_block().previous_hash,
            "0",
            "genesis block's previous hash should be '0'"
        );
    }

    #[test]
    fn blocks_linked_via_hashes() {
        let mut blockchain = Blockchain::new(2);

        let tx1 = Transaction::new("alice", "bob", 10.0);
        blockchain.add_block(vec![tx1]);

        let tx2 = Transaction::new("bob", "charlie", 5.0);
        blockchain.add_block(vec![tx2]);

        assert_eq!(blockchain.chain.len(), 3, "should have 3 blocks (genesis + 2)");

        // Verify linkage
        assert_eq!(
            blockchain.chain[1].previous_hash,
            blockchain.chain[0].hash,
            "block 1's previous hash should match block 0's hash"
        );
        assert_eq!(
            blockchain.chain[2].previous_hash,
            blockchain.chain[1].hash,
            "block 2's previous hash should match block 1's hash"
        );
    }

    #[test]
    fn chain_validation_passes_for_valid_chain() {
        let mut blockchain = Blockchain::new(2);

        let tx1 = Transaction::new("alice", "bob", 10.0);
        blockchain.add_block(vec![tx1]);

        let tx2 = Transaction::new("bob", "charlie", 5.0);
        blockchain.add_block(vec![tx2]);

        assert!(
            blockchain.validate(),
            "valid chain should pass validation"
        );
    }

    #[test]
    fn chain_validation_fails_when_tampered() {
        let mut blockchain = Blockchain::new(2);

        let tx1 = Transaction::new("alice", "bob", 10.0);
        blockchain.add_block(vec![tx1]);

        let tx2 = Transaction::new("bob", "charlie", 5.0);
        blockchain.add_block(vec![tx2]);

        assert!(blockchain.validate());

        // Tamper with block 1's transaction
        blockchain.tamper_block(1, 0, 999.0);

        assert!(
            !blockchain.validate(),
            "tampered chain should fail validation"
        );
    }

    #[test]
    fn transaction_hash_is_deterministic() {
        let tx1 = Transaction::new("alice", "bob", 10.0);
        let tx2 = Transaction::new("alice", "bob", 10.0);
        assert_eq!(
            tx1.hash, tx2.hash,
            "same transaction should produce same hash"
        );

        let tx3 = Transaction::new("alice", "bob", 20.0);
        assert_ne!(
            tx1.hash, tx3.hash,
            "different transactions should produce different hashes"
        );
    }
}
