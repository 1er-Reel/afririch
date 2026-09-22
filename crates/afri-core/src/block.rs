use crate::transaction::Transaction;
use crate::types::Hash;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: Hash,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub hash: Hash,
}

impl Block {
    pub fn new(index: u64, previous_hash: Hash, timestamp: u64, transactions: Vec<Transaction>) -> Self {
        let hash = Hash::new(format!("block:{}:{}:{}", index, timestamp, transactions.len()));
        Self {
            index,
            previous_hash,
            timestamp,
            transactions,
            nonce: 0,
            hash,
        }
    }
}
