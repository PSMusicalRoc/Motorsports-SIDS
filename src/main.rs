mod data_types;
mod websocket;
mod global;
mod omnikey_rs;

use futures::executor::block_on;

use data_types::WebsocketOutgoingMessage;
use global::*;
use websocket::*;

use std::{
    fs,
    process::exit
};
use warp::{filters::ws::Message, Filter};

use pretty_env_logger as pretty_log;
#[macro_use] extern crate log;


// fn print_help() {
//     println!("{}", "RM SIDS Webserver".bold().underline());
//     println!();
//     println!("{}", "Commands".bold());
//     println!("{}\t{}",
//         "message".bright_blue(),
//         "Sends a test message to all clients"
//     );
//     println!("{}\t{}",
//         "help".bright_blue(),
//         "Displays this help message"
//     );
//     println!("{}\t{}",
//         "exit".bright_blue(),
//         "Closes the webserver"
//     );
//     println!();
// }

#[tokio::main]
async fn main() {
    pretty_log::init();

    let mut lock = SETTINGS.lock().unwrap();
    *lock = toml::from_str(
        fs::read_to_string("settings.toml").unwrap().as_str()
    ).unwrap();
    drop(lock);

    match block_on(OMNIKEY.lock()).set_legacy_ccid_mode() {
        Ok(_) => {},
        Err(e) => {
            error!("Could not set Legacy Mode on RFID reader: {}", e);
            exit(-1);
        }
    }
    
    // let settings_warp = warp::any().map(move || settings.clone());
    
    info!("Launching RM Student ID Scan Server!");

    // let users: Users = Users::default();
    // let users = warp::any().map(move || users.clone());

    let index = warp::get()
        .and(warp::path!())
        .and(warp::fs::file("html/index.html"));

    let admin_page = warp::get()
        .and(warp::path!("adminPage"))
        .and(warp::fs::file("html/adminPage.html"));

    let login_page = warp::get()
        .and(warp::path!("login"))
        .and(warp::fs::file("html/login.html"));

    let static_dir = warp::path("static")
        .and(warp::fs::dir("html/static"));

    let error_page = warp::get()
        .and(warp::fs::file("html/error.html"));

    let websocket_route = warp::path("websocket")
        .and(warp::ws())
        // .and(settings_warp)
        .map(|socket: warp::ws::Ws| {
            socket.on_upgrade(|websocket: warp::ws::WebSocket| user_connected(websocket))
        });

    let routes = index
        .or(static_dir)
        .or(login_page)
        .or(admin_page)
        .or(websocket_route)
        .or(error_page);

    let _ = tokio::task::spawn(warp::serve(routes).run(([127, 0, 0, 1], 8080)));
    // let omnikey_task = tokio::task::spawn(omnikey_do(omnikey));

    loop {

        let mut last_id: u64 = 0;
        let mut still_reading = false;
        loop {
            let lock = match OMNIKEY.try_lock() {
                Ok(l) => l,
                Err(_) => continue
            };

            let data = match lock.check_for_rfid_card() {
                Ok(d) => d,
                Err(e) => {
                    error!("Error reading from Omnikey: {}", e);
                    continue;
                }
            };

            if data.status == 0 {
                if data.valid && last_id != data.id && !still_reading {
                    block_on(send_message(WebsocketOutgoingMessage {
                        msgtype: "parsing".to_string(),
                        message: "".to_string()
                    }));
                    block_on(user_message(0, Message::text(format!("rfid_scan {}", data.id).as_str())));
                    still_reading = true;
                    last_id = data.id;
                }
            } else {
                still_reading = false;
                last_id = 0;
            }
            drop(lock);
        }
    }

}
