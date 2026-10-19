use sha2::{Digest, Sha256, Sha512};

enum ModHasher {
    Sha512(Sha512),
    Sha256(Sha256),
}

pub(crate) struct ModHashVerifier {
    expected: String,
    hasher: ModHasher,
}

impl ModHashVerifier {
    pub(crate) fn new(hashes: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        let expected = hashes
            .get("sha512")
            .map(|value| ("sha512", value))
            .or_else(|| hashes.get("sha256").map(|value| ("sha256", value)))
            .ok_or("Mod package has no SHA-512 or SHA-256 checksum")?;
        let hasher = match expected.0 {
            "sha512" => ModHasher::Sha512(Sha512::new()),
            _ => ModHasher::Sha256(Sha256::new()),
        };
        Ok(Self {
            expected: expected.1.clone(),
            hasher,
        })
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) {
        match &mut self.hasher {
            ModHasher::Sha512(hasher) => hasher.update(bytes),
            ModHasher::Sha256(hasher) => hasher.update(bytes),
        }
    }

    pub(crate) fn verify(self) -> Result<(), String> {
        let actual = match self.hasher {
            ModHasher::Sha512(hasher) => format!("{:x}", hasher.finalize()),
            ModHasher::Sha256(hasher) => format!("{:x}", hasher.finalize()),
        };
        if !actual.eq_ignore_ascii_case(&self.expected) {
            return Err("Mod package checksum does not match".into());
        }
        Ok(())
    }
}

pub fn verify_mod_hash(
    bytes: &[u8],
    hashes: &std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let mut verifier = ModHashVerifier::new(hashes)?;
    verifier.update(bytes);
    verifier.verify()
}
