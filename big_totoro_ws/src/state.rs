use crate::client::{Client, ClientMessage};
use big_totoro_core::result::AppResult;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

type ClientIdent = String;

#[derive(Clone)]
pub struct AppState {
    clients: Arc<RwLock<HashMap<ClientIdent, Client>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_client_sender(&self, client: &Client) {
        let mut clients = self.clients.write().await;
        clients.insert(client.ident(), client.clone());
    }

    pub async fn del_client_sender(&self, client: &Client) {
        let mut clients = self.clients.write().await;
        clients.remove(&client.ident());
    }

    pub async fn send_message(&self, id: &String, message: &ClientMessage) -> AppResult<()> {
        
        Ok(())
    }
}
