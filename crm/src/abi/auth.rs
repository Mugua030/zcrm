use core::fmt;

use chrono::{DateTime, Utc};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use tonic::{service::Interceptor, Request, Status};
use tracing::info;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "crm";
const JWT_AUD: &str = "crm_client";

#[derive(Clone)]
pub struct Jwt {
    en_key: Ed25519KeyPair,
    de_key: Ed25519PublicKey,
}

impl Jwt {
    pub fn new(en_pem: &str, de_pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self {
            en_key: Ed25519KeyPair::from_pem(en_pem)?,
            de_key: Ed25519PublicKey::from_pem(de_pem)?,
        })
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
        self.en_key.sign(claims)
    }

    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let opts = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            ..Default::default()
        };

        let claims = self.de_key.verify_token(token, Some(opts))?;
        Ok(claims.custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UserID: {}, FullName: {}, Email: {}",
            self.id, self.fullname, self.email
        )
    }
}

impl Interceptor for Jwt {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        info!("token: {:?}", token);

        let user = match token {
            Some(bearer) => {
                let token = bearer
                    .strip_prefix("Bearer ")
                    .ok_or_else(|| Status::unauthenticated("invalid token format"))?;

                self.verify(token)
                    .map_err(|e| Status::unauthenticated(e.to_string()))?
            }
            None => return Err(Status::unauthenticated("miss token")),
        };

        req.extensions_mut().insert(user);

        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_jwt_token() {
        let config = AppConfig::load().expect("Get config fail");
        let jwt = Jwt::new(&config.auth.prk, &config.auth.pk).expect("jwt gen new fail");

        let user = User {
            id: 1,
            fullname: "C.Martin".to_string(),
            email: "testxxtestuaxx@gmail.com".to_string(),
            ..Default::default()
        };
        let ret_sign = jwt.sign(user).unwrap();
        println!("jwt-token: {}", ret_sign);

        let extract_ret = jwt.verify(&ret_sign).unwrap();
        println!("decode-result: {}", extract_ret);
    }
}
