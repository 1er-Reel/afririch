#[derive(Debug, Clone, PartialEq)]
pub enum AfriError {
    InvalidSignature,
    InvalidAmount,
    InvalidAddress,
    InvalidTransaction,
    LedgerError,
}

impl std::fmt::Display for AfriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AfriError::InvalidSignature => write!(f, "invalid signature"),
            AfriError::InvalidAmount => write!(f, "invalid amount"),
            AfriError::InvalidAddress => write!(f, "invalid address"),
            AfriError::InvalidTransaction => write!(f, "invalid transaction"),
            AfriError::LedgerError => write!(f, "ledger error"),
        }
    }
}
