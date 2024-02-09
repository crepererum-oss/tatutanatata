use anyhow::{anyhow, bail, Context, Result};
use cbc::cipher::{
    block_padding::{NoPadding, Pkcs7},
    BlockDecryptMut, KeyIvInit,
};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type HmacSha256 = Hmac<Sha256>;

pub fn decrypt_key(encryption_key: &[u8], key_to_be_decrypted: &[u8]) -> Result<Vec<u8>> {
    if let Ok(k) = TryInto::<[u8; 16]>::try_into(encryption_key) {
        let iv: [u8; 16] = [128u8 + 8; 16];
        Aes128CbcDec::new(&k.into(), &iv.into())
            .decrypt_padded_vec_mut::<NoPadding>(key_to_be_decrypted)
            .map_err(|e| anyhow!("{e}"))
            .context("AES decryption")
    } else if let Ok(_k) = TryInto::<[u8; 32]>::try_into(encryption_key) {
        bail!("not implemented: AES256")
    } else {
        bail!("invalid encryption key length: {}", encryption_key.len())
    }
}

pub fn decrypt_value(encryption_key: &[u8], value: &[u8]) -> Result<Vec<u8>> {
    if let Ok(_k) = TryInto::<[u8; 16]>::try_into(encryption_key) {
        bail!("not implemented: AES128")
    } else if let Ok(k) = TryInto::<[u8; 32]>::try_into(encryption_key) {
        let (k, value) = if value.len() % 2 == 1 {
            // use mac
            const MAC_LEN: usize = 32;
            if value.len() < MAC_LEN + 1 {
                bail!("mac missing")
            }
            let payload = &value[1..(value.len() - MAC_LEN)];
            let mac = &value[value.len() - MAC_LEN..];
            let subkeys = Aes256Subkeys::from(k);

            // check mac
            let mut m = HmacSha256::new_from_slice(&subkeys.mkey).expect("checked length");
            m.update(payload);
            m.verify_slice(mac)
                .map_err(|e| anyhow!("{e}"))
                .context("HMAC verification")?;

            (subkeys.ckey, payload)
        } else {
            (k, value)
        };

        // get IV
        const IV_LEN: usize = 16;
        if value.len() < IV_LEN {
            bail!("IV missing")
        }
        let iv: [u8; IV_LEN] = value[..IV_LEN].try_into().expect("checked length");
        let value = &value[IV_LEN..];
        Aes256CbcDec::new(&k.into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(value)
            .map_err(|e| anyhow!("{e}"))
            .context("AES decryption")
    } else {
        bail!("invalid encryption key length: {}", encryption_key.len())
    }
}

struct Aes256Subkeys {
    ckey: [u8; 32],
    mkey: [u8; 32],
}

impl From<[u8; 32]> for Aes256Subkeys {
    fn from(k: [u8; 32]) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(k);
        let hashed = hasher.finalize().to_vec();

        Self {
            ckey: hashed[..32].try_into().expect("check length"),
            mkey: hashed[32..].try_into().expect("check length"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrypt_key() {
        assert_eq!(
            decrypt_key(
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                &[10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160]
            )
            .unwrap(),
            vec![177u8, 11, 11, 117, 32, 75, 2, 15, 107, 230, 248, 94, 26, 11, 143, 0],
        );
    }

    #[test]
    fn test_decrypt_value() {
        let k = [
            163, 52, 230, 134, 76, 199, 13, 61, 124, 69, 58, 80, 3, 1, 198, 219, 215, 51, 42, 8,
            59, 76, 55, 188, 101, 165, 209, 167, 111, 205, 128, 60,
        ];

        let v = [
            1, 1, 221, 88, 186, 70, 178, 125, 28, 66, 245, 102, 7, 214, 121, 162, 88, 138, 118,
            208, 12, 173, 154, 251, 201, 68, 94, 254, 228, 178, 138, 73, 52, 118, 21, 143, 248,
            117, 32, 158, 29, 154, 194, 98, 55, 215, 5, 129, 18, 13, 32, 165, 44, 185, 129, 14, 78,
            146, 134, 10, 134, 81, 50, 252, 212,
        ];

        assert_eq!(decrypt_value(&k, &v,).unwrap(), b"fooooo".to_owned(),);

        let mut v_broken = v.clone();
        v_broken[1] = 0;

        assert_eq!(
            decrypt_value(&k, &v_broken).unwrap_err().to_string(),
            "HMAC verification",
        );
    }
}
