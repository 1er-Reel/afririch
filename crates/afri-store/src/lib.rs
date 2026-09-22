use afri_core::Ledger;
use std::fs;
use std::path::Path;

#[derive(Default, Clone, Debug)]
pub struct Store {
    path: String,
}

impl Store {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn save_ledger(&self, ledger: &Ledger) -> std::io::Result<()> {
        let parent = Path::new(&self.path).parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let json = serde_json::to_string_pretty(ledger)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn load_ledger(&self) -> std::io::Result<Ledger> {
        let data = fs::read_to_string(&self.path)?;
        let ledger: Ledger = serde_json::from_str(&data)?;
        Ok(ledger)
    }
}
