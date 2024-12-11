/// Derives a unique key with a number of elements from a
/// passphrase and salt array.
#[must_use]
pub fn derive_from_passphrase<const N: usize>(passphrase: &[u8], salt: &[u8]) -> [u8; N] {
    let mut buffer = [0u8; N];
    scrypt::scrypt(
        passphrase,
        salt,
        &scrypt::Params::recommended(),
        &mut buffer,
    )
    .unwrap();
    buffer
}
