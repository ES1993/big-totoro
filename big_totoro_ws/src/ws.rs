use crate::{
    client::{Client, ClientMessage},
    state::AppState,
};
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use big_totoro_core::{
    config::Config,
    result::{AppError, AppResult},
    token::Claims,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Receiver;

pub async fn handler(
    State(state): State<AppState>,
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> AppResult<impl IntoResponse> {
    let platform = params
        .get("platform")
        .ok_or(AppError::BadRequest("缺少platform".to_string()))?;

    let token = params
        .get("token")
        .ok_or(AppError::BadRequest("缺少token".to_string()))?;

    let id = Claims::decode(token, &config.token_secret)?.id;

    if !config.allow_platform.contains(platform) {
        return Err(AppError::BadRequest("此平台未开发".to_string()));
    }

    let (client, client_receiver) = Client::new(&id, &platform);

    Ok(ws.on_upgrade(move |socket| async move {
        state.add_client_sender(&client).await;

        let temp = client.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            temp.sender
                .send(ClientMessage::Close("aaaaaa".to_string()))
                .await
                .unwrap();
        });

        if let Err(e) = handle_socket(socket, client_receiver).await {
            println!("ws error2:{:?}", e);
        }

        state.del_client_sender(&client).await;
    }))
}

async fn handle_socket(
    socket: WebSocket,
    mut client_receiver: Receiver<ClientMessage>,
) -> AppResult<()> {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let mut client_receiver_task = tokio::spawn(async move {
        let mut close_message = "正常关闭".to_string();

        while let Some(msg) = client_receiver.recv().await {
            match msg {
                ClientMessage::Close(msg) => {
                    close_message = msg;
                    break;
                }
            }
        }

        ws_sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Cow::from(close_message),
            })))
            .await?;

        ws_sender.close().await?;

        Ok::<(), AppError>(())
    });

    let mut ws_receiver_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("Text:{text}");
                }
                Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => (),
                Message::Close(_) => break,
            }
        }
        Ok::<(), AppError>(())
    });

    tokio::select! {
        _ = (&mut ws_receiver_task) => {
            client_receiver_task.abort();
        },
        _ = (&mut client_receiver_task) => {
            ws_receiver_task.abort();
        },
    }

    Ok(())
}
