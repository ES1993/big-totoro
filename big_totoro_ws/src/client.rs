use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub enum ClientMessage {
    Close(String),
}

#[derive(Clone)]
pub struct Client {
    id: String,
    platform: String,
    pub sender: Sender<ClientMessage>,
}

impl Client {
    pub fn new(id: &str, platform: &str) -> (Self, Receiver<ClientMessage>) {
        let (sender, receiver) = mpsc::channel(100);

        (
            Client {
                id: id.to_string(),
                platform: platform.to_string(),
                sender: sender.clone(),
            },
            receiver,
        )
    }

    pub fn ident(&self) -> String {
        format!("{}:{}", self.platform, self.id)
    }
}
