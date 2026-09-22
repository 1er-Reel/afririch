use sha2::{Digest, Sha256};

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn sign_message(message: &str) -> String {
    format!("signature:{}", sha256_hex(message.as_bytes()))
}

pub fn verify_signature(message: &str, signature: &str) -> bool {
    let expected = sign_message(message);
    expected == signature
}
