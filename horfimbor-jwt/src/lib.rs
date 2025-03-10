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
    pub fn from_jwt(
        token: &str,
        secret: &str,
        audience: String,
        issuer: String,
    ) -> Result<Claims, ClaimError> {
        let mut val = Validation::default();
        val.set_audience(&[&audience]);
        val.set_issuer(&[&issuer]);
        val.set_required_spec_claims(&["exp", "iss", "aud"]);

        let value = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &val)
            .map_err(|e| ClaimError::JWT(e))?;

        Ok(value.claims)
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn expiration_at(&self) -> u64 {
        self.expiration_at
    }

    pub fn issued_at(&self) -> u64 {
        self.issued_at
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn user(&self) -> &ModelKey {
        &self.user
    }

    pub fn roles(&self) -> &Role {
        &self.roles
    }

    pub fn account(&self) -> &ModelKey {
        &self.account
    }
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

        let claim =
            Claims::from_jwt(&token, secret, audience.to_string(), issuer.to_string()).unwrap();

        assert_eq!(user, claim.user);
        assert_eq!(account, claim.account);
        assert_eq!(account_name, account_name);
    }
}
