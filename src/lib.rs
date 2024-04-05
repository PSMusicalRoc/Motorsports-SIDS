//! This crate is for the Rensselaer Motorsports'
//! computer login system. It was developed for
//! Spring 2024 RCOS by Tim Bishop \<bishot3@rpi.edu>.
//! 
//! The webserver itself is built on the Warp library,
//! using Websockets as its primary communications channel.
//! In addition to this, we use an Omnikey 5025CL RFID
//! reader by HID. There is also a self-created
//! communications library included (`omnikey_rs`), which,
//! as with the rest of this project, is free to use under
//! the GPLv2 license.

use omnikey_rs;

pub mod data_types;

/// Some global variables used by the server
pub mod global;

/// Functions dealing with websocket communications
pub mod websocket;
