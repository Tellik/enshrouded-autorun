use sha2::{Digest, Sha256, Sha512};

pub fn verify_mod_hash(
    bytes: &[u8],
    hashes: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let expected = hashes
        .get("sha512")
        .map(|value| ("sha512", value))
        .or_else(|| hashes.get("sha256").map(|value| ("sha256", value)))
        .ok_or("Mod package has no SHA-512 or SHA-256 checksum")?;
    let actual = match expected.0 {
        "sha512" => format!("{:x}", Sha512::digest(bytes)),
        _ => format!("{:x}", Sha256::digest(bytes)),
    };
    if !actual.eq_ignore_ascii_case(expected.1) {
        return Err("Mod package checksum does not match".into());
    }
    Ok(())
}
