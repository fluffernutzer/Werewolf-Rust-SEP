#![forbid(unsafe_code)]
mod logic;
mod roles;
mod roles_logic;
mod tag_nacht;
mod ws;
use axum::{
    Router,
    //extract::{Form, Path, State},
    //response::{Html, Json, Redirect},
    routing::get,
};
//use image::Luma;
use local_ip_address::local_ip;
use log::LevelFilter;
use qrcode::QrCode;
//use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    //fs,
    //io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex, broadcast},
};
//use urlencoding::encode;
//use uuid::Uuid;
//use webbrowser;

use crate::{
    logic::{Game, Phase},
    //ws::{send_game_state, ws_handler},
};

/*#[derive(Deserialize)]
struct NameForm {
    username: String,
}*/
struct PlayerDevice {
    name: String,
    token: String,
}
#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<Game>>,
    game_started: Arc<Mutex<bool>>,
    server_ip: String,
    play_dev: Arc<Mutex<Vec<PlayerDevice>>>,
    tx: broadcast::Sender<String>,
    endgame_signal: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}
//#[derive(Deserialize)]
/*struct ActionForm {
    actor: String,
    action_kind: String,
    target: String,
}*/

#[tokio::main]

async fn main() {
    let (tx, _rx) = broadcast::channel(32);
    let (endgame_tx, endgame_rx) = tokio::sync::oneshot::channel::<()>();
    let ip = local_ip().unwrap().to_string();
    let tx_1 = tx.clone();
    let logger = ClientLogger { tx: tx_1 };
    log::set_boxed_logger(Box::new(logger)).expect("Logger konnte nicht gesetzt werden");
    log::set_max_level(LevelFilter::Info);
    let state = AppState {
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        tx,
        server_ip: ip.clone(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
        endgame_signal: Arc::new(Mutex::new(Some(endgame_tx))),
    };
    let app = Router::new()
        .route("/", get(ws::index))
        .route("/join", get(ws::join_page))
        .route("/play/:token", get(ws::play_page))
        .route("/:username", get(ws::show_user))
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    log::info!("Server läuft auf http://127.0.0.1:7878");
    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = endgame_rx.await;
            log::info!("EndGame-Signal empfangen.")
        })
        .await
        .unwrap();
}

fn generate_qr(ip: &str) -> String {
    let url = format!("http://{}:7878/join", ip);
    let code = QrCode::new(url.as_bytes()).unwrap();
    code.render::<qrcode::render::svg::Color>()
        .min_dimensions(220, 220)
        .build()
}

struct ClientLogger {
    tx: broadcast::Sender<String>,
}

impl log::Log for ClientLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let message = format!("{}", record.args());
        println!("{}", message);
        let chat_message = serde_json::json!({
            "type": "CHAT_MESSAGE",
            "data": {
                "sender": "Server",
                "message": message,
            }
        });
        let chat_message_str = serde_json::to_string(&chat_message)
            .expect("Fehler beim Serialisieren der Chat-Nachricht");
        let _ = self.tx.send(chat_message_str);
    }

    fn flush(&self) {}
}
