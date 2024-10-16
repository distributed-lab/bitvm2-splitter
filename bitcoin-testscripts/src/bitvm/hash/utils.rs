use bitcoin_utils::treepp::*;

pub fn push_bytes_hex(hex: &str) -> Script {
    let hex: String = hex
        .chars()
        .filter(|c| c.is_ascii_digit() || c.is_ascii_alphabetic())
        .collect();

    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>();

    script! {
        for byte in bytes.iter().rev() {
            { *byte }
        }
    }
}
