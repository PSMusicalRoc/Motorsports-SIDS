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
//! 
//! The application
//! starts by initializing logging and ensuring
//! the USB Card Reader is attached and working
//! properly.
//! 
//! Afterwards, we initialize the webserver on the
//! local network at port 8080, then loop on reading
//! the keycard until we get a response.
//! 
//! To see more, see the documentation for the
//! functions in [websocket].

use omnikey_rs;
pub mod data_types;
pub mod global;
pub mod websocket;
