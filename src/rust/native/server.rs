use super::super::test;
use super::super::vm;
use super::super::compiler;
use colored::*;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tokio::task;
use tokio::time::{sleep, Duration, self};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use warp::Filter;
use futures_util::{SinkExt, StreamExt};

pub async fn main() {
    // tokioのbroadcastチャンネルを使用
    let (tx, _rx) = broadcast::channel::<String>(100);

    // HTTPサーバーを起動
    tokio::spawn(start_http_server());

    // WebSocketサーバーを起動
    let tx_clone = tx.clone();
    tokio::spawn(start_websocket_server(tx_clone));

    // キー入力監視用のタスク
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        key_watch(tx_clone).await;
    });

    tokio::signal::ctrl_c().await.unwrap();
    println!("Exit");
}

async fn key_watch(tx: broadcast::Sender<String>) {
    loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char(c), _) => {
                        println!("キー '{}' が押されました。", c);
                        let _ = tx.send(format!("キー '{}' が押されました。", c));
                    }
                    _ => {
                        println!("他のキーが押されました");
                    }
                }
            }
        }
        // 他の処理をブロックしないように少し待機
        sleep(Duration::from_millis(10)).await;
    }
}

async fn start_http_server() {
    let index = warp::path::end().map(|| {
        warp::reply::html(include_str!("index.html"))
    });

    warp::serve(index)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

async fn start_websocket_server(tx: broadcast::Sender<String>) {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("WebSocket server running at ws://localhost:8081");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Error during WebSocket handshake");

        println!("New WebSocket connection!");

        let tx = tx.clone();
        tokio::spawn(handle_connection(ws_stream, tx));
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    tx: broadcast::Sender<String>,
) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut rx = tx.subscribe();

    // メッセージ送信用タスク
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // メッセージ受信用タスク
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received message: {}", text);
                let _ = tx.send(text);
            }
        }
    });

    // どちらかのタスクが終了するまで待機
    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }
}