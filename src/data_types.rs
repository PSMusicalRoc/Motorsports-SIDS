use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub user: String,
    pub pass: String,
    pub database: String
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub login: Login
}



#[derive(Serialize, Deserialize)]
pub struct WebsocketOutgoingMessage {
    pub msgtype: String,
    pub message: String
}



#[derive(Serialize, Deserialize)]
pub struct PersonRow {
    pub rcsid: String,
    pub firstname: String,
    pub lastname: String,
    pub rfid: String,
    pub is_good: bool
}