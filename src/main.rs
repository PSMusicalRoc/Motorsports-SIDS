mod data_types;
mod websocket;
mod global;

use global::*;

use websocket::*;

use colored::Colorize;
use std::fs;
use text_io::read;
use warp::{filters::ws::Message, Filter};

use pretty_env_logger as pretty_log;
#[macro_use] extern crate log;


fn print_help() {
    println!("{}", "RM SIDS Webserver".bold().underline());
    println!();
    println!("{}", "Commands".bold());
    println!("{}\t{}",
        "help".bright_blue(),
        "Displays this help message"
    );
    println!("{}\t{}",
        "exit".bright_blue(),
        "Closes the webserver"
    );
    println!();
}


#[tokio::main]
async fn main() {
    pretty_log::init();

    let mut lock = SETTINGS.lock().unwrap();
    *lock = toml::from_str(
        fs::read_to_string("settings.toml").unwrap().as_str()
    ).unwrap();
    drop(lock);
    
    // let settings_warp = warp::any().map(move || settings.clone());
    
    info!("Launching RM Student ID Scan Server!");

    // let users: Users = Users::default();
    // let users = warp::any().map(move || users.clone());

    let index = warp::get()
        .and(warp::path!())
        .and(warp::fs::file("html/index.html"));

    let static_dir = warp::path("static")
        .and(warp::fs::dir("html/static"));

    let websocket_route = warp::path("websocket")
        .and(warp::ws())
        // .and(settings_warp)
        .map(|socket: warp::ws::Ws| {
            socket.on_upgrade(|websocket: warp::ws::WebSocket| user_connected(websocket))
        });

    let routes = index
        .or(static_dir)
        .or(websocket_route);

    let webserver = tokio::task::spawn(warp::serve(routes).run(([127, 0, 0, 1], 8080)));

    loop {
        let line: String = read!("{}\n");
        let words: Vec<&str> = line.split_ascii_whitespace().collect();

        if words.len() < 1 { continue; }

        match words[0] {
            "help" => print_help(),
            "exit" => {
                webserver.abort();
                break;
            },
            "message" => {
                user_message(0, Message::text("test_message")).await;
            },
            _ => {
                println!("Incorrect command - type \"help\" to see all commands.");
                println!();
            }
        }
    }

    info!("Shutting down webserver!");

}
