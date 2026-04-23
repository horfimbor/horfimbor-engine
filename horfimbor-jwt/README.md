# horfimbor-jwt

[![Crates.io](https://img.shields.io/crates/v/horfimbor-jwt.svg)](https://crates.io/crates/horfimbor-jwt)

Shared JWT authentication for the Horfimbor ecosystem. Ensures all services produce and consume tokens in the same format, with separate integration paths for server-side Rust services and WASM browser clients.

## Features

Choose exactly one feature:

| Feature | Use case |
|---|---|
| `server` | Backend services — full signature validation, Rocket integration, token building |
| `client` | WASM frontend — decode without signature verification (for display only) |

```toml
# Backend service
[dependencies]
horfimbor-jwt = { version = "0.3", features = ["server"] }

# WASM frontend
[dependencies]
horfimbor-jwt = { version = "0.3", features = ["client"] }
```

> **Note:** If you get a UUID build error in a WASM project, add the following to your `Cargo.toml`:
> ```toml
> [dependencies.uuid]
> version = "*"
> features = ["js"]
> ```

## Token Structure

All tokens share the same `Claims` shape:

| Claim | Field | Type | Description |
|---|---|---|---|
| `aud` | `audience` | `String` | Application identifier |
| `exp` | `expiration_at` | `u64` | Unix timestamp (seconds) |
| `iat` | `issued_at` | `u64` | Unix timestamp (seconds) |
| `iss` | `issuer` | `String` | Issuing service |
| `usr` | `user` | `ModelKey` | User entity key |
| `acc` | `account` | `ModelKey` | Account entity key |
| `an` | `account_name` | `String` | Display name |
| `r` | `roles` | `Role` | Access level |

### Roles

```rust
pub enum Role {
    Admin,     // serialized as "a"
    User,      // serialized as "u"
    Anonymous, // serialized as "an"
}
```

Role hierarchy for access gates: Admin passes all gates, User passes User and Anonymous gates.

## Server Usage

### Building a token

```rust
use horfimbor_jwt::{ClaimBuilder, Role};
use horfimbor_eventsource::ModelKey;

let user    = ModelKey::new_uuid_v7("user");
let account = ModelKey::new_uuid_v7("account");

let token = ClaimBuilder::new(
        3600,          // duration in seconds
        "my-app",      // audience
        "auth-service" // issuer
    )
    .set_account(user, account, "Alice".to_string(), Role::User)
    .build("my-secret-key")?;

println!("{token}");
```

### Validating a token

```rust
use horfimbor_jwt::Claims;

let claims = Claims::from_jwt(
    &token,
    "my-secret-key",
    "my-app",      // expected audience
    "auth-service" // expected issuer
)?;

println!("User:    {}", claims.user());
println!("Account: {}", claims.account());
println!("Role:    {:?}", claims.roles());
```

### Rocket integration

`AuthClaim<R>` is a Rocket request guard that extracts and validates the JWT from the `Authorization` header. Configure via environment variables:

| Variable | Description |
|---|---|
| `JWT_SECRET_KEY` | Signing secret |
| `AUTH_HOST` | Expected issuer |
| `APP_ID` | Expected audience |

```rust
use horfimbor_jwt::rocket::{AuthClaim, GateUser, GateAdmin};
use rocket::get;

#[get("/profile")]
fn profile(auth: AuthClaim<GateUser>) -> String {
    format!("Hello, {}", auth.claims().account_name())
}

#[get("/admin")]
fn admin(auth: AuthClaim<GateAdmin>) -> String {
    "Welcome, admin".to_string()
}
```

Returns `400 Bad Request` for a missing or malformed token, `403 Forbidden` for insufficient role.

You can also validate outside of Rocket:

```rust
use horfimbor_jwt::rocket::{get_checked_claims, GateUser};

let claims = get_checked_claims(&token, GateUser)?;
```

## Client Usage

```rust
use horfimbor_jwt::Claims;

// Decodes the token payload without verifying the signature.
// Safe only for display — the server must always re-validate.
let claims = Claims::from_jwt_insecure(&token)?;
println!("Logged in as: {}", claims.account_name());
```
