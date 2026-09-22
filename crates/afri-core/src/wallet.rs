use crate::types::Address;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Wallet {
    pub address: Address,
}

impl Wallet {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: Address::new(address),
        }
    }
}
