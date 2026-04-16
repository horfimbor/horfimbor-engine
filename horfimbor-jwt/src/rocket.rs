use crate::{Claims, Role};
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use std::env;
use std::marker::PhantomData;

pub struct AuthClaim<R> {
    claims: Claims,
    role: PhantomData<R>,
}

impl<R> AuthClaim<R> {
    #[must_use]
    pub const fn claims(&self) -> &Claims {
        &self.claims
    }
}

const fn has_access_to(claims: &Claims, role: Role) -> bool {
    match claims.roles {
        Role::Admin => true,
        Role::User => !matches!(role, Role::Admin),
        Role::Anonymous => {
            matches!(role, Role::Anonymous)
        }
    }
}

pub trait Access {
    fn has_access_to(&self, claims: &Claims) -> bool;
}

pub struct GateAdmin;
pub struct GateUser;
pub struct GateAnonymous;

impl Access for AuthClaim<GateAdmin> {
    fn has_access_to(&self, claims: &Claims) -> bool {
        has_access_to(claims, Role::Admin)
    }
}
impl Access for AuthClaim<GateUser> {
    fn has_access_to(&self, claims: &Claims) -> bool {
        has_access_to(claims, Role::User)
    }
}
impl Access for AuthClaim<GateAnonymous> {
    fn has_access_to(&self, claims: &Claims) -> bool {
        has_access_to(claims, Role::Anonymous)
    }
}

#[derive(Debug)]
pub enum AccountClaimError {
    Claim,
    PermissionDenied,
    Missing,
}

// FIXME replace String by a proper error
// FIXME audience ( app_id ) should not be from env
fn get_jwt_claims(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET_KEY").map_err(|_| "JWT_SECRET_KEY is missing")?;
    let auth_host = env::var("AUTH_HOST").map_err(|_| "AUTH_HOST is missing")?;
    let app_id = env::var("APP_ID").map_err(|_| "APP_ID is missing")?;
    let claims = Claims::from_jwt(token, &secret, &app_id, &auth_host).map_err(|e| {
        println!("claims error : {e:?}");
        "Invalid claims"
    })?;
    Ok(claims)
}

// FIXME replace String by a proper error
/// # Errors
///
/// When the token cannot be parsed, is not valid or the role is not enough.
pub fn get_checked_claims(token: &str, role: Role) -> Result<Claims, String> {
    let claims = get_jwt_claims(token)?;
    if has_access_to(&claims, role) {
        Ok(claims)
    } else {
        Err("no access".to_string())
    }
}

#[rocket::async_trait]
impl<'r, R> FromRequest<'r> for AuthClaim<R>
where
    Self: Access,
{
    type Error = AccountClaimError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        req.headers().get_one("Authorization").map_or(
            Outcome::Error((Status::BadRequest, AccountClaimError::Missing)),
            |token| match get_jwt_claims(token) {
                Ok(claims) => {
                    let auth = Self {
                        claims,
                        role: PhantomData,
                    };

                    if auth.has_access_to(&auth.claims) {
                        Outcome::Success(auth)
                    } else {
                        Outcome::Error((Status::Forbidden, AccountClaimError::PermissionDenied))
                    }
                }
                Err(e) => {
                    dbg!(e);
                    Outcome::Error((Status::BadRequest, AccountClaimError::Claim))
                }
            },
        )
    }
}
