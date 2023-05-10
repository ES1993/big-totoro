use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    User,
    Group,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    id: String,
    #[serde(rename = "type")]
    ty: Type,
    form: String,
    to: String,
    body: String,
}

pub enum WSRecv {
    Message(Message)
}

pub enum WSRes{

}