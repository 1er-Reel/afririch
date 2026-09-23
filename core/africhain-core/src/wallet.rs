#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wallet {
    pub address: String,
    pub balance: u64,
}

impl Wallet {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            balance: 0,
        }
    }

    pub fn credit(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn debit(&mut self, amount: u64) -> bool {
        if self.balance < amount {
            return false;
        }

        self.balance -= amount;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Wallet;

    #[test]
    fn wallet_can_credit_and_debit() {
        let mut wallet = Wallet::new("afri-001");
        wallet.credit(500);
        assert!(wallet.debit(250));
        assert_eq!(wallet.balance, 250);
    }
}
