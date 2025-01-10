use super::super::test;
use super::super::vm;
use super::super::compiler;
use colored::*;

use crossterm::{
    cursor,
    terminal::{self, Clear, ClearType},
    ExecutableCommand, event,
    event::{Event, KeyCode, KeyModifiers},
};
use tokio::task;
use tokio::time::{sleep, Duration, self, Instant};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use warp::Filter;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use lazy_static::lazy_static;
use tokio_tungstenite::tungstenite::protocol::WebSocket;
use notify::{Watcher, RecursiveMode, Result as NotifyResult};
use std::path::Path;
use std::collections::HashMap;


pub async fn main(input_path: String) {
    // tokioのbroadcastチャンネルを使用
    let (ws_tx, _ws_rx) = broadcast::channel::<String>(100); // websocket送信
    let (fc_tx, _fc_rx) = broadcast::channel::<String>(100); // ncg処理 (file change 通知)

    // HTTPサーバーを起動
    tokio::spawn(start_http_server());
    // WebSocketサーバーを起動
    let ws_tx_clone = ws_tx.clone();
    tokio::spawn(start_websocket_server(ws_tx_clone));

    // キー入力監視用のタスク
    let ws_tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        key_watch(ws_tx_clone).await;
    });

    // ファイル監視タスクを起動
    let fc_tx_clone = fc_tx.clone();
    let watch_path = input_path.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_file(watch_path, fc_tx_clone).await {
            eprintln!("Error watching file: {}", e);
        }
    });

    // NCGの処理系を起動
    let fc_tx_clone = fc_tx.clone();
    tokio::spawn({ncg_tool(input_path,fc_tx_clone)});

    tokio::signal::ctrl_c().await.unwrap();
    println!("Exit");
}

async fn watch_file(path: String, fc_tx: broadcast::Sender<String>) -> NotifyResult<()> {
    let (watcher_ws_tx, mut watcher_rx) = mpsc::channel(100);
    let mut watcher = notify::recommended_watcher(move |res: NotifyResult<notify::Event>| {
        if let Ok(event) = res {
            let _ = watcher_ws_tx.blocking_send(event);
        }
    })?;
    watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;
    println!("Watching for changes in: {}", path);
    // デバウンス用の状態管理
    let mut last_events: HashMap<String, Instant> = HashMap::new();
    let debounce_duration = Duration::from_millis(500); // 500msのデバウンス時間
    while let Some(event) = watcher_rx.recv().await {
        let path_str = event.paths.first()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();
        let now = Instant::now();
        // パスごとに最後のイベント時刻を確認
        if let Some(last_time) = last_events.get(&path_str) {
            if now.duration_since(*last_time) < debounce_duration {
                // デバウンス期間内なのでスキップ
                continue;
            }
        }
        // 最終イベント時刻を更新
        last_events.insert(path_str.clone(), now);
        let message = format!("File change detected - Path: {}, Event: {:?}", path_str, event.kind);
        println!("{}", message);
        let _ = fc_tx.send(message);
        // 古いエントリを削除
        last_events.retain(|_, time| now.duration_since(*time) < debounce_duration);
    }

    Ok(())
}


async fn key_watch(ws_tx: broadcast::Sender<String>) {
    loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char(c), _) => {
                        println!("キー '{}' が押されました。", c);
                        let _ = ws_tx.send(format!("キー '{}' が押されました。", c));
                    }
                    _ => {
                        println!("他のキーが押されました");
                    }
                }
            }
        }
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

async fn start_websocket_server(ws_tx: broadcast::Sender<String>) {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("WebSocket server running at ws://localhost:8081");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Error during WebSocket handshake");

        println!("New WebSocket connection!");

        let ws_tx = ws_tx.clone();
        tokio::spawn(handle_connection(ws_stream, ws_tx));
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ws_tx: broadcast::Sender<String>,
) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut rx = ws_tx.subscribe();

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
                let _ = ws_tx.send(text);
            }
        }
    });

    // どちらかのタスクが終了するまで待機
    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }
}



async fn ncg_tool(input_path: String, fc_tx: broadcast::Sender<String>) {
    let mut rx = fc_tx.subscribe();  // メッセージ受信用のreceiverを作成
    loop {
        // inputを処理
        let binary = process_input(&input_path);
        tokio::select! {
            _ = async {
                // vmを実行
            } => {}
            result = rx.recv() => {
                if let Ok(msg) = result {
                    println!("Received message: {}", msg);
                    // メッセージに基づく処理
                }
            }
        }
    }
}

// 入力処理を別関数として分離
fn process_input(input_path: &str) -> Vec<u32> {
    // 画面クリア
    print!("\x1B[2J\x1B[1;1H");  // ANSIエスケープシーケンスでクリア

    // inputを読み込み
    println!("{}:{} input file: {}","[info]".green(),"input".cyan(),input_path);
    let input = match std::fs::read_to_string(input_path) {
        Ok(v) => v,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => println!("{}:{} File not found","[error]".red(),"arguments".cyan()),
                std::io::ErrorKind::PermissionDenied => println!("{}:{} Permission denied","[error]".red(),"arguments".cyan()),
                _ => println!("{}:{} {}","[error]".red(),"arguments".cyan(),e),
            };
            return Vec::new();
        }
    };

    // inputを処理
    let result = compiler::intermediate_products(&input);

    for i in &result.warns {
        println!("{}:{} {}","[warn]".yellow(),"compile".cyan(),i);
    }
    for i in &result.errors {
        println!("{}:{} {}","[error]".red(),"compile".cyan(),i);
    }
    println!("sortedDependency {:?}",&result.module_dependency_sorted);

    if result.errors.len() > 0 {
        return Vec::new();
    }

    let module = match result.module_dependency_sorted.get(0) {
        Some(v) => v.clone(),
        None => return Vec::new(),
    };

    let binary = match compiler::serialize(result.clone(), module.as_str()) {
        Ok(v) => v,
        Err(v) => {
            println!("{}:{} {}","[error]".red(),"serialize".cyan(),v);
            return Vec::new();
        }
    };

    let test_result = test::test(result);
    for i in &test_result.warns {
        println!("{}:{} {}","[warn]".yellow(),"test".cyan(),i);
    }
    for i in &test_result.errors {
        println!("{}:{} {}","[error]".red(),"test".cyan(),i);
    }

    binary
}