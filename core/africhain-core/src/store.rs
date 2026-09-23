use std::fs;
use std::path::PathBuf;

use africhain_core::{Blockchain, Transaction};

#[derive(Clone, Debug)]
pub struct BlockchainStore {
    path: PathBuf,
}

impl BlockchainStore {
    pub fn new(path: &str) -> Self {
        let dir = PathBuf::from(path);
        if !dir.exists() {
            fs::create_dir_all(&dir).expect("failed to create store directory");
        }
        Self { path: dir }
    }

    pub fn save(&self, blockchain: &Blockchain) {
        let data = serde_json::to_string_pretty(blockchain).expect("failed to serialize blockchain");
        let file = self.path.join("blockchain.json");
        fs::write(file, data).expect("failed to write blockchain state");
    }

    pub fn load(&self) -> Blockchain {
        let file = self.path.join("blockchain.json");
        if !file.exists() {
            return Blockchain::new();
        }

        let raw = fs::read_to_string(file).expect("failed to read blockchain state");
        serde_json::from_str(&raw).unwrap_or_else(|_| Blockchain::new())
    }
}
