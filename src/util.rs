pub fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

pub fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() {
        return Some(Vec::new());
    }
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn hex_encode_single_byte() {
        assert_eq!(hex_encode(&[0x41]), "41");
    }

    #[test]
    fn hex_encode_multi_byte() {
        assert_eq!(hex_encode(&[0x0D, 0x0A, 0xFF]), "0D0AFF");
    }

    #[test]
    fn hex_decode_empty() {
        assert_eq!(hex_decode(""), Some(vec![]));
    }

    #[test]
    fn hex_decode_valid() {
        assert_eq!(hex_decode("48656C6C6F"), Some(b"Hello".to_vec()));
    }

    #[test]
    fn hex_decode_odd_length() {
        assert_eq!(hex_decode("ABC"), None);
    }

    #[test]
    fn hex_decode_mixed_case() {
        assert_eq!(hex_decode("0d0a"), Some(vec![0x0D, 0x0A]));
    }
}
