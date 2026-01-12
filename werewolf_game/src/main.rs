#![forbid(unsafe_code)]
pub mod roles;
mod logic;
mod tag_nacht;
// use crate::tag_nacht::{check_win, advance_phase};

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
    sync::Mutex,
};
use urlencoding::encode;
use webbrowser;

use crate::logic::{Game,Phase};

#[derive(Deserialize)]
struct NameForm {
    username: String,
}

#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<Game>>,
    game_started: Arc<Mutex<bool>>,
}
#[derive(Deserialize)]
struct ActionForm {
    actor: String,
    target: String,
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
        .route("/nacht/werwolf", post(werwolf_action))
        .route("/tag", post(tag_action))
        .route("/nacht/seher", post(seher_action))
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
    Some(roles::Rolle::Werwolf) => (
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
    Some(roles::Rolle::Seher) => (
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
        .replace("{{rolle}}", &rolle_html).replace("{{aktion}}", &action_html);

    Html(user_page)
}

async fn add_user(State(state): State<AppState>, Form(form): Form<NameForm>) -> Redirect {
    let mut started = state.game_started.lock().await;
    if *started {
        return Redirect::to("/");
    }
    let mut game = state.game.lock().await;
    game.add_player(form.username);
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
    let game_page = template.replace("{{users}}", &users_html).replace("{{phase}}", &phase);
    Html(game_page)
    /* let html = format!(
        "<h1>Game Started!</h1><ul>{}</ul>",
        users_list
    );

    Html(html) */
}

async fn werwolf_action(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Redirect {
    let mut game = state.game.lock().await;

    game.werwolf_toetet(&form.target);

    /*if let Some(winner) = tag_nacht::check_win(&game){
        println!("SPIEL ENDE: {}", winner);
    } else {
        tag_nacht::advance_phase(&mut game);
    } */

    Redirect::to(&format!("/{}", form.actor))
}

async fn tag_action(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Redirect {
    let mut game = state.game.lock().await;

    if let crate::logic::Phase::Tag = game.phase {
        game.tag_lynchen(&form.target);

        /* if let Some(winner) = tag_nacht::check_win(&game) {
            println!("SPIEL ENDE: {}", winner);
        } else {
            tag_nacht::advance_phase(&mut game);

    } */
}

    Redirect::to("/")
}

async fn seher_action(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Redirect{
    let mut game = state.game.lock().await;
    let rolle_opt = game.seher_schaut(&form.target);
    
    if let Some(p) =  rolle_opt {
        game.last_seher_result = Some((form.target.clone(), p));

    println!("Seher '{}' sieht, dass '{}' die Rolle {:?} hat", form.actor, form.target, p);
}

    Redirect::to(&format!("/{}", form.actor))
}

/*async fn jaeger_action(
    State(state):State<AppState>,
    Form(form):Form<ActionForm>,
)-> Redirect{
    let mut game=state.game.lock().await;
    game.jaeger_ziel=Some(form.target.clone());
    Redirect::to(&format!("/{}",form.actor))
}*/