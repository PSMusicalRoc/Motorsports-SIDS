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