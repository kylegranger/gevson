use crate::Gevson;
// use crate::ProofRequest;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use log::*;
// use serde::Serialize;
// use serde_json::from_str;
use std::thread;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error as Err};
use tungstenite::Message;
use tungstenite::Result as Res;

pub struct GevsonMsg {
    pub msg: String,
    // pub callback: Box<dyn Fn(String)>,
}

// use crate::args::{Args, Parser};
// use crate::handler::handler_echo::handle_echo;
// use crate::msg::msg_in::MsgIn;
// use crate::msg::msg_out::MsgOut;
// use crate::util::{get_msg_text, ToMessage};

// mod args;
// mod handler;
// mod msg;
// mod util;

pub fn get_msg_text(msg: &Message) -> Option<&str> {
    match msg {
        Message::Text(s) => Some(s),
        Message::Binary(v) => Some(std::str::from_utf8(v).expect("Invalid UTF8")),
        _ => None,
    }
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream, arc_gevson: Arc<Mutex<Gevson>>) {
    if let Err(e) = handle_connection(peer, stream, arc_gevson).await {
        match e {
            Err::ConnectionClosed | Err::Protocol(_) | Err::Utf8 => (),
            err => error!("Error processing connection: {:?}", err),
        }
    }
}

// async fn handle_msg(stream: &mut WebSocketStream<TcpStream>, msg: String) -> Res<()> {
//     let response = "My response is this: ".to_string() + &msg;
//     stream.send(Message::Text(response)).await?;

//     Ok(())
// }

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    arc_gevson: Arc<Mutex<Gevson>>,
) -> Res<()> {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    tracing::info!("New WebSocket connection: {}", peer);

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;

        if let Some(text) = get_msg_text(&msg) {
            tracing::info!("msg text: {}", text);
            let copy = ws_stream.clone();
            // handle_msg(&mut ws_stream, text.to_string()).await?;
            // let handler = |response| async {
            //     let stream = ws_stream;
            //     stream.send(Message::Text(response)).await;
            // };

            // let gevson_msg = GevsonMsg {
            //     msg: text.to_string(),
            //     callback: handler,
            // };
            // let mut gevson: std::sync::MutexGuard<'_, Gevson> = arc_gevson.lock().unwrap();
            // gevson.messages.push(gevson_msg);

            // ws_stream

            // let response = "My response is this: ".to_string() + &text;
            // ws_stream.send(Message::Text(response)).await?;
        }
    }

    Ok(())
}

// async fn handle_event(stream: &mut WebSocketStream<TcpStream>, event: MsgIn) -> Res<()> {
//     match event {
//         MsgIn::Echo(data) => {
//             stream
//                 .send(MsgOut::Echo(handle_echo(&data)).to_msg())
//                 .await?
//         }
//     };

//     Ok(())
// }

// pub fn start_ws_server() {
//     let ws_thread = thread::spawn(move || {
//         start_ws_server();
//     });
// }

pub async fn start_ws_server(
    arc_gevson: Arc<Mutex<Gevson>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3000".to_string();

    // Start server
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    tracing::info!("Listening on: {}", addr);

    // Handle connections
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        tracing::info!("Peer address: {}", peer);

        let gevson = arc_gevson.clone();
        // tokio::spawn(accept_connection(peer, stream, gevson));

        let work_thread = thread::spawn(move || {
            accept_connection(peer, stream, gevson);
        });
    }

    Ok(())
}
