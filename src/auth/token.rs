use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use rand::{self, RngCore};

pub fn generate_token() -> String {
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);
    URL_SAFE.encode(buf)
}
