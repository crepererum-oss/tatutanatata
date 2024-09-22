use std::ops::Deref;

use anyhow::{anyhow, bail, Context, Result};
use cbc::cipher::{
    block_padding::{NoPadding, Pkcs7},
    BlockDecryptMut, KeyIvInit,
};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

use crate::proto::keys::{EncryptedKey, Key};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type HmacSha256 = Hmac<Sha256>;

const IV_LEN: usize = 16;

pub(crate) fn decrypt_key(encryption_key: Key, key_to_be_decrypted: EncryptedKey) -> Result<Key> {
    let encrypted = match key_to_be_decrypted {
        EncryptedKey::Aes128NoMac(_) | EncryptedKey::Aes256NoMac(_) => {
            // add constant IV to encrypted data
            let iv: [u8; IV_LEN] = [128u8 + 8; IV_LEN];
            let mut data = Vec::with_capacity(key_to_be_decrypted.len() + iv.len());
            data.extend_from_slice(&iv);
            data.extend_from_slice(key_to_be_decrypted.as_ref());
            data
        }
        EncryptedKey::Aes128WithMac(_) => key_to_be_decrypted.deref().to_vec(),
    };

    let decrypted = decrypt(encryption_key, &encrypted, false)?;

    match key_to_be_decrypted {
        EncryptedKey::Aes128NoMac(_) | EncryptedKey::Aes128WithMac(_) => {
            Ok(Key::Aes128(decrypted.try_into().expect("checked length")))
        }
        EncryptedKey::Aes256NoMac(_) => {
            Ok(Key::Aes256(decrypted.try_into().expect("checked length")))
        }
    }
}

pub(crate) fn decrypt_value(encryption_key: Key, value: &[u8]) -> Result<Vec<u8>> {
    if value.is_empty() {
        return Ok(vec![]);
    }

    decrypt(encryption_key, value, true)
}

fn decrypt(encryption_key: Key, value: &[u8], padding: bool) -> Result<Vec<u8>> {
    let (encryption_key, value) = if value.len() % 2 == 1 {
        // use mac
        const MAC_LEN: usize = 32;
        if value.len() < MAC_LEN + 1 {
            bail!("mac missing")
        }
        let payload = &value[1..(value.len() - MAC_LEN)];
        let mac = &value[value.len() - MAC_LEN..];
        let subkeys = Subkeys::from(encryption_key);

        // check mac
        let mut m = HmacSha256::new_from_slice(&subkeys.mac_key).expect("checked length");
        m.update(payload);
        m.verify_slice(mac)
            .map_err(|e| anyhow!("{e}"))
            .context("HMAC verification")?;

        (subkeys.encryption_key, payload)
    } else {
        (encryption_key, value)
    };

    // get IV
    if value.len() < IV_LEN {
        bail!("IV missing")
    }
    let iv: [u8; IV_LEN] = value[..IV_LEN].try_into().expect("checked length");
    let value = &value[IV_LEN..];

    match encryption_key {
        Key::Aes128(k) => {
            if padding {
                Aes128CbcDec::new(&k.into(), &iv.into())
                    .decrypt_padded_vec_mut::<Pkcs7>(value)
                    .map_err(|e| anyhow!("{e}"))
                    .context("AES decryption")
            } else {
                Aes128CbcDec::new(&k.into(), &iv.into())
                    .decrypt_padded_vec_mut::<NoPadding>(value)
                    .map_err(|e| anyhow!("{e}"))
                    .context("AES decryption")
            }
        }
        Key::Aes256(k) => {
            if padding {
                Aes256CbcDec::new(&k.into(), &iv.into())
                    .decrypt_padded_vec_mut::<Pkcs7>(value)
                    .map_err(|e| anyhow!("{e}"))
                    .context("AES decryption")
            } else {
                Aes256CbcDec::new(&k.into(), &iv.into())
                    .decrypt_padded_vec_mut::<NoPadding>(value)
                    .map_err(|e| anyhow!("{e}"))
                    .context("AES decryption")
            }
        }
    }
}

struct Subkeys {
    encryption_key: Key,
    mac_key: Key,
}

impl From<Key> for Subkeys {
    fn from(k: Key) -> Self {
        match k {
            Key::Aes128(k) => {
                let mut hasher = Sha256::new();
                hasher.update(k);
                let hashed = hasher.finalize().to_vec();

                Self {
                    encryption_key: Key::Aes128(hashed[..16].try_into().expect("check length")),
                    mac_key: Key::Aes128(hashed[16..].try_into().expect("check length")),
                }
            }
            Key::Aes256(k) => {
                let mut hasher = Sha512::new();
                hasher.update(k);
                let hashed = hasher.finalize().to_vec();

                Self {
                    encryption_key: Key::Aes256(hashed[..32].try_into().expect("check length")),
                    mac_key: Key::Aes256(hashed[32..].try_into().expect("check length")),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_decrypt_key() {
        assert_eq!(
            decrypt_key(
                Key::Aes128(hex!("0102030405060708090a0b0c0d0e0f10")),
                EncryptedKey::Aes128NoMac(hex!("0a141e28323c46505a646e78828c96a0")),
            )
            .unwrap(),
            Key::Aes128(hex!("b10b0b75204b020f6be6f85e1a0b8f00")),
        );

        assert_eq!(
            decrypt_key(
                Key::Aes128(hex!("0102030405060708090a0b0c0d0e0f10")),
                EncryptedKey::Aes256NoMac(hex!(
                    "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"
                )),
            )
            .unwrap(),
            Key::Aes256(hex!(
                "a7e4f05300dda8d576d20ce2f818e3c3054652f1a27f0a77d470ae405aba4161"
            )),
        );

        assert_eq!(
            decrypt_key(
                Key::Aes256(hex!("a812fd92b4a09011b51799477e8c057abd6de8d9021a8289bfe4210d6812dcc0")),
                EncryptedKey::Aes128WithMac(hex!("011155a44089b3b56c809d1fd7d1a922476a5c13de555b78a7258b8b3f37c5ba839e10bbe06529a35bffaa6b2582d9b8a77b1f75247e2a7ca238202abe2f3ff55f")),
            )
            .unwrap(),
            Key::Aes128(hex!("c547a0ef919bbe29e5abaeeb6ac75264")),
        );
    }

    #[test]
    fn test_decrypt_value() {
        let k = Key::Aes256(hex!(
            "a334e6864cc70d3d7c453a500301c6dbd7332a083b4c37bc65a5d1a76fcd803c"
        ));

        let v = [
            1, 1, 221, 88, 186, 70, 178, 125, 28, 66, 245, 102, 7, 214, 121, 162, 88, 138, 118,
            208, 12, 173, 154, 251, 201, 68, 94, 254, 228, 178, 138, 73, 52, 118, 21, 143, 248,
            117, 32, 158, 29, 154, 194, 98, 55, 215, 5, 129, 18, 13, 32, 165, 44, 185, 129, 14, 78,
            146, 134, 10, 134, 81, 50, 252, 212,
        ];
        assert_eq!(decrypt_value(k, &v,).unwrap(), b"fooooo".to_owned(),);

        assert_eq!(decrypt_value(k, &[]).unwrap(), b"".to_owned());

        let mut v_broken = v;
        v_broken[1] = 0;
        assert_eq!(
            decrypt_value(k, &v_broken).unwrap_err().to_string(),
            "HMAC verification",
        );
    }
}
