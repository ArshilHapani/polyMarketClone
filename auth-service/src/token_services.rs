use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// This module provides functions to create and verify JWT tokens using the jsonwebtoken crate.
/// It defines a `Claims` struct that represents the payload of the JWT.
/// The `Claims` struct includes a subject (`sub`) and an expiration time (`exp`).
/// It also provides methods to create a new token and verify an existing token.
///
/// The `new_token` method generates a JWT token using the HS256 algorithm.
/// The `verify_token` method verifies the token and returns the claims if valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// The subject of the token, typically the user ID or username.
    pub sub: String,
    /// The expiration time of the token, represented as a Unix timestamp.
    pub exp: usize,
}

impl Claims {
    /// Creates a new `Claims` instance with the given subject and expiration time.
    ///
    /// # Arguments
    ///
    /// * `sub` - The subject of the token, typically the user ID or username.
    /// * `exp` - The expiration time of the token, represented as a Unix timestamp.
    ///
    /// # Returns
    ///
    /// A `Claims` instance with the specified subject and expiration time.
    pub fn new(sub: String, exp: usize) -> Self {
        Claims { sub, exp }
    }

    /// Generates a new JWT token using the HS256 algorithm.
    /// The token is signed with a secret key stored in the environment variable `JWT_SECRET`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated token as a `String` or an error if the token generation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable `JWT_SECRET` is not set or if the token generation fails.
    pub fn new_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let header = Header::new(Algorithm::HS256);
        let secret = std::env::var("JWT_SECRET")?;
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let token = encode(&header, self, &encoding_key)?;

        Ok(token)
    }

    /// Verifies a JWT token and returns the claims if valid.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token to verify.
    ///
    /// # Returns
    ///
    /// A `Result` containing the claims if the token is valid or an error if the verification fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable `JWT_SECRET` is not set or if the token verification fails.
    pub fn verify_token(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let secret = std::env::var("JWT_SECRET")?;
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}
