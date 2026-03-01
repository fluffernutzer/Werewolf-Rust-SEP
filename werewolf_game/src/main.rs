#![forbid(unsafe_code)]
mod logic;
mod roles;
mod roles_logic;
mod tag_nacht;
mod ws;
use axum::{
    Router,
    routing::get,
};
use local_ip_address::local_ip;
use log::LevelFilter;
use qrcode::QrCode;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{Mutex, broadcast},
};

use crate::{
    logic::{Game, Phase},
};

struct PlayerDevice {
    name: String,
    token: String,
}
//zentrale Struktur, die alle wichtigen Daten spiechert
#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<Game>>,
    game_started: Arc<Mutex<bool>>,
    server_ip: String,
    play_dev: Arc<Mutex<Vec<PlayerDevice>>>,
    tx: broadcast::Sender<String>,
    endgame_signal: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}


#[tokio::main]

async fn main() {
    let (tx, _rx) = broadcast::channel(32);
    let (endgame_tx, endgame_rx) = tokio::sync::oneshot::channel::<()>(); // Zum beenden nötig
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
        .route("/", get(ws::index))//Startseite
        .route("/join", get(ws::join_page))//QR Code beitritts seite
        .route("/play/:token", get(ws::play_page))//Spieler seite auf mobilem gerät
        .route("/:username", get(ws::show_user))//Spieler Seite in Browser tab
        .route("/ws", get(ws::ws_handler))//Websocket Endpunkt
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
// Erstellt QR Code
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
