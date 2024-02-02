use futures_util::{StreamExt, FutureExt};

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{SinkExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

use pretty_env_logger as pretty_log;
#[macro_use] extern crate log;

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    warn!("New User: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(_my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        println!("recieved a message: {}", s);
        s
    } else {
        return;
    };

    let new_msg: &str;

    if msg == "hello" {
        new_msg = "hi there!";
    } else if msg == "pressed_a_button" {
        new_msg = "you sure did, champ";
    } else {
        new_msg = "huh";
    }

    // New message from this user, send it to everyone else (except same uid)...
    for (&_uid, tx) in users.read().await.iter() {
        if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

#[tokio::main]
async fn main() {
    pretty_log::init();
    
    info!("Hello world!");

    let users: Users = Users::default();
    let users = warp::any().map(move || users.clone());

    let index = warp::get()
        .and(warp::path!())
        .and(warp::fs::file("html/index.html"));

    let static_dir = warp::path("static")
        .and(warp::fs::dir("html/static"));

    let websocket_route = warp::path("websocket")
        .and(warp::ws())
        .and(users)
        .map(|socket: warp::ws::Ws, users| {
            socket.on_upgrade(|websocket: warp::ws::WebSocket| user_connected(websocket, users))
        });

    let routes = index
        .or(static_dir)
        .or(websocket_route);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
