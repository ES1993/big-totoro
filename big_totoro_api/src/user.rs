use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use big_totoro_core::{
    config::Config,
    result::{AppError, AppResult},
    token::Claims,
};
use serde::{Deserialize, Serialize};

pub fn router(path: &str) -> Router {
    let router = Router::new().route("/login", post(login));

    Router::new().nest(path, router)
}

#[derive(Debug, Deserialize)]
struct LoginReqBody {
    id: String,
}

#[derive(Serialize)]
struct LoginResBody {
    token: String,
}

async fn login(
    Extension(config): Extension<Arc<Config>>,
    Json(req_body): Json<LoginReqBody>,
) -> AppResult<Json<LoginResBody>> {
    let token = Claims::new(&req_body.id, &config.token_secret, config.token_exp)?;

    Ok(Json(LoginResBody { token: token }))
}

async fn exist() -> AppResult<()> {
    unimplemented!()
}

async fn register() -> AppResult<()> {
    std::fs::read_dir("/a.txt")?;
    Err(AppError::BadRequest("sjdsdjs".into()))
}
