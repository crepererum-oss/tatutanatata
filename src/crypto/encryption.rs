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
    use super::*;

    #[test]
    fn test_decrypt_key() {
        assert_eq!(
            decrypt_key(
                Key::Aes128([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
                EncryptedKey::Aes128NoMac([
                    10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160
                ]),
            )
            .unwrap(),
            Key::Aes128([177u8, 11, 11, 117, 32, 75, 2, 15, 107, 230, 248, 94, 26, 11, 143, 0]),
        );

        assert_eq!(
            decrypt_key(
                Key::Aes128([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
                EncryptedKey::Aes256NoMac([42; 32]),
            )
            .unwrap(),
            Key::Aes256([
                167, 228, 240, 83, 0, 221, 168, 213, 118, 210, 12, 226, 248, 24, 227, 195, 5, 70,
                82, 241, 162, 127, 10, 119, 212, 112, 174, 64, 90, 186, 65, 97
            ]),
        );

        assert_eq!(
            decrypt_key(
                Key::Aes256([
                    168, 18, 253, 146, 180, 160, 144, 17, 181, 23, 153, 71, 126, 140, 5, 122, 189,
                    109, 232, 217, 2, 26, 130, 137, 191, 228, 33, 13, 104, 18, 220, 192,
                ],),
                EncryptedKey::Aes128WithMac([
                    1, 17, 85, 164, 64, 137, 179, 181, 108, 128, 157, 31, 215, 209, 169, 34, 71,
                    106, 92, 19, 222, 85, 91, 120, 167, 37, 139, 139, 63, 55, 197, 186, 131, 158,
                    16, 187, 224, 101, 41, 163, 91, 255, 170, 107, 37, 130, 217, 184, 167, 123, 31,
                    117, 36, 126, 42, 124, 162, 56, 32, 42, 190, 47, 63, 245, 95,
                ],)
            )
            .unwrap(),
            Key::Aes128([
                197, 71, 160, 239, 145, 155, 190, 41, 229, 171, 174, 235, 106, 199, 82, 100
            ]),
        );
    }

    #[test]
    fn test_decrypt_value() {
        let k = Key::Aes256([
            163, 52, 230, 134, 76, 199, 13, 61, 124, 69, 58, 80, 3, 1, 198, 219, 215, 51, 42, 8,
            59, 76, 55, 188, 101, 165, 209, 167, 111, 205, 128, 60,
        ]);

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
