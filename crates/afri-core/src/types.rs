#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Address(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Hash(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Amount(pub u64);

impl Address {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        if value.trim().is_empty() {
            panic!("address cannot be empty");
        }
        Self(value)
    }
}

impl Hash {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Amount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}
