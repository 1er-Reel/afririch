// ===== AFRI-HEX — Notre propre encodage hexadécimal from scratch =====
// 100% souverain — zéro dépendance externe
// Encode bytes -> hex string et decode hex string -> bytes

const HEX_CHARS: &[u8] = b"0123456789abcdef";

/// Encode des bytes en string hexadécimale (lowercase)
pub fn encode(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        result.push(HEX_CHARS[(b >> 4) as usize] as char);
        result.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    result
}

/// Décode une string hexadécimale en bytes
/// Retourne None si la string est invalide (longueur impaire ou caractères non-hex)
pub fn decode(hex: &str) -> Option<Vec<u8>> {
    let hex_bytes = hex.as_bytes();
    if hex_bytes.len() % 2 != 0 {
        return None;
    }
    let mut result = Vec::with_capacity(hex_bytes.len() / 2);
    for i in (0..hex_bytes.len()).step_by(2) {
        let hi = hex_val(hex_bytes[i])?;
        let lo = hex_val(hex_bytes[i + 1])?;
        result.push((hi << 4) | lo);
    }
    Some(result)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode(&[0x00]), "00");
        assert_eq!(encode(&[0xff]), "ff");
        assert_eq!(encode(&[0xab, 0xcd, 0xef]), "abcdef");
        assert_eq!(encode(&[]), "");
        assert_eq!(encode(&[0x48, 0x65, 0x6c, 0x6c, 0x6f]), "48656c6c6f");
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode("00"), Some(vec![0x00]));
        assert_eq!(decode("ff"), Some(vec![0xff]));
        assert_eq!(decode("abcdef"), Some(vec![0xab, 0xcd, 0xef]));
        assert_eq!(decode(""), Some(vec![]));
        assert_eq!(decode("48656c6c6f"), Some(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]));
    }

    #[test]
    fn test_decode_invalid() {
        assert_eq!(decode("abc"), None);     // longueur impaire
        assert_eq!(decode("xy"), None);      // caractères non-hex
        assert_eq!(decode("ab cd"), None);   // espace
    }

    #[test]
    fn test_round_trip() {
        let original = vec![0x00, 0x01, 0x02, 0xff, 0xab, 0xcd];
        let encoded = encode(&original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
