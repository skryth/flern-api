use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub exp: i64,
}

pub fn generate_token<K: AsRef<[u8]>>(
    claims: UserClaims,
    key: K,
) -> jsonwebtoken::errors::Result<String> {
    let header = Header::default();
    let key = EncodingKey::from_secret(key.as_ref());

    let token = jsonwebtoken::encode(&header, &claims, &key)?;
    Ok(token)
}

pub fn process_token<K: AsRef<[u8]>>(
    token: &str,
    key: K,
) -> jsonwebtoken::errors::Result<TokenData<UserClaims>> {
    let validation = Validation::default();
    let key = DecodingKey::from_secret(key.as_ref());

    let claims = jsonwebtoken::decode::<UserClaims>(token, &key, &validation)?;
    Ok(claims)
}
