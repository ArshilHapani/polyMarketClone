use aes::cipher::generic_array::GenericArray;
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};

pub fn encrypt(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let key_str = std::env::var("SECRET_KEY")?;
    let key_raw = key_str.as_bytes();

    if key_raw.len() != 32 {
        return Err("Key must be 32 bytes long for AES-256".into());
    }

    let key = GenericArray::clone_from_slice(key_raw);
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; 12];

    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut cipher_text = cipher
        .encrypt(nonce, data)
        .map_err(|_| "Encryption failed")?;

    cipher_text.extend_from_slice(&nonce_bytes);

    Ok(cipher_text)
}

pub fn decrypt(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let key_str = std::env::var("SECRET_KEY")?;
    let key_raw = key_str.as_bytes();

    if key_raw.len() != 32 {
        return Err("Key must be 32 bytes long for AES-256".into());
    }

    let key = GenericArray::clone_from_slice(key_raw);
    let cipher = Aes256Gcm::new(&key);

    let (cipher_text, nonce) = data.split_at(data.len() - 12);
    let nonce = Nonce::from_slice(nonce);
    let decrypted_data = cipher
        .decrypt(nonce, cipher_text)
        .map_err(|_| "Decryption failed")?;

    Ok(decrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"58a46bda57e1cf00a0cc0548d5744568fe846edcf13824616cddda8b4c376ba37029f1c0c63b940a2c3187d1b7cecc1815c0f72d987ed3b815bdd4d7725d243cf7d20db1d954cbd5b5c2d2fdba110096bb1e8dc705c3f63d4e59b01462ff6d4d570f1963f1e6ffc8150cedc97b13f479d65e016c641e522443affc8e226f5af4b49c36956dcf755e12a936a383af310ee5b675d66880d266862ff6d82359fc9188e2f48d3dbd2a5c15ea1123a3a3b14dd4100b8d9802483c9eeb5b4274b2fc32b6c3f34c97a7d234ccf617e4b434a52b764c6bf29c1bfc3df6a6a548d1e632295342347ae18d1e1113944c616e986130d704bb9727e447f48a53bcbc9db00e76"; // random 256 bytes
        let encrypted_data = encrypt(data).unwrap();
        let string_encrypted_data = String::from_utf8(encrypted_data).unwrap();

        let decrypted_data = decrypt(&string_encrypted_data.as_bytes()).unwrap();

        assert_eq!(data.to_vec(), decrypted_data);
    }
}
