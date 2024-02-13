use super::data_types::*;
use lazy_static::lazy_static;

use std::sync::Mutex;

lazy_static!{
    pub static ref USERS: Users = Users::default();
    pub static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
}