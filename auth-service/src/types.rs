use serde::Deserialize;

#[derive(Deserialize)]
pub struct GoogleClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: String,
    pub exp: usize,
}

pub enum GoogleClaimsError {
    InvalidTokenId,
    MissingKid,
    FailedToGetKeyFromGoogle,
    InvalidResponseTypeFromGoogle,
    InvalidKeyComponentFromGoogle,
    FailedToDecodeRsaComponents,
    KeyNotFound,
    ExpiredToken,
}
