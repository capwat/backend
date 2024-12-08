use sha2::{Digest, Sha256};

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().try_into().unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn sha512() {
        // Generated from: https://codebeautify.org/sha256-hash-generator
        let result = super::sha256(b"hello_world!");
        let expected_hash =
            hex::decode("b7a98bdbdb3294473ff2c204e3658b051487b24f99bcaa0666dc340373141df0")
                .unwrap();

        assert_eq!(expected_hash, result.as_slice());
    }
}
