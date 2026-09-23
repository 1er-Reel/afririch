use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub balance: i64,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub nonce: u64,
    pub note: String,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {} : {} {}", self.from, self.to, self.amount, self.note)
    }
}

impl Wallet {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            balance: 0,
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.balance += tx.amount;
        self.transactions.push(tx);
    }

    pub fn balance(&self) -> i64 {
        self.balance
    }
}

#[cfg(test)]
mod tests {
    use super::{Transaction, Wallet};

    #[test]
    fn wallet_tracks_balances_and_history() {
        let mut wallet = Wallet::new("afri-wallet-001");
        wallet.add_transaction(Transaction {
            from: "alice".to_string(),
            to: "afri-wallet-001".to_string(),
            amount: 100,
            nonce: 1,
            note: "credit".to_string(),
        });

        wallet.add_transaction(Transaction {
            from: "afri-wallet-001".to_string(),
            to: "bob".to_string(),
            amount: -25,
            nonce: 2,
            note: "payment".to_string(),
        });

        assert_eq!(wallet.balance(), 75);
        assert_eq!(wallet.transactions.len(), 2);
    }
}
