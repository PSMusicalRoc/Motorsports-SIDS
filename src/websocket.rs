//! Websocket communication functions.
//! 
//! These are the functions that truly do the
//! heavy lifting of the application. They are
//! the backbone of the webserver and are the
//! main functions that are spawned off when
//! a user connects to the server.
//! 
//! These methods are not used much throughout
//! the rest of the code, but they are spawned
//! at the very beginning via [user_connected()].

use crate::omnikey_rs;

use crate::data_types::*;
use crate::global::*;

use log::{info, error};

use std::sync::atomic::{AtomicUsize, Ordering};

use sqlx::Row;
use sqlx::{
    mysql::*, ConnectOptions
};

use futures_util::{SinkExt, TryFutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use warp::ws::{Message, WebSocket};

/// Our global unique user id counter.
pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);


/// A small, non-async function to get settings from the
/// settings global variable
/// 
/// # Returns
/// A 3-string tuple:
/// - Tuple\[0] = username
/// - Tuple\[1] = password
/// - Tuple\[2] = database
/// 
/// # Examples
/// ```rust
/// async fn test() {
///     let (user, pass, database) = get_settings_data();
///     /* Create some connection here */
/// }
/// ```
fn get_settings_data() -> (String, String, String) {
    let lock = SETTINGS.lock().unwrap();
    let user = lock.login.user.clone();
    let pass = lock.login.pass.clone();
    let database = lock.login.database.clone();
    drop(lock);

    (user, pass, database)
}


/// A function that is called when a user connects to
/// our warp server. It adds them to the list of users,
/// and starts their asyncronous websocket task.
/// 
/// # Parameters
/// - `ws`: A warp::ws::Websocket instance
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

/// A function that fires when a user sends a
/// websocket message and we handle it. In here
/// is the giant if statement tree that we use
/// to handle every possible websocket request.
/// 
/// # Parameters
/// - `_my_id`: A `usize` that represents the id of the
/// user that sent the message.
/// - `msg`: The actual message sent by the user
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

    else if msg[0] == "rfid_scan" {
        let (user, pass, database) = get_settings_data();

        let opts = MySqlConnectOptions::new()
            .host("localhost")
            .username(&user)
            .password(&pass)
            .database(&database);
        let mut conn = opts.connect().await.unwrap();
        let data = sqlx::query_as::<_, PersonRow>(
            format!("SELECT * FROM people WHERE rfid=\"{}\"",
                msg[1]
            ).as_str()
        ).fetch_all(&mut conn).await.unwrap();

        // Check how many people match the rfid tag
        // in the database.

        if data.len() == 0 {
            new_msg.msgtype = "unknown_person".to_string();
            new_msg.message = "".to_string();
        } else if data.len() > 1 {
            new_msg.msgtype = "too_many_people_with_id".to_string();
            new_msg.message = "".to_string();
        } else {
            // There is only 1 person matching the rfid tag.
            let response = sqlx::query(
                format!("SELECT COUNT(*) as count FROM in_shop WHERE rfid=\"{}\"",
                    msg[1]
                ).as_str()
            ).fetch_all(&mut conn).await.unwrap();

            let mut pplcount: i32 = 0;
            for row in response.iter() {
                pplcount = row.get("count");
            }

            if pplcount == 0 {
                // There are no people matching that rfid in the shop currently.
                // As such, add them to the shop.

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
            } else {
                // There is such a person matching that rfid in the shop currently.
                // As such, remove them from the shop

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
            }

            let keycard_msg = WebsocketOutgoingMessage {
                msgtype: "rfid_success".to_string(),
                message: "".to_string()
            };
            send_message(keycard_msg).await;

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
    }

    else if msg[0] == "new_person" {
        let (user, pass, database) = get_settings_data();

        let opts = MySqlConnectOptions::new()
            .host("localhost")
            .username(&user)
            .password(&pass)
            .database(&database);
        let mut conn = opts.connect().await.unwrap();

        
        
        let firstname = msg[1].to_string();
        let lastname = msg[2].to_string();
        let rcsid = msg[3].to_string();
        let is_good = msg[4] == "true";

        let omnikey_lock = crate::global::OMNIKEY.lock().await;
        let mut last_scanned = crate::global::LAST_SCANNED_ID.lock().await;
        println!("waiting");
        let mut data: omnikey_rs::structs::ReaderData;
        loop {
            data = match omnikey_lock.check_for_rfid_card() {
                Ok(d) => d,
                Err(_) => continue
            };

            if data.status == 0 && data.valid {
                println!("Found ID# {}", data.id);
                let ppl_data = sqlx::query_as::<_, PersonRow>(
                    format!("SELECT * FROM people WHERE rfid=\"{}\"",
                        data.id
                    ).as_str()
                ).fetch_all(&mut conn).await.unwrap();

                println!("{}", ppl_data.len());

                if ppl_data.len() == 0 {
                    break;
                }
            }
        }
        
        println!("beginning add query");
        let _ = sqlx::query_as::<_, PersonRow>(
            format!("{} {}",
                "INSERT INTO people (rcsid, firstname, lastname, rfid, is_good)",
                format!("VALUES (\"{}\", \"{}\", \"{}\", \"{}\", {})",
                    rcsid,
                    firstname,
                    lastname,
                    data.id,
                    match is_good { true => "true", false => "false" }
                )
            ).as_str()
        ).fetch_all(&mut conn).await.unwrap();
        println!("ending add query");

        println!("Setting last scanned to this");
        *last_scanned = data.id;

        println!("done");
        drop(omnikey_lock);
        drop(last_scanned);
    }
    send_message(new_msg).await;
}


/// Sends a message to all clients currently connected
/// to our warp server.
/// 
/// # Parameters
/// - `message`: The message to be sent to all users
/// connected currently.
pub async fn send_message(message: WebsocketOutgoingMessage) {
    let new_msg: String = serde_json::to_string(&message).unwrap().to_string();

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

/// This function fires when a user disconnects from the
/// warp server.
/// 
/// # Parameters
/// - `my_id`: The `usize` corresponding with the user
/// that just disconnected.
pub async fn user_disconnected(my_id: usize) {
    info!("User {} left.", my_id);

    // Stream closed up, so remove from the user list
    
    USERS.write().await.remove(&my_id);
}