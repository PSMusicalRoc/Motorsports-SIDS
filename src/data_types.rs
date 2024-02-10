use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Login {
    pub user: String,
    pub pass: String,
    pub database: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub login: Login
}



#[derive(Serialize, Deserialize)]
pub struct WebsocketOutgoingMessage {
    pub msgtype: String,
    pub message: String
}

/* SQL Data Types */

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct PersonRow {
    pub rcsid: String,
    pub firstname: String,
    pub lastname: String,
    pub rfid: String,
    pub is_good: bool
}

#[derive(sqlx::FromRow)]
pub struct JoinedPersonInShopSQL {
    pub rcsid: String,
    pub firstname: String,
    pub lastname: String,
    pub time_in: sqlx::types::chrono::DateTime<sqlx::types::chrono::Local>
}

#[derive(Serialize, Deserialize)]
pub struct JoinedPersonInShop {
    pub rcsid: String,
    pub firstname: String,
    pub lastname: String,
    pub timestamp: String
}
