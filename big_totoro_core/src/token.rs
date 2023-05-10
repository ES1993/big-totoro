use crate::{result::{AppError, AppResult}, config::Config};
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    Extension, RequestPartsExt,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{ops::Add, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    exp: i64,
}

impl Claims {
    pub fn new(id: &str, secret: &str, exp: i64) -> AppResult<String> {
        let claims = Claims {
            exp: Utc::now().add(Duration::days(exp)).timestamp(),
            id: id.to_string(),
        };

        Ok(encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|_| AppError::ServerError("创建令牌失败".to_string()))?)
    }

    pub fn decode(token_str: &str, secret: &str) -> AppResult<Claims> {
        Ok(decode::<Claims>(
            token_str,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AppError::BadRequest("无效的令牌".into()))?
        .claims)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::BadRequest("缺失的令牌".into()))?;

        let Extension(config) = Extension::<Arc<Config>>::from_request_parts(parts, state).await?;

        let claims = Claims::decode(bearer.token(), config.token_secret.as_str())?;

        Ok(claims)
    }
}
