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
use local_ip_address::local_ip;
use qrcode::QrCode;
use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, broadcast},
};
use urlencoding::encode;
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
    tx: broadcast::Sender<String>,
    server_ip: String,
    play_dev: Arc<Mutex<Vec<PlayerDevice>>>,
}
#[derive(Deserialize)]
struct ActionForm {
    actor: String,
    target: String,
}

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
        .route("/:username", get(ws::show_user))
        .route("/ws", get(ws::ws_handler))
        .with_state(state);

    println!("Running on http://127.0.0.1:7878");
    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();


    axum::serve(listener, app).await.unwrap();
}

fn generate_qr(ip: &str) -> String {
    let url = format!("http://{}:7878", ip);
    let code = QrCode::new(url.as_bytes()).unwrap();
    //let image = code.render::<Luma<u8>>().build();

    //image.save("qr.png").unwrap();
    code.render::<qrcode::render::svg::Color>().min_dimensions(220,220).build()
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

    let page = template.replace("{{users}}", &users_html);
    Html(page)
}


async fn show_user(Path(username): Path<String>, State(state): State<AppState>) -> Html<String> {
    let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());
    let safe_username = htmlescape::encode_minimal(&username);

    let mut game = state.game.lock().await;
    let rolle_text = match game.rolle_von(&username) {
        Some(rolle) => format!("{:?}", rolle),
        None => "Unbekannt".to_string(),
    };

    let phase = game.phase.clone();
    let last_seher_result = game.last_seher_result.clone();

    let player_opt = game.players.iter_mut().find(|p| p.name == username);

    let (rolle_text, action_html) = if let Some(player) = player_opt {
        match player.rolle {
            crate::roles::Rolle::Werwolf => {
                if phase == crate::logic::Phase::WerwölfePhase
                    && player.lebend
                    && !player.bereits_gesehen
                {
                    (
                        "Werwolf".to_string(),
                        format!(
                            r#"
                            <h2>Werwolf-Aktion</h2>
                            <form action="/nacht/werwolf" method="post">
                                <input type="hidden" name="actor" value="{username}">
                                <input name="target" placeholder="Opfer">
                                <button>Töten</button>
                            </form>
                            "#,
                            username = safe_username
                        ),
                    )
                } else {
                    (
                        "Werwolf".to_string(),
                        "<p> Aktion abgeschlossen. Bitte warte auf die nächste Phase.</p>"
                            .to_string(),
                    )
                }
            }
            crate::roles::Rolle::Seher => {
                if phase == crate::logic::Phase::SeherPhase
                    && player.lebend
                    && !player.bereits_gesehen
                {
                    (
                        "Seher".to_string(),
                        format!(
                            r#"
                            <h2>Seher-Aktion</h2>
                            <form action="/nacht/seher" method="post">
                                <input type="hidden" name="actor" value="{username}">
                                <input name="target" placeholder="Spieler">
                                <button>Schauen</button>
                            </form>
                            "#,
                            username = safe_username
                        ),
                    )
                } else if let Some((target, rolle)) = &last_seher_result {
                    (
                        "Seher".to_string(),
                        format!(
                            "<p> Du hast bereits geschaut: {} ist ein {:?}</p>",
                            htmlescape::encode_minimal(target),
                            rolle
                        ),
                    )
                } else {
                    (
                        "Seher".to_string(),
                        "<p> Aktion abgeschlossen. Bitte warte auf die nächste Phase.</p>"
                            .to_string(),
                    )
                }
            }
            _ => ("Dorfbewohner".to_string(), String::new()),
        }
    } else {
        ("Unbekannt".to_string(), String::new())
    };

    let user_page = template
        .replace("{{username}}", &safe_username)
        .replace("{{rolle}}", &rolle_text)
        .replace("{{aktion}}", &action_html);

    Html(user_page)
}


async fn add_user(State(state): State<AppState>, Form(form): Form<NameForm>) -> Redirect {
    let mut started = state.game_started.lock().await;
    if *started {
        return Redirect::to("/");
    }
    let mut game = state.game.lock().await;
    game.add_player(form.username.clone());
    send_game_state(&state).await;
    Redirect::to("/")
}


async fn start_game(State(state): State<AppState>) -> Html<String> {
    let mut started = state.game_started.lock().await;
    *started = true;
    let mut game_logic = state.game.lock().await;
    game_logic.verteile_rollen();
  
    for p in game_logic.players.iter() {
        let safe_username = encode(&p.name);
        let url = format!("http://127.0.0.1:7878/{}", safe_username);
        let _ = webbrowser::open(&url);
    }
    let template = tokio::fs::read_to_string("start_game.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());

    let users_html: String = game_logic
        .players
        .iter()
        .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(&u.name)))
        .collect::<Vec<_>>()
        .join("\n");
    let phase = format!("{:?}", game_logic.phase);

    let game_page = template
        .replace("{{users}}", &users_html)
        .replace("{{phase}}", &phase);

    Html(game_page)
}
