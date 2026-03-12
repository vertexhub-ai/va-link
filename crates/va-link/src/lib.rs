use hmac::{Hmac, Mac};
use sha2::Sha256;

pub type HmacSha256 = Hmac<Sha256>;

pub fn hash_api_key(api_key: &str, secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(api_key.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

// verify_api_key is not strictly needed for this service, as we only store the hash.
