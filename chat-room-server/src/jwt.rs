use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::Months;
use hmac::{Hmac, Mac};

use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub user_id: String,
    exp: i64,
}

#[derive(Serialize, Deserialize)]
struct Header<'u> {
    alg: &'u str,
    typ: &'u str,
}
#[derive(Debug)]
pub enum TokenError {
    JsonEncode,
    Base64Encode,
    JsonDecode,
    HmacCreation,
    TokenPartMissing,
    InvalidSign,
    ExpireTimeCalculation,
}

pub fn token_from_barear(str: &str) -> Result<&str, TokenError> {
    let mut parts = str.split(" ");
    if parts.next() != Some("Bearer") {
        return Err(TokenError::TokenPartMissing);
    }
    parts.next().ok_or(TokenError::TokenPartMissing)
}

pub fn generate_jwt(user_id: &str, secret_key: &[u8]) -> Result<String, TokenError> {
    let exp = match chrono::Utc::now().checked_add_months(Months::new(1)) {
        Some(date) => date,
        None => return Err(TokenError::ExpireTimeCalculation),
    };

    let body = Body {
        user_id: user_id.to_string(),
        exp: exp.timestamp_millis(),
    };

    let header = Header {
        alg: "HS256",
        typ: "JWT",
    };

    let header = rocket::serde::json::to_string(&header).map_err(|_| TokenError::JsonDecode)?;
    let body = rocket::serde::json::to_string(&body).map_err(|_| TokenError::JsonDecode)?;

    let header = BASE64_STANDARD.encode(header);
    let body = BASE64_STANDARD.encode(body);

    let data = format!("{}.{}", header, body);

    let sign = hmac_256_sign(&secret_key, data.as_bytes())?;

    Ok(format!("{}.{}", data, sign))
}

pub fn validate_jwt(token: &str, secret_key: &[u8]) -> Result<Body, TokenError> {
    let parts = token.split('.').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(TokenError::TokenPartMissing);
    };

    let header = parts[0];
    let body = parts[1];
    let sign = parts[2];

    let header = BASE64_STANDARD
        .decode(header)
        .map_err(|_| TokenError::Base64Encode)?;
    let body = BASE64_STANDARD
        .decode(body)
        .map_err(|_| TokenError::Base64Encode)?;

    let _header: Header =
        rocket::serde::json::from_slice(&header).map_err(|_| TokenError::JsonEncode)?;
    let body: Body = rocket::serde::json::from_slice(&body).map_err(|_| TokenError::JsonEncode)?;

    let data = token.split('.').take(2).collect::<Vec<&str>>().join(".");
    let received_sign = sign;
    let expected_sign = hmac_256_sign(secret_key, data.as_bytes())?;

    if received_sign != expected_sign {
        return Err(TokenError::InvalidSign);
    }

    Ok(body)
}

fn hmac_256_sign(secret_key: &[u8], data: &[u8]) -> Result<String, TokenError> {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret_key).map_err(|_| TokenError::HmacCreation)?;
    mac.update(data);
    let sign = mac.finalize().into_bytes();
    Ok(BASE64_STANDARD.encode(sign))
}

#[cfg(test)]
mod test {
    use crate::user::User;

    use super::*;

    #[test]
    fn test_jwt_generate_validate() {
        let user = User {
            id: "123".to_string(),
            name: "john doe".to_string(),
            secret: "secret".to_string(),
        };

        let s = b"secret";
        let jwt = generate_jwt(&user.id, s).expect("Cant generate jwt");
        let body = validate_jwt(&jwt, s).expect("Cant validate jwt");
        assert_eq!(body.user_id, user.id)
    }

    #[test]
    fn test_token_from_barear() {
        let token = "Bearer secret";

        let str = token_from_barear(token).unwrap();
        assert_eq!(str, "secret");
    }
}
