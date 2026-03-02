// #![deny(missing_docs)]
// #![doc = include_str!("../README.md")]

#[cfg(feature = "server")]
pub mod builder;
pub mod model_key;

#[cfg(feature = "server")]
use horfimbor_eventsource::model_key::ModelKey;

#[cfg(not(feature = "server"))]
use model_key::ModelKey;

#[cfg(feature = "server")]
use jsonwebtoken::{DecodingKey, Validation, decode};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "client")]
use jsonwebtoken::dangerous::insecure_decode;
#[cfg(feature = "client")]
use jsonwebtoken::decode_header;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "aud")]
    audience: String,
    #[serde(rename = "exp")]
    expiration_at: u64,
    #[serde(rename = "iat")]
    issued_at: u64,
    #[serde(rename = "iss")]
    issuer: String,
    #[serde(rename = "usr")]
    user: ModelKey,
    #[serde(rename = "acc")]
    account: ModelKey,
    #[serde(rename = "an")]
    account_name: String,
    #[serde(rename = "r")]
    roles: Role,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Role {
    #[serde(rename = "a")]
    Admin,
    #[serde(rename = "u")]
    User,
    #[serde(rename = "an")]
    Anonymous,
}

#[derive(Error, Debug)]
pub enum ClaimError {
    #[error("jsonwebtoken")]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("cannot get data `{0}`")]
    Other(String),

    #[error("no account when building claims")]
    EmptyAccount,
}

impl Claims {
    #[cfg(feature = "server")]
    /// parse the token, validate the secrets, audience and issuer
    ///
    /// # Errors
    ///
    /// Will return `ClaimError` if the decoding failed
    pub fn from_jwt(
        token: &str,
        secret: &str,
        audience: &str,
        issuer: &str,
    ) -> Result<Self, ClaimError> {
        let mut val = Validation::default();
        val.set_audience(&[&audience]);
        val.set_issuer(&[&issuer]);
        val.set_required_spec_claims(&["exp", "iss", "aud"]);

        let value = decode::<Self>(token, &DecodingKey::from_secret(secret.as_ref()), &val)
            .map_err(ClaimError::JWT)?;

        Ok(value.claims)
    }

    #[cfg(feature = "client")]
    /// parse the token, but do not validate it
    ///
    /// # Errors
    ///
    /// Will return `ClaimError` if the decoding failed
    pub fn from_jwt_insecure(token: &str) -> Result<Self, ClaimError> {
        match decode_header(token) {
            Ok(_) => {
                let value = insecure_decode::<Self>(token).map_err(ClaimError::JWT)?;
                Ok(value.claims)
            }
            Err(e) => Err(ClaimError::JWT(e)),
        }
    }

    #[must_use]
    pub fn audience(&self) -> &str {
        &self.audience
    }

    #[must_use]
    pub const fn expiration_at(&self) -> u64 {
        self.expiration_at
    }

    #[must_use]
    pub const fn issued_at(&self) -> u64 {
        self.issued_at
    }

    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    #[must_use]
    pub const fn user(&self) -> &ModelKey {
        &self.user
    }

    #[must_use]
    pub const fn roles(&self) -> &Role {
        &self.roles
    }

    #[must_use]
    pub const fn account(&self) -> &ModelKey {
        &self.account
    }
    #[must_use]
    pub fn account_name(&self) -> &str {
        &self.account_name
    }
}
