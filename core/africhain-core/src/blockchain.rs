use serde::{Deserialize, Serialize};
use std::fmt;

use chrono::Utc;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: usize,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {} ({})", self.from, self.to, self.amount)
    }
}

impl Block {
    pub fn new(index: u64, previous_hash: &str, transactions: Vec<Transaction>) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Self {
            index,
            timestamp,
            transactions,
            previous_hash: previous_hash.to_string(),
            hash: String::new(),
            nonce: 0,
        };

        block.hash = block.compute_hash();
        block
    }

    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let payload = format!(
            "{}|{}|{}|{}|{}",
            self.index,
            self.timestamp,
            self.previous_hash,
            self.nonce,
            self.transactions
                .iter()
                .map(|tx| format!("{}:{}:{}:{}", tx.from, tx.to, tx.amount, tx.nonce))
                .collect::<Vec<_>>()
                .join(";")
        );

        hasher.update(payload);
        format!("{:x}", hasher.finalize())
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::new(0, "0", vec![]);
        Self {
            chain: vec![genesis],
            pending_transactions: vec![],
            difficulty: 2,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn mine_pending_transactions(&mut self, miner: &str) {
        let transactions = self.pending_transactions.clone();
        let previous_hash = self.last_hash();
        let mut block = Block::new((self.chain.len() as u64), &previous_hash, transactions);

        block.nonce = 0;
        while !block.hash.starts_with(&"0".repeat(self.difficulty)) {
            block.nonce += 1;
            block.hash = block.compute_hash();
        }

        self.chain.push(block);
        self.pending_transactions.clear();

        let _ = miner;
    }

    pub fn last_hash(&self) -> String {
        self.chain
            .last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| "0".to_string())
    }

    pub fn is_valid(&self) -> bool {
        for index in 1..self.chain.len() {
            let current = &self.chain[index];
            let previous = &self.chain[index - 1];

            if current.hash != current.compute_hash() {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Blockchain, Transaction};

    #[test]
    fn genesis_block_is_created() {
        let chain = Blockchain::new();
        assert_eq!(chain.chain.len(), 1);
        assert_eq!(chain.chain[0].index, 0);
    }

    #[test]
    fn new_transaction_can_be_added() {
        let mut chain = Blockchain::new();
        chain.add_transaction(Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 100,
            nonce: 1,
        });

        assert_eq!(chain.pending_transactions.len(), 1);
    }

    #[test]
    fn chain_remains_valid_after_mining() {
        let mut chain = Blockchain::new();
        chain.add_transaction(Transaction {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 25,
            nonce: 1,
        });

        chain.mine_pending_transactions("miner-a");
        assert!(chain.is_valid());
        assert_eq!(chain.chain.len(), 2);
    }
}
