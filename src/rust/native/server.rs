
use super::super::test;
use super::super::vm;
use super::super::compiler;
use colored::*;

use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use warp::Filter;
use futures_util::{SinkExt, StreamExt}; // StreamExt をインポート

pub async fn main() {
    // HTTPサーバー
    tokio::spawn(start_http_server());
    // WebSocketサーバー
    tokio::spawn(start_websocket_server());
    // サーバーが動作し続けるために待機
    tokio::signal::ctrl_c().await.unwrap();
}


async fn start_http_server() {
    // `index.html`を返すHTTPサーバー
    let index = warp::path::end().map(|| {
        warp::reply::html(include_str!("index.html"))
    });

    // HTTPサーバーの開始
    warp::serve(index)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

async fn start_websocket_server() {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("WebSocket server running at ws://localhost:8081");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Error during WebSocket handshake");

        println!("New WebSocket connection!");

        tokio::spawn(handle_websocket(ws_stream));
    }
}

async fn handle_websocket(ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split(); // `StreamExt` トレイトが必要

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            println!("Received message: {}", text);

            // 受け取ったメッセージをオウム返し
            if ws_sender.send(Message::Text(text)).await.is_err() {
                println!("Error sending message");
                break;
            }
        }
    }
}