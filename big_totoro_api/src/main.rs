mod user;

use axum::{Extension, Router, Server};
use big_totoro_core::{config::Config, result::AppResult};
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Arc::new(Config::new()?);

    let router = Router::new()
        .merge(user::router("/user"))
        .layer(Extension(config.clone()));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
