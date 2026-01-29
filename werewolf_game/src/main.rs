#![forbid(unsafe_code)]
mod logic;
mod roles;
mod tag_nacht;
mod ws;
use axum::{
    Router,
    extract::{Form, Path, State},
    response::{Html, Json, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, broadcast},
};
use urlencoding::encode;
use local_ip_address::local_ip;
use qrcode::QrCode;
use image::Luma;
use uuid::Uuid;
use webbrowser;

use crate::{logic::{Game, Phase}, ws::{send_game_state, ws_handler}};

#[derive(Deserialize)]
struct NameForm {
    username: String,
}
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
#[derive(Deserialize)]
struct ActionForm {
    actor: String,
    action_kind: String,
    target: String,
}

#[tokio::main]

async fn main() {
    let (tx, _rx) = broadcast::channel(32);
    let state = AppState {
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        tx
    };
    let app = Router::new()
        .route("/", get(ws::index))
        .route("/:username", get(ws::show_user))
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    println!("Running on http://127.0.0.1:7878");
    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}


async fn index(State(state): State<AppState>) -> Html<String> {
    let template = tokio::fs::read_to_string("hello.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());

    let game = state.game.lock().await;
    let users_html: String = game
        .players
        .iter()
        .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(&u.name)))
        .collect();
    let join_url = format!("http://{}:7878", state.server_ip);
    let qr_svg=generate_qr(&state.server_ip);
    let page = template.replace("{{users}}", &users_html).replace("{{join_url}}", &join_url).replace("{{qr}}", &qr_svg);
    Html(page)
}

fn generate_qr(ip: &str) -> String {
    let url = format!("http://{}:7878", ip);
    let code = QrCode::new(url.as_bytes()).unwrap();
    code.render::<qrcode::render::svg::Color>().min_dimensions(220,220).build()
}

