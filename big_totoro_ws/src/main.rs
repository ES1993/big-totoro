mod client;
mod state;
mod ws;

use axum::{routing::get, Extension, Router, Server};
use big_totoro_core::{config::Config, etcd::Etcd, result::AppResult};
use state::AppState;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Arc::new(Config::new()?);
    let etcd = Arc::new(Etcd::new(config.clone()));
    let state = AppState::new();

    let router = Router::new()
        .route("/ws", get(ws::handler))
        .layer(Extension(config.clone()))
        .layer(Extension(etcd.clone()))
        .with_state(state);

    Server::bind(
        &format!("0.0.0.0:{}", config.local_ws_server_port)
            .parse()
            .unwrap(),
    )
    .serve(router.into_make_service())
    .await
    .unwrap();

    Ok(())
}
