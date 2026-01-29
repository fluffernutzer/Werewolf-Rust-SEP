#![forbid(unsafe_code)]
mod logic;
mod roles;
mod tag_nacht;
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
use local_ip_address::local_ip;
use qrcode::QrCode;
use image::Luma;
use uuid::Uuid;
use webbrowser;

use crate::logic::{Game, HexenAktion, Phase, Winner};

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
}
#[derive(Deserialize)]
struct ActionForm {
    actor: String,
    action_kind: String,
    target: String,
}
#[tokio::main]
async fn main() {
    let ip = local_ip().unwrap().to_string();
    generate_qr(&ip);
    let state = AppState {
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        server_ip: ip.clone(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/add-user", post(add_user))
        .route("/start-game", post(start_game))
        .route("/:username", get(show_user))
        .route("/tag", get(tag_show).post(tag_action))
        //.route("/winner", get(winner_show))
        .route("/nacht/werwolf", post(werwolf_action))
        .route("/nacht/seher", post(seher_action))
        .route("/nacht/hexe",post(hexe_action))
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
    //let image = code.render::<Luma<u8>>().build();

    //image.save("qr.png").unwrap();
    code.render::<qrcode::render::svg::Color>().min_dimensions(220,220).build()
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

    let opfer_text=match &game.nacht_opfer{
                    Some(name)=>format!("Das Opfer der Werwölfe ist {}. Was willst du tun?", name),
                    None=>"Die Werwölfe haben noch kein Opfer ausgewählt.".to_string(),
                };
                
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
                                <input type="hidden" name="action_kind" value="Toeten">
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
                            .to_string()
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
                                <input type="hidden" name="action_kind" value="Schauen">
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
                            .to_string()
                    )
                }
            }
            crate::roles::Rolle::Hexe=> {   
                if phase==crate::logic::Phase::HexePhase
                &&player.lebend
                //&&!hexe_done
                {
                    (
                        "Hexe".to_string(),
                        format!(
                            r#"
                            <h2>Hexen-Aktion</h2>
                            
                            <form action="/nacht/hexe" method="post">
                                <input type="hidden" name="actor" value="{username}">
                                <p>{opfer}</p>
                                <label>Was willst du tun:</label>
                                <select name="action_kind">
                                    <option value="Heilen">Heilen</option>
                                    <option value="Vergiften">Vergiften</option>
                                    <option value="NichtsTun">Nichts tun</option>
                                </select>

                            <label>zusätzliches Opfer (nur wenn du Vergiften wählst):</label>
                            <input name="target" placeholder="Spieler">

                            <button>Ausführen</button>
                            </form>
                            "#,
                            username=safe_username,
                            opfer=opfer_text
                        ),
                    )

                }else {
                    ("Hexe".to_string(),
                    "<p>Warte bist du dran bist.</p>".to_string(),
                )
                }}
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
    let token = uuid::Uuid::new_v4().to_string();
    let player = PlayerDevice {
        name: form.username.clone(),
        token: token.clone(),
    };
    state.play_dev.lock().await.push(player);
    let mut game = state.game.lock().await;
    game.add_player(form.username.clone());
    Redirect::to("/")
}
async fn start_game(State(state): State<AppState>) -> Html<String> {
    let mut started = state.game_started.lock().await;
    *started = true;
    let mut game_logic = state.game.lock().await;
    game_logic.verteile_rollen();
  
    //game_logic.current_phase();
    for p in game_logic.players.iter() {
        let url = format!("http://{}:7878/{}", state.server_ip, encode(&p.name));
        println!("Spieler {} geht zu {}", p.name, url);
    }
    /*for p in game_logic.players.iter() {
        let safe_username = encode(&p.name);
        let url = format!("http://127.0.0.1:7878/{}", safe_username);
        let _ = webbrowser::open(&url);
    }*/
    let template = tokio::fs::read_to_string("start_game.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());

    //let game = state.game.lock().await;
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
async fn werwolf_action(
    State(state): State<AppState>,
    Form(form): Form<ActionForm>,
) -> Html<String> {
    let mut game = state.game.lock().await;
    //Redirect::to(&format!("/{}", form.actor))
    let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Fehler</h1>".to_string());

    let safe_username = htmlescape::encode_minimal(&form.actor);
    let action_html = match game.werwolf_toetet(&form.actor,&form.target){
        Ok(())=>{
            game.nacht_opfer=Some(form.target.clone());
            format!("<p>Du hast <strong>{}</strong> getötet.</p>",
            htmlescape::encode_minimal(&form.target))
        },
        Err(msg)=>format!(
            "<p>Fehler: {}</p>",
            htmlescape::encode_minimal(&msg)
        ),

    };
   println!("Phase NACH Aktion: {:?}", game.phase);

    let rolle_text = "Werwolf";
    //println!("Phase NACH Aktion = {:?}", game.phase);
    let page = template
        .replace("{{username}}", &safe_username)
        .replace("{{rolle}}", rolle_text)
        .replace("{{aktion}}", &action_html);
    Html(page)
}
async fn tag_show(State(state): State<AppState>) -> Html<String> {
  
    let mut game = state.game.lock().await;
    let winner_opt = game.check_win();

    let template = tokio::fs::read_to_string("tag-action.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());

    let users_html: String = game
        .players
        .iter()
        .filter(|p| p.lebend)
        .map(|u| format!("<li>{}</li>", htmlescape::encode_minimal(&u.name)))
        .collect::<Vec<_>>()
        .join("\n");

    let phase = format!("{:?}", game.phase);

    let action_html = if let Some(winner) = winner_opt {
        tokio::spawn(async move {
        println!("Spiel vorbei! Server wird beendet.");
        std::process::exit(0);
    });
        let winner_text = match winner {
            Winner::Dorf => "Dorf",
            Winner::Werwolf => "Werwolf",
        };
        format!(
            "<p>Glückwunsch Team <strong>{}</strong>! Ihr habt gewonnen.</p>",
            winner_text
        )
    } else if game.phase == Phase::Tag {
        r#"<h2>Tagaktion</h2>
        <form action="/tag" method="post">
            <input type="text" name="username" placeholder="Spielername">
            <button type="submit">Lynchen</button>
        </form>"#.to_string()
    } else if let Some(opfer) = &game.tag_opfer {
        format!(
            "<p>Ihr habt <strong>{}</strong> gelyncht.</p>",
            htmlescape::encode_minimal(opfer)
        )
    } else {
        "<p>Es ist gerade Nacht.</p>".to_string()
    };

    let page = template
        .replace("{{phase}}", &phase)
        .replace("{{users}}", &users_html)
        .replace("{{aktion}}", &action_html);

    Html(page)
}
async fn tag_action(State(state): State<AppState>, Form(form): Form<NameForm>) -> Redirect {
    let mut game = state.game.lock().await;
    if game.phase == Phase::Tag {
        game.tag_lynchen(&form.username);
   
        println!("Phase NACH Tag = {:?}", game.phase);
    }


    Redirect::to("/tag")
}


async fn seher_action( State(state): State<AppState>, Form(form): Form<ActionForm>)-> Html<String>{
    let mut game = state.game.lock().await;

    //Redirect::to(&format!("/{}", form.actor))

   let template = tokio::fs::read_to_string("user.html")
        .await
        .unwrap_or("<h1>Fehler</h1>".to_string());

    let safe_username = htmlescape::encode_minimal(&form.actor);
    let rolle_text = "Seher";

    let action_html = match game.seher_schaut(&form.target){
        Ok(rolle)=>format!(
                "<p>Du hast gesehen, dass <strong>{}</strong> die Rolle {:?} hat.</p>",
                htmlescape::encode_minimal(&form.target),
                rolle
        ),
        
        Err(msg)=>format!(
                "<p>Fehler: {}</p>",
                htmlescape::encode_minimal(&msg)
            ),
        };

    println!("Phase NACH Aktion: {:?}", game.phase);
    let page = template
        .replace("{{username}}", &safe_username)
        .replace("{{rolle}}", rolle_text)
        .replace("{{aktion}}", &action_html);

    Html(page)
}
async fn winner_show(winner: Winner) -> Html<String> {
    let template = tokio::fs::read_to_string("winner.html")
        .await
        .unwrap_or("<h1>Could not read file</h1>".to_string());

      let winner_text = format!("{:?}", winner);
    let page = template.replace("{{winner}}", &winner_text);
    Html(page)
}

async fn hexe_action(
    State(state):State<AppState>,
    Form(form):Form<ActionForm>,
)->Html<String>{
    let mut game=state.game.lock().await;
    
    let template=tokio::fs::read_to_string("user.html")
    .await.unwrap_or("<h1>Fehler</h1>".to_string());
    let actor_name=form.actor.clone();
    let extra_target = form.target.clone();
    let aktion=match form.action_kind.as_str(){
        "Heilen"=>HexenAktion::Heilen,
        "Vergiften"=>HexenAktion::Vergiften,
        _=>HexenAktion::NichtsTun,
        
    };
    let action_html=match game.hexe_arbeitet(aktion, &actor_name, &extra_target){
        Ok(())=>"<p>Aktion erfolgreich</p>".to_string(),
        Err(msg)=>format!("<p>Fehler: {}</p>", htmlescape::encode_minimal(&msg)),
    };
    println!("Phase NACH Aktion: {:?}", game.phase);
    let safe_username=htmlescape::encode_minimal(&form.actor);
    let rolle_text="Hexe";
    let page=template
    .replace("{{username}}",&safe_username)
    .replace("{{rolle}}", rolle_text)
    .replace("{{aktion}}", &action_html);
    Html(page)
}
