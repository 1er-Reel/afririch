use crate::crypto::sign_message;
use crate::types::{Address, Amount, Hash};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: Amount,
    pub timestamp: u64,
    pub signature: String,
    pub hash: Hash,
}

impl Transaction {
    pub fn new(from: Address, to: Address, amount: Amount, timestamp: u64) -> Self {
        if amount.value() == 0 {
            panic!("amount must be > 0");
        }

        let payload = format!("{}:{}:{}:{}", from.0, to.0, amount.value(), timestamp);
        let hash = Hash::new(format!("0x{}", sha256::digest(payload.clone())));
        let signature = sign_message(&payload);

        Self {
            from,
            to,
            amount,
            timestamp,
            signature,
            hash,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.amount.value() > 0
            && !self.from.0.trim().is_empty()
            && !self.to.0.trim().is_empty()
            && !self.signature.trim().is_empty()
            && !self.hash.0.trim().is_empty()
    }
}
