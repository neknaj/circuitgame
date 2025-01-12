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


pub async fn main(input_path: String, output_path: Vec<String>, doc_output_path: Option<String>, module: Option<String>, run_vm: bool, watch: bool, server: bool, server_port: Option<String>) {
    // tokioのbroadcastチャンネルを使用
    let (ws_tx, _ws_rx) = broadcast::channel::<String>(100); // websocket送信
    let (fc_tx, _fc_rx) = broadcast::channel::<String>(100); // ncg処理 (file change 通知)
    let (vmset_tx, _vmset_rx) = broadcast::channel::<u32>(100); // ncg処理 (file change 通知)

    let mut server_msg = if server {
            // WebSocketサーバーを起動
            let ws_tx_clone = ws_tx.clone();
            let input_path_clone = input_path.clone();
            start_websocket_server(ws_tx_clone,input_path_clone,server_port).await
        }
        else {
            Err("disabled".to_string())
        };
    // serverの起動に失敗したとき
    match &server_msg {
        Ok(_) => {},
        Err(v) => {
            use colored::*;
            if !(watch|run_vm) {
                let _ = super::common::process_input(&input_path, module,output_path,doc_output_path );
                println!("{}:{} {}","[error]".red(),"webSock".cyan(),v);
                return;
            }
        }
    }

    if watch {
        // ファイル監視タスクを起動
        let fc_tx_clone = fc_tx.clone();
        let ws_tx_clone = ws_tx.clone();
        let watch_path = input_path.clone();
        tokio::spawn(async move {
            if let Err(e) = watch_file(watch_path,ws_tx_clone, fc_tx_clone).await {
                eprintln!("Error watching file: {}", e);
            }
        });
    }

    // キー入力監視用のタスク
    let ws_tx_clone = ws_tx.clone();
    let vmset_tx_clone = vmset_tx.clone();
    tokio::spawn(async move {
        key_watch(ws_tx_clone, vmset_tx_clone).await;
    });

    // NCGの処理系を起動
    let ws_tx_clone = ws_tx.clone();
    let fc_tx_clone = fc_tx.clone();
    let vmset_tx_clone = vmset_tx.clone();
    tokio::spawn({ncg_tool(input_path,fc_tx_clone,vmset_tx_clone,ws_tx_clone,module,output_path,doc_output_path,server_msg, run_vm,watch,server)});

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
    let debounce_duration = Duration::from_millis(1); // 1msのデバウンス時間
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
        { // websocketでファイルを送信
            let input = match std::fs::read_to_string(&path) {
                Ok(v) => v,
                Err(e) => "fs error".to_string()
            };
            let _ = ws_tx.send(format!("file:{}",input));
        }
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

async fn start_websocket_server(ws_tx: broadcast::Sender<String>,input_path: String, server_port: Option<String>) -> Result<String,String> {
    let port = match server_port.unwrap_or("8080".to_string()) {
        v if v=="true" => "8080".to_string(),
        v => v,
    };
    let listener = match TcpListener::bind(format!("localhost:{}",port)).await {
        Ok(v) => v,
        Err(v) => { return Err(format!("{}: {}",v,port)); },
    };

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during WebSocket handshake");

            // println!("New WebSocket connection!");

            let ws_tx = ws_tx.clone();
            let input_path_clone = input_path.clone();
            tokio::spawn(handle_connection(ws_stream, ws_tx, input_path_clone));
        }
    });
    Ok(format!(
        "Server running at {}",
        format!(
            "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            format!("https://neknaj.github.io/circuitgame/?socket=ws://localhost:{}",port),
            format!("ws://localhost:{}",port),
        )
    ))
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
                // println!("Received message: {}", text);
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



async fn ncg_tool(input_path: String, fc_tx: broadcast::Sender<String>, vmset_tx: broadcast::Sender<u32>, ws_tx: broadcast::Sender<String>,module: Option<String>, output_path: Vec<String>, doc_output_path: Option<String>, server_msg: Result<String,String>, run_vm: bool, watch: bool, server: bool) {
    let mut rx = fc_tx.subscribe();  // メッセージ受信用のreceiverを作成
    loop {
        // inputを処理
        // 画面クリア
        print!("\x1B[2J\x1B[1;1H");  // ANSIエスケープシーケンスでクリア
        use colored::*;
        println!("{}:{} {}","[info]".green(),"ncg".cyan(),format!("watch: {} server: {} vm: {}",match watch{true=>"on".cyan(),false=>"off".blue()},match server{true=>"on".cyan(),false=>"off".blue()},match run_vm{true=>"on".cyan(),false=>"off".blue()}));
        if server {
            match &server_msg {
                Ok(msg) => {
                    println!("{}:{} {}","[info]".green(),"webSock".cyan(),msg);
                }
                Err(msg) => {
                    println!("{}:{} {}","[error]".red(),"webSock".cyan(),msg);
                }
            }
        }
        println!("");

        let binary = super::common::process_input(&input_path,module.clone(),output_path.clone(),doc_output_path.clone());

        let vmset_tx_clone = vmset_tx.clone();
        let ws_tx_clone = ws_tx.clone();

        tokio::select! {
            Ok(message) = rx.recv() => {
                // println!("done {}", message);
                // ファイル変更を検知した場合の処理を追加
            }
            _ = async {
                if run_vm {
                    if (!match module.clone() {Some(_)=>true,_=>false}) {
                        println!("{}:{} {}","[error]".red(),"output".cyan(),format!("Output module was not specified for VM"));
                        sleep(Duration::from_secs(100)).await;
                        return;
                    }
                    tokio::select! {
                        vm_res = super::common::runVM(binary, vmset_tx_clone,ws_tx_clone) => {
                            match vm_res {
                                Ok(_) => {},
                                Err(_) => { sleep(Duration::from_secs(100)).await; },
                            }
                        }
                    }
                }
                else {
                    sleep(Duration::from_secs(100)).await;
                }
            } => {}
        }
    }
}
