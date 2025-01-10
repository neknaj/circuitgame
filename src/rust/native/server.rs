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


pub async fn main(input_path: String, output_path: Option<String>, module: Option<String>) {
    // tokioのbroadcastチャンネルを使用
    let (ws_tx, _ws_rx) = broadcast::channel::<String>(100); // websocket送信
    let (fc_tx, _fc_rx) = broadcast::channel::<String>(100); // ncg処理 (file change 通知)
    let (vmset_tx, _vmset_rx) = broadcast::channel::<u32>(100); // ncg処理 (file change 通知)

    // WebSocketサーバーを起動
    let ws_tx_clone = ws_tx.clone();
    let input_path_clone = input_path.clone();
    tokio::spawn(start_websocket_server(ws_tx_clone,input_path_clone));

    // キー入力監視用のタスク
    let ws_tx_clone = ws_tx.clone();
    let vmset_tx_clone = vmset_tx.clone();
    tokio::spawn(async move {
        key_watch(ws_tx_clone, vmset_tx_clone).await;
    });

    // ファイル監視タスクを起動
    let fc_tx_clone = fc_tx.clone();
    let ws_tx_clone = ws_tx.clone();
    let watch_path = input_path.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_file(watch_path,ws_tx_clone, fc_tx_clone).await {
            eprintln!("Error watching file: {}", e);
        }
    });

    // NCGの処理系を起動
    let ws_tx_clone = ws_tx.clone();
    let fc_tx_clone = fc_tx.clone();
    let vmset_tx_clone = vmset_tx.clone();
    tokio::spawn({ncg_tool(input_path,fc_tx_clone,vmset_tx_clone,ws_tx_clone,module)});

    tokio::signal::ctrl_c().await.unwrap();
    println!("Exit");
}

async fn watch_file(path: String, ws_tx: broadcast::Sender<String>, fc_tx: broadcast::Sender<String>) -> NotifyResult<()> {
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
        let _ = fc_tx.send(message);
        // 古いエントリを削除
        last_events.retain(|_, time| now.duration_since(*time) < debounce_duration);
    }

    Ok(())
}


async fn key_watch(ws_tx: broadcast::Sender<String>,vmset_tx: broadcast::Sender<u32>) {
    // デバウンス用の状態管理
    let debounce_duration = Duration::from_millis(100); // 100ms のデバウンス時間
    let mut last_events: HashMap<char, Instant> = HashMap::new();

    loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if let KeyCode::Char(c) = key_event.code {
                    let now = Instant::now();

                    // キーの最後のイベント時刻を確認
                    if let Some(last_time) = last_events.get(&c) {
                        if now.duration_since(*last_time) < debounce_duration {
                            // デバウンス期間内なのでスキップ
                            continue;
                        }
                    }

                    // 最終イベント時刻を更新
                    last_events.insert(c, now);

                    if let Ok(i) = c.to_string().parse::<u32>() {
                        let _ = vmset_tx.send(i);
                    }
                }
            }
        }

        // 古いエントリを削除
        let now = Instant::now();
        last_events.retain(|_, time| now.duration_since(*time) < debounce_duration);

        // 少し待つ
        sleep(Duration::from_millis(10)).await;
    }
}

async fn start_websocket_server(ws_tx: broadcast::Sender<String>,input_path: String) {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("WebSocket server running at ws://localhost:8081");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Error during WebSocket handshake");

        println!("New WebSocket connection!");

        let ws_tx = ws_tx.clone();
        let input_path_clone = input_path.clone();
        tokio::spawn(handle_connection(ws_stream, ws_tx, input_path_clone));
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ws_tx: broadcast::Sender<String>,
    input_path: String,
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
    let ws_tx_clone = ws_tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received message: {}", text);
                if (text.starts_with("get file")) { // websocketでファイルを送信
                    let input = match std::fs::read_to_string(&input_path) {
                        Ok(v) => v,
                        Err(e) => "fs error".to_string()
                    };
                    let _ = ws_tx_clone.send(format!("file:{}",input));
                }
            }
        }
    });

    // どちらかのタスクが終了するまで待機
    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }
}



async fn ncg_tool(input_path: String, fc_tx: broadcast::Sender<String>, vmset_tx: broadcast::Sender<u32>, ws_tx: broadcast::Sender<String>,module: Option<String>) {
    let mut rx = fc_tx.subscribe();  // メッセージ受信用のreceiverを作成
    loop {
        // inputを処理
        let binary = process_input(&input_path,module.clone());
        { // websocketでファイルを送信
            let input = match std::fs::read_to_string(&input_path) {
                Ok(v) => v,
                Err(e) => "fs error".to_string()
            };
            let _ = ws_tx.send(format!("file:{}",input));
        }

        let vmset_tx_clone = vmset_tx.clone();
        let ws_tx_clone = ws_tx.clone();

        tokio::select! {
            Ok(message) = rx.recv() => {
                println!("done {}", message);
                // ファイル変更を検知した場合の処理を追加
            }
            _ = runVM(binary, vmset_tx_clone,ws_tx_clone) => {
                println!("VM execution completed");
            }
        }
    }
}

// 入力処理を別関数として分離
fn process_input(input_path: &str,module: Option<String>) -> Vec<u32> {
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

    let module_name = match module {
        Some(v) => v,
        None => match result.module_dependency_sorted.get(0) {
            Some(v) => v.clone(),
            None => {return Vec::new();}
        }
    };
    println!("{}:{} Compiling module: {}","[info]".green(),"compile".cyan(),module_name);

    let binary = match compiler::serialize(result.clone(), module_name.as_str()) {
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


async fn runVM(data: Vec<u32>, vmset_tx: broadcast::Sender<u32>, ws_tx: broadcast::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut rx = vmset_tx.subscribe();
    use crate::vm::types::Module;
    let mut vm_module = match Module::new(data) {
        Ok(v) => v,
        Err(_) => {println!("failed to init VM");loop{}}
    };

    println!("");
    println!("VM start");
    println!("Press the number key in the index to switch inputs");
    println!("\n\n");
    loop {
        // まずVMを1ステップ実行
        let _ = vm_module.next(1);
        // outputをプリント
        println!("\x1B[4A\x1B[2K");
        println!("tick   {}",vm_module.get_tick());
        println!("input  {}",vm_module.get_input().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(" "));
        println!("output {}",vm_module.get_output().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(" "));
        // let _ = ws_tx.send(format!("tick:{},input:{:?},output:{:?}",
        //     vm_module.get_tick(),
        //     vm_module.get_input().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(""),
        //     vm_module.get_output().unwrap().iter().map(|&b| if b {"t"}else{"f"}).collect::<Vec<_>>().join(""),
        // ));
        // メッセージの確認（ノンブロッキング）
        if let Ok(index) = rx.try_recv() {
            // println!("Received VM setting: index={}",index);
            // ここでメッセージに基づく処理を実装
            // 例：VMの設定を更新するなど
            let _ = vm_module.inv(index);
        }
        // 必要に応じて短い待機を入れる
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}