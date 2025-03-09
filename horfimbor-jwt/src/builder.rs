use horfimbor_eventsource::model_key::ModelKey;
use crate::{ClaimError, Claims, Role};

use jsonwebtoken::{encode, get_current_timestamp, EncodingKey, Header};

pub struct ClaimBuilder {
    check_part: CheckPart,
    app_part: Option<AppPart>,
}

struct CheckPart {
    audience: String,
    expiration_at: u64,
    issued_at: u64,
    issuer: String,
}

struct AppPart {
    user: ModelKey,
    account: ModelKey,
    account_name: String,
    roles: Role,
}

impl ClaimBuilder {
    pub fn new(duration: u64, audience: String, issuer: String) -> Self {
        let since_the_epoch = get_current_timestamp();

        let check_part = CheckPart {
            expiration_at: since_the_epoch + duration,
            issued_at: since_the_epoch,
            audience,
            issuer,
        };

        Self {
            check_part,
            app_part: None,
        }
    }

    pub fn set_account(&mut self, user: ModelKey, account: ModelKey,account_name: String , roles: Role) {
        let app_part = AppPart {
            user,
            account,
            account_name,
            roles,
        };
        self.app_part = Some(app_part)
    }

    pub fn build(self, secret: &str) ->  Result<String, ClaimError> {
        let (user, account, account_name, roles) = match self.app_part {
            None => {
                return Err(ClaimError::EmptyAccount)
            },
            Some(app_part) => (app_part.user, app_part.account, app_part.account_name, app_part.roles),
        };

        let claim = Claims {
            audience: self.check_part.audience,
            expiration_at: self.check_part.expiration_at,
            issued_at: self.check_part.issued_at,
            issuer: self.check_part.issuer,
            user,
            account,
            account_name,
            roles,
        };

        encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(secret.as_ref()),
        )
            .map_err(|e| ClaimError::JWT(e))
    }
}
