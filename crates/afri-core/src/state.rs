use std::collections::HashMap;

use crate::types::Address;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub balances: HashMap<String, u64>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_balance(&self, address: &Address) -> u64 {
        *self.balances.get(&address.0).unwrap_or(&0)
    }

    pub fn set_balance(&mut self, address: &Address, value: u64) {
        self.balances.insert(address.0.clone(), value);
    }
}
