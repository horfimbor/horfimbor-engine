// #![deny(missing_docs)]
// #![doc = include_str!("../README.md")]

pub mod builder;

use horfimbor_eventsource::model_key::ModelKey;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Serialize, Deserialize)]
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

    #[error("no account when building claims")]
    EmptyAccount,
}

impl Claims {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::ClaimBuilder;

    #[test]
    fn test() {
        let audience = "some_app";
        let issuer = "http://auth.localhost:8000";
        let secret = "SOME_SECRET";

        let mut cb = ClaimBuilder::new(30, audience.to_string(), issuer.to_string());

        let user = ModelKey::new_uuid_v7("user");
        let account = ModelKey::new_uuid_v7("account");
        let account_name = "horfirion".to_string();

        cb.set_account(
            user.clone(),
            account.clone(),
            account_name.clone(),
            Role::Anonymous,
        );

        let token = cb.build("SOME_SECRET").unwrap();

        let claim = Claims::from_jwt(&token, secret, audience, issuer).unwrap();

        assert_eq!(user, claim.user);
        assert_eq!(account, claim.account);
        assert_eq!(account_name, account_name);
    }
}
