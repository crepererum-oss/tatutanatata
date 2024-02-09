use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::proto::{Base64Url, KdfVersion};

/// Build auth verifier for session creation.
pub fn build_auth_verifier(
    kdf_version: KdfVersion,
    passphrase: &str,
    salt: &[u8],
) -> Result<Base64Url> {
    let passkey = derive_passkey(kdf_version, passphrase, salt).context("derive passkey")?;
    Ok(encode_auth_verifier(&passkey))
}

fn derive_passkey(kdf_version: KdfVersion, passphrase: &str, salt: &[u8]) -> Result<Vec<u8>> {
    match kdf_version {
        KdfVersion::Bcrypt => {
            let mut hasher = Sha256::new();
            hasher.update(passphrase.as_bytes());
            let passphrase = hasher.finalize();

            let salt: [u8; 16] = salt.try_into().context("salt length")?;

            let hashed = bcrypt::bcrypt(8, salt, &passphrase);

            Ok(hashed[..16].to_owned())
        }
        KdfVersion::Argon2id => bail!("not implemented: Argon2id"),
    }
}

fn encode_auth_verifier(passkey: &[u8]) -> Base64Url {
    let mut hasher = Sha256::new();
    hasher.update(passkey);
    let hashed = hasher.finalize().to_vec();

    Base64Url::from(hashed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_verifier() {
        assert_eq!(
            build_auth_verifier(KdfVersion::Bcrypt, "password", b"saltsaltsaltsalt")
                .unwrap()
                .to_string(),
            "r3YdONamUCQ7yFZwPFX8KLWZ4kKnAZLyt7rwi1DCE1I",
        );
    }
}
