#![forbid(unsafe_code)]
mod logic;
use axum::{
    Router,
    extract::{Form, Path, State},
    response::{Html, Json, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::{Arc};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use webbrowser;
use urlencoding::encode;

use crate::logic::Game;

#[derive(Deserialize)]
struct NameForm {
    username: String,
}
#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<Game>>,
    game_started: Arc<Mutex<bool>>,
}
#[tokio::main]

async fn main() {
    let state = AppState {
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/add-user", post(add_user))
        .route("/start-game", post(start_game))
        .route("/:username", get(show_user))
        .with_state(state);

    println!("Running on http://127.0.0.1:7878");
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    axum::serve(listener, app).await.unwrap();
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
    let users_html: String = game.players
        .iter()
        .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(&u.name)))
        .collect();

    let page = template.replace("{{users}}", &users_html);
    Html(page)
}

/* async fn submit_name(Form(form): Form<NameForm>) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", form.username))
} */
async fn show_user(Path(username): Path<String>) -> Html<String> {
    let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());
    let safe_username = htmlescape::encode_minimal(&username);
    let user_page = template.replace("{{username}}", &safe_username);
    Html(user_page)
}
async fn add_user(State(state): State<AppState>, Form(form): Form<NameForm>) -> Redirect {
    let mut started = state.game_started.lock().await;
    if *started {
        return Redirect::to("/");
    }
    let mut game =state.game.lock().await;
    game.add_player(form.username);
   /*  let mut users = state.usernames.lock().await;
    users.push(form.username.clone()); */

    Redirect::to("/")
}
async fn start_game(State(state): State<AppState>) -> Html<String> {
    let mut started = state.game_started.lock().await;
    *started = true;

    let users = state.game.lock().await;
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
    let template = tokio::fs::read_to_string("start_game.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());
    let users_html: String = users.players
        .iter()
        .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(&u.name)))
        .collect::<Vec<_>>()
        .join("\n");

    let game_page = template.replace("{{users}}", &users_html);
    Html(game_page)
/* let html = format!(
    "<h1>Game Started!</h1><ul>{}</ul>",
    users_list
);

Html(html) */

}
