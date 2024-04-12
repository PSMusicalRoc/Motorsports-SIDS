//! Global structure references for very
//! used objects.
//! 
//! Much as I didn't want to have global objects,
//! for a couple specific uses, it made the most sense.
//! As such, we have global instances of:
//! - The actual Omnikey Reader itself, [struct@OMNIKEY]
//! - The server connection settings, like the
//! username, password, and database, stored in
//! [struct@SETTINGS]
//! - The list of users, stored in [struct@USERS]
//! 
//! These were mainly stowed into global instances
//! solely for multithread use. All of these were,
//! at some point, required in multiple threads. As
//! such, they became `lazy_static` references to
//! allow them to be used safely across threads.

use super::data_types::*;
use lazy_static::lazy_static;
use crate::omnikey_rs::structs::Reader;

// use std::sync::Mutex;
// use tokio::sync::Mutex;

lazy_static!{
    pub static ref USERS: Users = Users::default();
    pub static ref SETTINGS: std::sync::Mutex<Settings> = std::sync::Mutex::new(Settings::default());
    pub static ref OMNIKEY: tokio::sync::Mutex<Reader> = tokio::sync::Mutex::new(Reader::new().unwrap());
    pub static ref LAST_SCANNED_ID: tokio::sync::Mutex<u64> = tokio::sync::Mutex::new(0);
}