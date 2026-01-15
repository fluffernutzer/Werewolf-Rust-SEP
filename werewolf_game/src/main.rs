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

    let mut game = state.game.lock().await;

    let phase = game.phase.clone();                     
    let last_seher_result = game.last_seher_result.clone(); 

    let player_opt = game.players.iter_mut().find(|p| p.name == username);

    let (rolle_text, action_html) = if let Some(player) = player_opt {
        match player.rolle {
            crate::roles::Rolle::Werwolf => {
                if phase == crate::logic::Phase::WerwölfePhase && player.lebend && !player.bereits_gesehen {
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
                    ("Werwolf".to_string(), "<p> Aktion abgeschlossen. Bitte warte auf die nächste Phase.</p>".to_string())
                }
            }
            crate::roles::Rolle::Seher => {
                if phase == crate::logic::Phase::SeherPhase && player.lebend && !player.bereits_gesehen {
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
                    ("Seher".to_string(), "<p> Aktion abgeschlossen. Bitte warte auf die nächste Phase.</p>".to_string())
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
    game.add_player(form.username);
    /*  let mut users = state.usernames.lock().await;
    users.push(form.username.clone()); */

    Redirect::to("/")
}
async fn start_game(State(state): State<AppState>) -> Html<String> {
    let mut started = state.game_started.lock().await;
    *started = true;
    
    let mut users = state.game.lock().await;

    users.phase=Phase::Tag;
    users.runden=1;

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

) -> Html<String> {
    let mut game = state.game.lock().await;

    if game.phase != Phase::WerwölfePhase {
        println!("Es ist gerade keine Werwolf Phase");
        return Html(format!("<p>Es ist gerade keine Werwolf Phase</p>"));
    }

    if let Some(player) = game.players.iter_mut().find(|p| p.name == form.actor) {
        if !player.lebend || player.bereits_gesehen {
            println!("Werwolf darf diese Runde nicht mehr handeln.");
            return Html(format!("<p>Werwolf darf diese Runde nicht mehr handeln.</p>"));
        }
    }

    game.werwolf_toetet(&form.target);

    if let Some(player) = game.players.iter_mut().find(|p| p.name == form.actor) {
        player.bereits_gesehen = true; 
    }

    game.current_phase();
    //Redirect::to(&format!("/{}", form.actor))
    let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Fehler</h1>".to_string());

    let safe_username = htmlescape::encode_minimal(&form.actor);

    let action_html = format!(
        "<p>Du hast <strong>{}</strong> getötet.</p>",
        htmlescape::encode_minimal(&form.target)
    );

    let rolle_text = "Werwolf";
    println!("Phase NACH Aktion = {:?}", game.phase);
    let page = template
        .replace("{{username}}", &safe_username)
        .replace("{{rolle}}", rolle_text)
        .replace("{{aktion}}", &action_html);

    Html(page)
}

async fn tag_action(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Redirect {
    let mut game = state.game.lock().await;
    println!("Phase VOR Aktion = {:?}", game.phase);
    if let crate::logic::Phase::Tag = game.phase {
        game.tag_lynchen(&form.target);

        /* if let Some(winner) = tag_nacht::check_win(&game) {
            println!("SPIEL ENDE: {}", winner);
        } else {
            tag_nacht::advance_phase(&mut game);

    } */
    }
    game.naechste_phase();
    game.current_phase();
    println!("Phase NACH Aktion = {:?}", game.phase);
    Redirect::to("/")

}

async fn seher_action(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Redirect{
    let mut game = state.game.lock().await;
    println!("Phase vor Seher-Aktion = {:?}", game.phase);
    let rolle_opt = game.seher_schaut(&form.target);
    
    if let Some(p) =  rolle_opt {
        game.last_seher_result = Some((form.target.clone(), p));

    println!("Seher '{}' sieht, dass '{}' die Rolle {:?} hat", form.actor, form.target, p);
}
    game.naechste_phase();
    game.current_phase();
    println!("Phase NACH Aktion = {:?}", game.phase);
    Redirect::to(&format!("/{}", form.actor))

}

async fn hexe_heilt(
    State(state):State<AppState>,
    Form(form):Form<ActionForm>,
)->Redirect{
    let mut game=state.game.lock().await;
    game.hexe_heilt();
    game.naechste_phase();
    game.current_phase();
    Redirect::to(&format!("/{}", form.actor))

) -> Html<String>{
    let mut game = state.game.lock().await;

    if game.phase != Phase::SeherPhase {
        println!("Seher ist nicht dran.");
        return Html(format!("<p>Seher ist nicht dran.</p>"));
    }

    if let Some(player) = game.players.iter_mut().find(|p| p.name == form.actor) {
        if !player.lebend || player.bereits_gesehen {
            println!("Seher darf diese Runde nicht mehr handeln.");
            return Html(format!("<p>Seher ist diese Runde nicht mehr dran.</p>"));
        }
    }

    if let Some(rolle) = game.seher_schaut(&form.target) {
        game.last_seher_result = Some((form.target.clone(), rolle));
    }

    if let Some(player) = game.players.iter_mut().find(|p| p.name == form.actor) {
        player.bereits_gesehen = true; 
    }

    game.current_phase(); 
    //Redirect::to(&format!("/{}", form.actor))
    let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Fehler</h1>".to_string());

    let safe_username = htmlescape::encode_minimal(&form.actor);

    let action_html = if let Some(rolle) = game.seher_schaut(&form.target) {
    game.last_seher_result = Some((form.target.clone(), rolle));
    format!(
        "<p>Du hast gesehen, dass <strong>{}</strong> die Rolle {:?} hat.</p>",
        htmlescape::encode_minimal(&form.target),
        rolle
    )
} else {
    "<p>Die Aktion konnte nicht durchgeführt werden.</p>".to_string()
};
    let rolle_text = "Seher";

    let page = template
        .replace("{{username}}", &safe_username)
        .replace("{{rolle}}", rolle_text)
        .replace("{{aktion}}", &action_html);

    Html(page)

}

/*async fn jaeger_action(
    State(state):State<AppState>,
    Form(form):Form<ActionForm>,
)-> Redirect{
    let mut game=state.game.lock().await;
    game.jaeger_ziel=Some(form.target.clone());
    Redirect::to(&format!("/{}",form.actor))
}*/