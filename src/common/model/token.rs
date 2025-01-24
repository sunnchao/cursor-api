use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokenPayload {
    pub sub: String,
    pub time: String,
    pub randomness: String,
    pub exp: i64,
    pub iss: String,
    pub scope: String,
    pub aud: String,
}
