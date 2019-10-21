use bytes::{BufMut, BytesMut};
use openssl::symm::{decrypt, Cipher};
use std::hash::Hasher;

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(bytes);
    hasher.finish()
}

pub fn get_iv(iv_prefix: &str, uuid: &str) -> BytesMut {
    let before_hash_str = format!("{}{}", iv_prefix, uuid);
    let hash = fnv1a(before_hash_str.as_bytes());
    let mut buf = BytesMut::with_capacity(16);
    buf.put_u64_be(hash);
    buf.put_u64_be(hash);
    buf.take()
}

pub fn decrypt_aes_256_cbc(hextext: String, key: &[u8], iv: BytesMut) -> Result<String, String> {
    let text = hex::decode(&hextext);
    match text {
        Ok(t) => {
            let cipher = Cipher::aes_256_cbc();
            let decrypted_text = decrypt(cipher, key, Some(&iv), &t);
            match decrypted_text {
                Ok(d) => Ok(String::from_utf8(d.to_vec()).unwrap()),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(format!("Encrypted hex input text decode failure: {:?}", e).to_string()),
    }
}
