#![forbid(unsafe_code)]
mod logic;
use axum::{
    Router,
    extract::{
        Form, Path, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::{Html, IntoResponse, Json, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::to_string;
use std::sync::Arc;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, broadcast},
};
use urlencoding::encode;
use webbrowser;

use crate::logic::{Game, Phase};

#[derive(Deserialize)]
struct NameForm {
    username: String,
}
#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<Game>>,
    game_started: Arc<Mutex<bool>>,
    tx: broadcast::Sender<String>,
}
#[derive(Deserialize)]
struct ActionForm {
    actor: String,
    target: String,
}
#[tokio::main]

async fn main() {
    let (tx, _) = broadcast::channel(32);
    let state = AppState {
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        tx,
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/add-user", post(add_user))
        .route("/start-game", post(start_game))
        .route("/:username", get(show_user))
        .route("/tag", post(tag_action))
        .route("/nacht/werwolf", post(werwolf_action))
        .route("/nacht/seher", post(seher_action))
        .route("/ws", get(ws_handler))
        .with_state(state);

    println!("Running on http://127.0.0.1:7878");
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, axum::extract::State(state)))
}
async fn handle_socket(mut socket: WebSocket, State(state): State<AppState>) {
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(ws::Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
/* async fn index() -> Html<String> {
    let html = tokio::fs::read_to_string("hello.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());

    Html(html)
} */
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

/* async fn submit_name(Form(form): Form<NameForm>) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", form.username))
} */
async fn show_user(Path(username): Path<String>, State(state): State<AppState>) -> Html<String> {
    let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());
    let safe_username = htmlescape::encode_minimal(&username);
    let game = state.game.lock().await;
    let rolle_text = match game.rolle_von(&username) {
        Some(rolle) => format!("{:?}", rolle),
        None => "Unbekannt".to_string(),
    };

    let rolle_html = htmlescape::encode_minimal(&rolle_text);
    let rolle = game.rolle_von(&username);

    let (rolle_text, action_html) = match rolle {
        Some(logic::Rolle::Werwolf) => (
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
        ),
        Some(logic::Rolle::Seher) => (
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
        ),
        _ => ("Dorfbewohner".to_string(), String::new()),
    };

    let user_page = template
        .replace("{{username}}", &safe_username)
        .replace("{{rolle}}", &rolle_html)
        .replace("{{aktion}}", &action_html);

    Html(user_page)
}
async fn add_user(State(state): State<AppState>, Form(form): Form<NameForm>) -> Redirect {
    let mut started = state.game_started.lock().await;
    if *started {
        return Redirect::to("/");
    }
    let mut game = state.game.lock().await;
    let username = &form.username;
    game.add_player(username.to_string());
    let _ = state.tx.send(format!("USER_ADDED:{}", username));
    /*  let mut users = state.usernames.lock().await;
    users.push(form.username.clone()); */

    Redirect::to("/")
}
async fn start_game(State(state): State<AppState>) -> Html<String> {
    let mut started = state.game_started.lock().await;
    *started = true;

    let mut users = state.game.lock().await;
    users.verteile_rollen();
    for p in users.players.iter() {
        let safe_username = encode(&p.name);
        let url = format!("http://127.0.0.1:7878/{}", safe_username);
        let _ = webbrowser::open(&url);
    }

    /* let users_list: String = users
       .iter()
       .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(u)))
       .collect::<Vec<_>>()
       .join("\n");
    */
    let _ = state.tx.send("GAME_STARTED".to_string());
    let template = tokio::fs::read_to_string("start_game.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());
    let users_html: String = users
        .players
        .iter()
        .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(&u.name)))
        .collect::<Vec<_>>()
        .join("\n");
    let phase = format!("{:?}", users.phase);
    let game_page = template
        .replace("{{users}}", &users_html)
        .replace("{{phase}}", &phase);
    Html(game_page)
    /* let html = format!(
        "<h1>Game Started!</h1><ul>{}</ul>",
        users_list
    );

    Html(html) */
}
async fn tag_action(State(state): State<AppState>, Form(form): Form<ActionForm>) -> Redirect {
    let mut game = state.game.lock().await;
    if let Phase::Tag = game.phase {
        game.tag_lynchen(&form.target);
        game.naechste_phase();
    }
    let _ = state
        .tx
        .send(format!("DAY_ACTION:{}:{}", form.actor, form.target));
    Redirect::to("/")
}

async fn werwolf_action(State(state): State<AppState>, Form(form): Form<ActionForm>) -> Redirect {
    println!("actor = '{}', target = '{}'", form.actor, form.target);
    let mut game = state.game.lock().await;

    game.werwolf_toetet(&form.target);

    let _ = state
        .tx
        .send(format!("WERWOLF_ACTION:{}:{}", form.actor, form.target));
    Redirect::to(&format!("/{}", form.actor))
}

async fn seher_action(State(state): State<AppState>, Form(form): Form<ActionForm>) -> Redirect {
    let game = state.game.lock().await;
    let rolle = game.seher_schaut(&form.target);
    println!(
        "Seher '{}' sieht, dass '{}' die Rolle {:?} hat",
        form.actor, form.target, rolle
    );
    let _ = state
        .tx
        .send(format!("SEHER_ACTION:{}:{}", form.actor, form.target));
    Redirect::to(&format!("/{}", form.actor))
}
