#[cfg(test)]
pub mod general_tests {
    #[test]
    pub fn get_discriminator() {
        let hex_str = "04e462090000000000a803690000000000df5a2f0000000000";
        // Convert hex string to Vec<u8>
        let bytes: Vec<u8> = (0..hex_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap())
            .collect();

        // Log the first 8 bytes
        println!("First 8 bytes: {:?}", &bytes[..8]);
    }
}
