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
    routing::{get},
};
//use image::Luma;
use local_ip_address::local_ip;
use qrcode::QrCode;
//use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    //fs,
    //io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener},
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
    let ip = local_ip().unwrap().to_string();
    let state = AppState {
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        tx,
        server_ip: ip.clone(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
    let app = Router::new()
        .route("/", get(ws::index))
        .route("/join", get(ws::join_page))
        .route("/play/:token", get(ws::play_page))
        .route("/:username", get(ws::show_user))
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    println!("Running on http://127.0.0.1:7878");
    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn generate_qr(ip: &str) -> String {
    let url = format!("http://{}:7878/join", ip);
    let code = QrCode::new(url.as_bytes()).unwrap();
    code.render::<qrcode::render::svg::Color>()
        .min_dimensions(220, 220)
        .build()
}