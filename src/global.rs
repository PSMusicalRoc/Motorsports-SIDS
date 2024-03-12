use super::data_types::*;
use lazy_static::lazy_static;
use crate::omnikey_rs::structs::Reader;

// use std::sync::Mutex;
// use tokio::sync::Mutex;

lazy_static!{
    pub static ref USERS: Users = Users::default();
    pub static ref SETTINGS: std::sync::Mutex<Settings> = std::sync::Mutex::new(Settings::default());
    pub static ref OMNIKEY: tokio::sync::Mutex<Reader> = tokio::sync::Mutex::new(Reader::new().unwrap());
}