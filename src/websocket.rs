use super::data_types::*;
use super::global::*;

use std::sync::atomic::{AtomicUsize, Ordering};

use sqlx::{
    mysql::*, ConnectOptions
};

use futures_util::{SinkExt, TryFutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use warp::ws::{Message, WebSocket};

/// Our global unique user id counter.
pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

fn get_settings_data() -> (String, String, String) {
    let lock = SETTINGS.lock().unwrap();
    let user = lock.login.user.clone();
    let pass = lock.login.pass.clone();
    let database = lock.login.database.clone();
    drop(lock);

    (user, pass, database)
}

pub async fn user_connected(ws: WebSocket) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    info!("User connected: {}", my_id);

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
                    error!("Websocket failed to send: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    USERS.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id).await;
}

pub async fn user_message(_my_id: usize, msg: Message) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        info!("Message Recieved: {}", s);
        s
    } else {
        return;
    };

    let mut new_msg = WebsocketOutgoingMessage {
        msgtype: "null".to_string(),
        message: "".to_string()
    };

    let msg: Vec<&str> = msg.split(" ").collect();

    if msg[0] == "hello" {
        new_msg.msgtype = "message".to_string();
        new_msg.message = "hello world!".to_string();
    } else if msg[0] == "pressed_a_button" {
        new_msg.msgtype = "message".to_string();
        new_msg.message = "you sure did, champ".to_string();
    } else if msg[0] == "test_message" {
        new_msg.msgtype = "in_shop_add".to_string();
        new_msg.message = "[]".to_string(); 
    } else if msg[0] == "get_in_shop" {
        
        let (user, pass, database) = get_settings_data();

        let opts = MySqlConnectOptions::new()
            .host("localhost")
            .username(&user)
            .password(&pass)
            .database(&database);
        let mut conn = opts.connect().await.unwrap();
        let data = sqlx::query_as::<_, JoinedPersonInShopSQL>(
            format!("{} {} {}",
                "select people.rcsid, people.firstname, people.lastname, people.rfid, in_shop.time_in",
                "from people",
                "inner join in_shop on in_shop.rfid=people.rfid").as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let mut realdata: Vec<JoinedPersonInShop> = Vec::new();

        for obj in data {
            realdata.push(JoinedPersonInShop {
                rcsid: obj.rcsid,
                firstname: obj.firstname,
                lastname: obj.lastname,
                timestamp: format!("{} {}",
                    obj.time_in.date_naive(),
                    obj.time_in.time()
                )
            })
        }

        new_msg.msgtype = "in_shop_refresh".to_string();
        new_msg.message = serde_json::to_string(&realdata).unwrap();
    } else if msg[0] == "get_all_timestamps" {
        let (user, pass, database) = get_settings_data();

        let opts = MySqlConnectOptions::new()
            .host("localhost")
            .username(&user)
            .password(&pass)
            .database(&database);
        let mut conn = opts.connect().await.unwrap();
        let data = sqlx::query_as::<_, JoinedTimestampSQL>(
            format!("{} {} {}",
                "select people.rcsid, people.firstname, people.lastname, people.rfid, timestamps.is_checking_in, timestamps.time_stamp",
                "from people",
                "inner join timestamps on timestamps.rfid=people.rfid").as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let mut realdata: Vec<JoinedTimestamp> = Vec::new();

        for obj in data {
            println!("{}", obj.firstname);
            realdata.push(JoinedTimestamp {
                rcsid: obj.rcsid,
                firstname: obj.firstname,
                lastname: obj.lastname,
                entering: obj.is_checking_in,
                timestamp: format!("{} {}",
                    obj.time_stamp.date_naive(),
                    obj.time_stamp.time()
                )
            })
        }

        new_msg.msgtype = "timestamps_refresh".to_string();
        new_msg.message = serde_json::to_string(&realdata).unwrap();
        println!("{}", new_msg.message);
    }
    
    else if msg[0] == "add_to_shop" {
        let (user, pass, database) = get_settings_data();

        let opts = MySqlConnectOptions::new()
            .host("localhost")
            .username(&user)
            .password(&pass)
            .database(&database);
        let mut conn = opts.connect().await.unwrap();
        let _ = sqlx::query_as::<_, JoinedPersonInShopSQL>(
            format!("INSERT INTO in_shop (rfid, time_in) VALUES (\"{}\", current_timestamp())",
                msg[1]
            ).as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let _ = sqlx::query(
            format!("{} SELECT rfid, TRUE, time_in FROM in_shop WHERE rfid=\"{}\"",
                "INSERT INTO timestamps (rfid, is_checking_in, time_stamp)",
                msg[1]
            ).as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let data = sqlx::query_as::<_, JoinedPersonInShopSQL>(
            format!("{} {} {}",
                "select people.rcsid, people.firstname, people.lastname, people.rfid, in_shop.time_in",
                "from people",
                "inner join in_shop on in_shop.rfid=people.rfid").as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let mut realdata: Vec<JoinedPersonInShop> = Vec::new();

        for obj in data {
            realdata.push(JoinedPersonInShop {
                rcsid: obj.rcsid,
                firstname: obj.firstname,
                lastname: obj.lastname,
                timestamp: format!("{} {}",
                    obj.time_in.date_naive(),
                    obj.time_in.time()
                )
            })
        }

        new_msg.msgtype = "in_shop_refresh".to_string();
        new_msg.message = serde_json::to_string(&realdata).unwrap();
    }
    
    else if msg[0] == "remove_from_shop" {
        let (user, pass, database) = get_settings_data();

        let opts = MySqlConnectOptions::new()
            .host("localhost")
            .username(&user)
            .password(&pass)
            .database(&database);
        let mut conn = opts.connect().await.unwrap();
        let _ = sqlx::query(
            format!("DELETE FROM in_shop WHERE rfid=\"{}\"",
                msg[1]
            ).as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let _ = sqlx::query(
            format!("{} VALUES (\"{}\", FALSE, CURRENT_TIMESTAMP())",
                "INSERT INTO timestamps (rfid, is_checking_in, time_stamp)",
                msg[1]
            ).as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let data = sqlx::query_as::<_, JoinedPersonInShopSQL>(
            format!("{} {} {}",
                "select people.rcsid, people.firstname, people.lastname, people.rfid, in_shop.time_in",
                "from people",
                "inner join in_shop on in_shop.rfid=people.rfid").as_str()
        ).fetch_all(&mut conn).await.unwrap();

        let mut realdata: Vec<JoinedPersonInShop> = Vec::new();

        for obj in data {
            realdata.push(JoinedPersonInShop {
                rcsid: obj.rcsid,
                firstname: obj.firstname,
                lastname: obj.lastname,
                timestamp: format!("{} {}",
                    obj.time_in.date_naive(),
                    obj.time_in.time()
                )
            })
        }

        new_msg.msgtype = "in_shop_refresh".to_string();
        new_msg.message = serde_json::to_string(&realdata).unwrap();
    }

    let new_msg: String = serde_json::to_string(&new_msg).unwrap().to_string();

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in USERS.read().await.iter() {
        info!("Sending message containing the following string to client {}: \"{}\"", uid, new_msg);
        if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
            info!("Sent!");
            continue;
        }
    }
}

pub async fn user_disconnected(my_id: usize) {
    info!("User {} left.", my_id);

    // Stream closed up, so remove from the user list
    USERS.write().await.remove(&my_id);
}