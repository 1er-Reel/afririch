use crate::transaction::Transaction;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Ledger {
    pub transactions: Vec<Transaction>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        if !tx.is_valid() {
            panic!("invalid transaction");
        }
        self.transactions.push(tx);
    }

    pub fn count(&self) -> usize {
        self.transactions.len()
    }
}
