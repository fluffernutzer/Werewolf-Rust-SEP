use askama::Template;
use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{Html, Response},
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json};
//use std::{os::macos::raw::stat, sync::Arc};
use base64::{Engine as _, engine::general_purpose};
use qrcode::QrCode;
use std::{fs, os::macos::raw::stat, str::FromStr, sync::Arc};
use tokio::sync::{Mutex, broadcast, mpsc};
use urlencoding::encode;
use webbrowser;

use crate::{
    AppState, PlayerDevice, generate_qr,
    logic::{Game, HexenAktion, Phase, Spieler, Winner},
    roles::Rolle,
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    players: Vec<PlayerTemplate<'a>>,
    phase: String,
    qr_code: String,
}
/*#[derive(Template)]
#[template(path = "winner.html")]
pub struct WinnerTemplate {
    winner: String,
}*/
#[derive(Template)]
#[template(path = "user.html")]
struct UserTemplate<'a> {
    username: &'a str,
    rolle: &'a str,
    players: Vec<PlayerTemplate<'a>>,
    phase: String,
}
#[derive(Template)]
#[template(path = "join.html")]
struct JoinTemplate {}

#[derive(Serialize)]
struct PlayerTemplate<'a> {
    name: &'a str,
    rolle: &'a str,
    status: bool,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
#[serde(tag = "type", content = "data")]

pub enum ClientMessage{
    StartGame,
    ResetGame,
    IngameBereit{
        username: String,
        ready: bool,
    },
    AddUser {
        username: String,
    },
    ReadyStatus {
        username: String,
        ready: bool,
    },
    TagAction {
        direction: ActionForm,
    },
    WerwolfAction {
        direction: ActionForm,
    },
    SeherAction {
        actor: String,
        target: String,
    },
    HexenAction{direction: ActionForm, hexenAktion:HexenAktion, extra_target: String},
    AmorAction {actor: String, target1: String, target2: String },
    DoktorAction { direction: ActionForm },
    PriesterAction { actor: String, target: Option<String> },
    JaegerAction { actor: String, target: Option<String> },
    ChatMessage {
        sender: String,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ActionForm {
    pub actor: String,
    pub target: Option<String>,
}

pub enum ActionKind {
    DorfLyncht,
    WerwolfFrisst,
    //SeherSieht,
    //HexeHext,
}
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<String>();

    let mut rx = state.tx.subscribe();

    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {

                Ok(msg) = rx.recv() => {
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }

                Some(msg) = client_rx.recv() => {
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            }
        }

    });

    let recv_state = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            //println!("Client Message: {}", text); 

            let Ok(client_message) = serde_json::from_str::<ClientMessage>(&text) else {
                eprintln!("Ungültige Nachricht");
                continue;
            };

            if let Err(e) = handle_message(&recv_state, client_message, &client_tx).await {
                eprintln!("Fehler: {e}");
            }
            let game = recv_state.game.lock().await;

            

            drop(game);

           
            send_game_state(&recv_state).await;
        }

        
    });

    let _ = tokio::join!(send_task, recv_task);
}
pub async fn handle_message( state: &AppState,client_message: ClientMessage,client_tx: &mpsc::UnboundedSender<String>) -> Result<(), String> {
    let mut game = state.game.lock().await;
    let recv_state = state.clone();
    
    match client_message {

                ClientMessage::StartGame => {
                    if !*recv_state.game_started.lock().await {
                        *recv_state.game_started.lock().await = true;

                        game.phase = Phase::Spielbeginn;
                        game.runden = 1;
                        let _ = game.verteile_rollen();

                        let _ = recv_state.tx.send(serde_json::json!({
                            "type": "GAME_STARTED"
                        }).to_string());
                    }
                }
                ClientMessage::ResetGame => {
                            println!("Starte zrücksetzen");
                                //let mut game = state.game.lock().await;
                                *game = Game::new();
                                let mut game_started = state.game_started.lock().await;
                                *game_started = false;
                                let mut play_dev = state.play_dev.lock().await;
                                play_dev.clear();
                                let _ = recv_state.tx.send(serde_json::json!({
                                    "type": "GAME_RESET"
                                }).to_string());
                                println!("Zurüclsetzen beendet");

                }
                ClientMessage::ReadyStatus { username, ready } => {
                    if let Some(player) = game.players.iter_mut().find(|p| p.name == username) {
                        player.ready_state = ready;
                    }

                    if game.players.iter().all(|p| p.ready_state) {
                        *recv_state.game_started.lock().await = true;

                        game.phase = Phase::Spielbeginn;
                        game.runden = 1;
                        let _ = game.verteile_rollen();

                        let _ = recv_state.tx.send(serde_json::json!({
                            "type": "GAME_STARTED"
                        }).to_string());
                    }
                }
                ClientMessage::IngameBereit{username, ready} => {
                    if let Some(player) = game.players.iter_mut().find(|p| p.name == username) {
                        player.ingame_ready_state = ready;
                    }

                    if game.players.iter().all(|p| p.ingame_ready_state) {
                        game.phase_change();

                    }
                }
                ClientMessage::AddUser { username } => {
                    if *state.game_started.lock().await {
                        log::error!("Aktuell können keine Spieler mehr der Runde beitreten");
                        //return //Err("Aktuell können keine Spieler mehr der Runde beitreten");
                    } 
                    if game.players.iter().any(|p| p.name == username) {
                        return Err("Name existiert bereits".to_string());
                    }
                    if *state.game_started.lock().await {
                        log::error!("Aktuell können keine Spieler mehr der Runde beitreten");
                        //return //Err("Aktuell können keine Spieler mehr der Runde beitreten");
                    } 
                    let token = uuid::Uuid::new_v4().to_string();

                    recv_state.play_dev.lock().await.push(PlayerDevice {
                        name: username.clone(),
                        token: token.clone(),
                    });
                    

                    game.add_player(username);
                    
                    let _ = client_tx.send(serde_json::json!({
                        "type": "JOINED",
                        "token": token
                    }).to_string());
                }

                ClientMessage::TagAction { direction } => {
                    if let Phase::Tag = game.phase {
                         match direction.target {
                            Some(target) => {
                                let _ = handle_vote(&mut game, direction.actor, target, ActionKind::DorfLyncht).await;
                            },
                            None => log::error!("Werwolf Ziel fehlt"),
                        } 
                    }
                }

                ClientMessage::WerwolfAction { direction } => {
                    if let Phase::WerwölfePhase = game.phase {
                        match direction.target {
                            Some(target) => {
                                let _ = handle_vote(& mut game, direction.actor, target, ActionKind::WerwolfFrisst).await;
                            },
                            None => log::error!("Werwolf Ziel fehlt"),
                        }
                    } else {log::info!("Werwölfe gerade nicht dran!");
                            //return
                        }
                }

                ClientMessage::SeherAction { actor,target} => {
                    if let Phase::SeherPhase = game.phase {
                        if let Ok(rolle) = game.seher_schaut(&target) {
                            game.last_seher_result =
                                Some((target.clone(), rolle));
                        }
                    } else {
                        log::info!("Seher gerade nicht dran");
                        //return;
                    }
                }
                ClientMessage::HexenAction {direction, hexenAktion , extra_target } => {
                            let _ = game.hexe_arbeitet(hexenAktion, &direction.actor ,extra_target);
                        }
                ClientMessage::AmorAction { actor, target1, target2 } =>{
                        let _ = game.amor_waehlt(target1, target2);
                    }        
                ClientMessage::DoktorAction { direction } => {
                    match direction.target {
                            Some(target) => {
                                let _ = game.doktor_schuetzt(&target);
                            },
                            None => log::info!("Doktor tut nichts"),
                        } 
                        }
                ClientMessage::PriesterAction { actor, target} => {
                            let _ = game.priester_wirft(&actor, target);
                        }
                ClientMessage::JaegerAction {actor,target} => {
                    game.jaeger_ziel = target;
                }
                     

                ClientMessage::ChatMessage { sender, message } => {
                    let _ = recv_state.tx.send(serde_json::json!({
                        "type": "CHAT_MESSAGE",
                        "data": { "sender": sender, "message": message }
                    }).to_string());
                }
                
            }
            Ok(())
        }
        //Ok(())

pub async fn send_game_state(state: &AppState) {
    let game = state.game.lock().await;
    let game_started = state.game_started.lock().await;
    let win = game.check_win();
    let message = json!({
        "type": "GAME_STATE",
        "state": {
            "players": game.players.iter()
                .map(|p| json!({
                    "name": p.name,
                    "rolle": match game.rolle_von(&p.name) {
                        Some(r) => format!("{:?}", r),
                        None => "?".to_string()
                    },
                    "status": p.lebend,
                    "ready":p.ready_state
                }))
                .collect::<Vec<_>>(),
            "phase": format!("{:?}", game.phase),
            "last_seher_result": game.last_seher_result
                .as_ref()
                .map(|(t, r)| format!("{} is {:?}", t, r)),
            "eligible_players": game.eligible_players,
            "current_votes": game.current_votes,
            "ongoing_vote": game.ongoing_vote,
        }
    });

    let message_str =
        serde_json::to_string(&message).expect("Fehler beim Serialisieren des GameState");
    //println!("Sende GameState: {}", message_str); // Debug-Ausgabe
    let _ = state.tx.send(message_str);

    if let Some(winner) = win
        && *game_started
    {
        let winner_message = json!({
            "type": "WINNER",
            "winner": format!("{:?}", winner)
        });
        let winner_message_str = serde_json::to_string(&winner_message)
            .expect("Fehler beim Serialisieren der Winner-Nachricht");
        println!("Sende Winner-Nachricht: {}", winner_message_str);
        let _ = state.tx.send(winner_message_str);
    }
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let game = state.game.lock().await;
    let qr_svg = generate_qr(&state.server_ip);
    let qr_code_base64 = general_purpose::STANDARD.encode(qr_svg.as_bytes());

    let players: Vec<PlayerTemplate> = game
        .players
        .iter()
        .map(|p| PlayerTemplate {
            name: &p.name,
            rolle: match game.rolle_von(&p.name) {
                Some(rolle) => match rolle {
                    Rolle::Werwolf => "Werwolf",
                    Rolle::Seher => "Seher",
                    _ => "Dorfbewohner",
                },
                None => "?",
            },
            status: p.lebend,
        })
        .collect();

    let template = IndexTemplate {
        players,
        phase: format!("{:?}", game.phase),
        qr_code: format!("data:image/svg+xml;base64,{}", qr_code_base64),
    };

    Html(template.render().unwrap())
}

pub async fn show_user(Path(username): Path<String>,State(state): State<AppState>,) -> Html<String> {
    let game = state.game.lock().await;

    let rolle = match game.rolle_von(&username) {
        Some(rolle) => match rolle {
            Rolle::Werwolf => "Werwolf",
            Rolle::Seher => "Seher",
            Rolle::Hexe => "Hexe",
            Rolle::Amor => "Amor",
            Rolle::Jäger => "Jäger",
            Rolle::Doktor => "Doktor",
            Rolle::Priester => "Priester",
            _ => "Dorfbewohner",
        },
        None => "?",
    };

    let players: Vec<PlayerTemplate> = game
        .players
        .iter()
        .map(|p| PlayerTemplate {
            name: &p.name,
            rolle: match game.rolle_von(&p.name) {
                Some(rolle) => match rolle {
                    Rolle::Werwolf => "Werwolf",
                    Rolle::Seher => "Seher",
                    Rolle::Hexe => "Hexe",
                    Rolle::Amor => "Amor",
                    Rolle::Jäger => "Jäger",
                    Rolle::Doktor => "Doktor",
                    Rolle::Priester => "Priester",
                    _ => "Dorfbewohner",
                },
                None => "?",
            },
            status: p.lebend,
        })
        .collect();

    let template = UserTemplate {
        username: &username,
        rolle: &rolle,
        players,
        phase: format!("{:?}", game.phase),
    };

    Html(template.render().unwrap())
}


async fn handle_vote(
    game: &mut Game,
    actor: String,
    target: String,
    action: ActionKind,
) -> Result<(), String> {
    if let Some (_player)= game.players.iter_mut().find(|p| p.name == target){
        if let Some(_player) = game.players.iter_mut().find(|p| p.name == actor && p.has_voted) {
                return Err("Du hast schon abgestimmt".to_string())};
        if let Some(player) = game.players.iter_mut().find(|p| p.name == actor && p.lebend){
            player.has_voted = true;
        } 
        else{
        return Err("Spieler nicht gefunden oder nicht lebendig".to_string());
        }
    }
    else {
        return Err("Spieler nicht gefunden oder nicht lebendig".to_string());
    }

    game.votes.entry(target.to_string()).or_default().push(actor.to_string());
    game.ongoing_vote = true;

    game.eligible_players = game
        .players
        .iter()
        .filter(|p| {
            p.lebend && match game.phase {
                Phase::Spielbeginn => false,
                Phase::Tag => true,
                Phase::WerwölfePhase => p.rolle == Rolle::Werwolf,
                Phase::SeherPhase => p.rolle == Rolle::Seher,
                Phase::HexePhase => p.rolle == Rolle::Hexe,
                Phase::AmorPhase => p.rolle == Rolle::Amor,
                Phase::PriesterPhase => p.rolle == Rolle::Priester,
                Phase::DoktorPhase => p.rolle == Rolle::Doktor,
            }
        })
        .map(|p| p.name.clone())
        .collect();

    game.current_votes = game.votes.clone();

    let all_voted = game.eligible_players.iter().all(|name| {
        game.players
            .iter()
            .find(|p| p.name == *name)
            .map_or(false, |p| p.has_voted)
    });

    if all_voted {
        let max_votes = game.votes.values().map(|v| v.len()).max().unwrap_or(0);
        let candidates: Vec<String> = game
            .votes
            .iter()
            .filter(|(_, voters)| voters.len() == max_votes)
            .map(|(target, _)| target.clone())
            .collect();

        if candidates.len() > 1 {
            game.votes.clear();
            for player in game.players.iter_mut() {
                if game.eligible_players.contains(&player.name) {
                    player.has_voted = false;
                }
            }
            game.current_votes = candidates
                .into_iter()
                .map(|c| (c, Vec::new()))
                .collect();
            return Ok(());
        }

        if let Some(final_target) = candidates.into_iter().next() {
            match action {
                ActionKind::WerwolfFrisst => match game.werwolf_toetet(&actor, &final_target){
                    Ok(()) => (),
                    Err(msg) => {log::info!("{}",msg)}
                },
                ActionKind::DorfLyncht => game.tag_lynchen(&final_target),
                //ActionKind::HexeHext => (),
                //ActionKind::SeherSieht => (),
            }
            game.votes.clear();
            for player in game.players.iter_mut() {
                if game.eligible_players.contains(&player.name) {
                    player.has_voted = false;
                }
            }
            game.current_votes.clear();
            game.ongoing_vote = false;
        }
    }

    Ok(())
}
/*#[derive(Deserialize)]
pub struct WinnerParams {
    winner: String,
}

pub async fn winner_page(Query(params): Query<WinnerParams>) -> Html<String> {
    let template = WinnerTemplate { winner: params.winner };
    Html(template.render().unwrap())
}*/
pub async fn join_page() -> Html<String> {
    let template = JoinTemplate {};
    Html(template.render().unwrap())
}
pub async fn play_page(Path(token): Path<String>, State(state): State<AppState>,) -> Html<String> {
    let play_dev = state.play_dev.lock().await;
   
    // Spieler anhand des Tokens finden

    if let Some(player) = play_dev.iter().find(|p| p.token == token) {
        let game = state.game.lock().await;

        // Rolle des Spielers
        let rolle = match game.rolle_von(&player.name) {
            Some(r) => match r {
                Rolle::Werwolf => "Werwolf",
                Rolle::Seher => "Seher",
                Rolle::Hexe => "Hexe",
                Rolle::Amor => "Amor",
                Rolle::Jäger => "Jäger",
                Rolle::Priester => "Priester", // Hoffe OK? 
                Rolle::Doktor => "Doktor", // Hoffe OK? 
                _ => "Dorfbewohner",
            },
            None => "?",
        };

        // Spieler-Liste für die Anzeige
        let players: Vec<PlayerTemplate> = game.players
            .iter()
            .map(|p| PlayerTemplate {
                name: &p.name,
                rolle: match game.rolle_von(&p.name) {
                    Some(r) => match r {
                        Rolle::Werwolf => "Werwolf",
                        Rolle::Seher => "Seher",
                        Rolle::Hexe => "Hexe",
                        Rolle::Amor => "Amor",
                        Rolle::Jäger => "Jäger",
                        Rolle::Priester => "Priester", // Hoffe OK? 
                        Rolle::Doktor => "Doktor", // Hoffe OK? 
                        _ => "Dorfbewohner",
                    },
                    None => "?",
                },
                status: p.lebend,
            })
            .collect();

        let template = UserTemplate {
            username: &player.name,
            rolle,
            players,
            phase: format!("{:?}", game.phase),
        };

        Html(template.render().unwrap())
    } else {
        Html("Token ungültig oder Spieler nicht gefunden!".to_string())
    }
}












//Testing: 

fn load_game(num: usize) -> Vec<ClientMessage> {
    let path = format!("tests/game{num}.txt");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Could not read file: {}", &path));
    let lines: Vec<&str> = content.lines().collect();
    lines
        .iter()
        .map(|a| a.parse().expect("parse failed"))
        .collect()
}
fn load_players(num: usize) -> Vec<Spieler> {
    let path = format!("tests/players{num}.txt");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Could not read file: {}", &path));
    let lines: Vec<&str> = content.lines().collect();
    lines
        .iter()
        .map(|a| a.parse().expect("parse failed"))
        .collect()
}

impl FromStr for ClientMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts.as_slice() {
            ["StartGame"] => Ok(ClientMessage::StartGame),
            ["ResetGame"] => Ok(ClientMessage::ResetGame),
            ["AddUser", username] => Ok(ClientMessage::AddUser { username: username.to_string() }),
            ["IngameBereit", username] => Ok(ClientMessage::IngameBereit { username: username.to_string(), ready: true }),
            ["ReadyStatus", username, _, ready] => {
                let ready = ready.parse::<bool>().map_err(|_| "Invalid bool for ready status")?;
                Ok(ClientMessage::ReadyStatus { username: username.to_string(), ready })
            }
            ["TagAction", actor, target] => Ok(ClientMessage::TagAction {
                direction: ActionForm { actor: actor.to_string(), target: Some(target.to_string()) },
            }),
            ["Werwolf", actor, target] => Ok(ClientMessage::WerwolfAction {
                direction: ActionForm { actor: actor.to_string(), target: Some(target.to_string()) },
            }),
            ["Seher", actor, target] => Ok(ClientMessage::SeherAction {
                 actor: actor.to_string(), target: target.to_string() ,
            }),
            ["Hexe",hexenaktion, actor, extra_target] => {
            let hexenaktion = match *hexenaktion {
                "heilt" => HexenAktion::Heilen,
                "vergiftet" => HexenAktion::Vergiften,
                "tut_nichts" => HexenAktion::NichtsTun,
                _ => return Err("ungültige Hexenaktion".to_string()),
            };
            Ok(ClientMessage::HexenAction {
                direction: ActionForm { actor: actor.to_string(), target: Some("None".to_string()) },
                hexenAktion: hexenaktion,extra_target: extra_target.to_string(),
            })},
            ["Amor", actor, target1, target2] => Ok(ClientMessage::AmorAction {
                actor: actor.to_string(),
                target1: target1.to_string(),
                target2: target2.to_string(),
            }),
            ["Doktor", actor, target] => Ok(ClientMessage::DoktorAction {
                direction: ActionForm { actor: actor.to_string(), target: Some(target.to_string()) },
            }),
            ["Priester", actor, target @ ..] => {
                let target = if target.is_empty() {None} 
                    else {Some(target[0].to_string())};
                Ok(ClientMessage::PriesterAction {
                    actor: actor.to_string(),
                    target,
                })},
            ["Chat", sender, message @ ..] => {
                let message = message.join(" ");
                Ok(ClientMessage::ChatMessage { sender: sender.to_string(), message })
            }
            _ => Err(format!("Invalid action format: '{}'", s)),
        }
    }
}


async fn test_Client_message_ACTION_handling(
    mut game: &mut Game,
    state: &AppState,
    client_message: ClientMessage,
){
    // Der MatchBlock muss stets genau dem der ws_handler-funktion(Aktionen) entsprechen! Ausnhame hexe: benötigt an sich kein Target in direction da nur Extra-target genutzt wird
    match client_message {
                ClientMessage::ResetGame => {
                            println!("Starte zrücksetzen");
                                //let mut game = state.game.lock().await;
                                *game = Game::new();
                                let mut game_started = state.game_started.lock().await;
                                *game_started = false;
                                let mut play_dev = state.play_dev.lock().await;
                                play_dev.clear();
                                println!("Zurüclsetzen beendet");

                        }
                ClientMessage::IngameBereit{username, ready} => {
                    if let Some(player) = game.players.iter_mut().find(|p| p.name == username) {
                        player.ingame_ready_state = ready;
                    }

                    if game.players.iter().all(|p| p.ingame_ready_state) {
                        game.phase_change();
                        println!("iname bereits phasenwechsel");

                    }
                }
                ClientMessage::TagAction { direction } => {
                    if let Phase::Tag = game.phase {
                         match direction.target {
                            Some(target) => {
                                let _ = handle_vote(&mut game, direction.actor, target, ActionKind::DorfLyncht).await;
                            },
                            None => log::error!("Werwolf Ziel fehlt"),
                        } 
                    }
                }

                ClientMessage::WerwolfAction { direction } => {
                    if let Phase::WerwölfePhase = game.phase {
                        match direction.target {
                            Some(target) => {
                                let _ = handle_vote(& mut game, direction.actor, target, ActionKind::WerwolfFrisst).await;
                            },
                            None => log::error!("Werwolf Ziel fehlt"),
                        }
                    } else {log::info!("Werwölfe gerade nicht dran!");
                            //return
                        }
                }

                ClientMessage::SeherAction { actor,target} => {
                    if let Phase::SeherPhase = game.phase {
                        if let Ok(rolle) = game.seher_schaut(&target) {
                            game.last_seher_result =
                                Some((target.clone(), rolle));
                        }
                    } else {
                        log::info!("Seher gerade nicht dran");
                        return;
                    }
                }
                ClientMessage::HexenAction {direction, hexenAktion , extra_target } => {
                            let _ = game.hexe_arbeitet(hexenAktion, &direction.actor ,extra_target);
                        }
                ClientMessage::AmorAction { actor, target1, target2 } =>{
                    game.amor_waehlt(target1, target2);
                }
                ClientMessage::DoktorAction { direction } => {
                    match direction.target {
                            Some(target) => {
                                let _ = game.doktor_schuetzt(&target);
                            },
                            None => log::info!("Doktor tut nichts"),
                        } 
                        }
                ClientMessage::PriesterAction { actor, target} => {
                            let _ = game.priester_wirft(&actor, target);
                        }
                ClientMessage::JaegerAction {actor,target} => {
                    game.jaeger_ziel = target;
                }
                        _ => {}
            }

}






#[tokio::test]


    async fn test_game_one_werwolf_winner  () {
        let state = AppState {
        tx: broadcast::channel(10).0,
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        server_ip: "Test-IP".to_string(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
        let spieler = load_players(10);
        let spielablauf = load_game(10);
        let mut game = state.game.lock().await;
        game.runden = 2;
        game.players = spieler; 
        for clientmessage in spielablauf {
            let clientprint = clientmessage.clone();
            let phase_print = game.phase.clone();
            test_Client_message_ACTION_handling(&mut game, &state, clientmessage).await;
            println!("Clientmessage:{:?}",clientprint);
            println!("Phase:{:?}",phase_print);
        }
        let remaining:Vec<String> = game.players.iter().filter(|p| {p.lebend}).map(|p| p.name.clone()).collect();
        println!("Remaining:{:?}",remaining);
        let winner = game.check_win();
        assert_eq!(Some(Winner::Werwolf),winner)
    }
#[tokio::test]
        async fn test_game_two_village_winner  () {
        let state = AppState {
        tx: broadcast::channel(10).0,
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        server_ip: "Test-IP".to_string(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
        let spieler = load_players(10);
        let spielablauf = load_game(20);
        let mut game = state.game.lock().await;
        game.players = spieler; 
        for clientmessage in spielablauf {
            let clientprint = clientmessage.clone();
            let phase_print = game.phase.clone();
            test_Client_message_ACTION_handling(&mut game, &state, clientmessage).await;
            println!("Clientmessage:{:?}",clientprint);
            println!("Phase:{:?}",phase_print);
        }
        let remaining:Vec<String> = game.players.iter().filter(|p| {p.lebend}).map(|p| p.name.clone()).collect();
        println!("Remaining:{:?}",remaining);
        let winner = game.check_win();
        assert_eq!(Some(Winner::Dorf),winner)
    }

    #[tokio::test]
        async fn test_game_two_wrong_timed_action  () {
        let state = AppState {
        tx: broadcast::channel(10).0,
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        server_ip: "Test-IP".to_string(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
        let spieler = load_players(10);
        let spielablauf = load_game(21);
        let mut game = state.game.lock().await;
        game.players = spieler; 
        for clientmessage in spielablauf {
            let clientprint = clientmessage.clone();
            let phase_print = game.phase.clone();
            test_Client_message_ACTION_handling(&mut game, &state, clientmessage).await;
            println!("Clientmessage:{:?}",clientprint);
            println!("Phase:{:?}",phase_print);
        }
        let remaining:Vec<String> = game.players.iter().filter(|p| {p.lebend}).map(|p| p.name.clone()).collect();
        println!("Remaining:{:?}",remaining);
        let winner = game.check_win();
        assert_eq!(Some(Winner::Dorf),winner)
    }

     #[tokio::test]
        async fn test_game_reset_game  () { //Same game as test_game_two_village_winner but last action is reset-call -> winner gets deleted
        let state = AppState {
        tx: broadcast::channel(10).0,
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        server_ip: "Test-IP".to_string(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
        let spieler = load_players(10);
        let spielablauf = load_game(211);
        let mut game = state.game.lock().await;
        game.players = spieler; 
        for clientmessage in spielablauf {
            let clientprint = clientmessage.clone();
            test_Client_message_ACTION_handling(&mut game, &state, clientmessage).await;
            println!("Clientmessage:{:?}",clientprint);
        }
        let remaining:Vec<String> = game.players.iter().filter(|p| {p.lebend}).map(|p| p.name.clone()).collect();
        println!("Remaining:{:?}",remaining);
        let winner = game.check_win();
        assert_eq!(None,winner)
    }

     #[tokio::test]
        async fn critical_votes_game3  () { //Testing for handling votes of same amount. e.g 5 for A and 5 for B
        let state = AppState {
        tx: broadcast::channel(10).0,
        game: Arc::new(Mutex::new(Game::new())),
        game_started: Arc::new(Mutex::new(false)),
        server_ip: "Test-IP".to_string(),
        play_dev: Arc::new(Mutex::new(Vec::new())),
    };
        let spieler = load_players(10);
        let spielablauf = load_game(3);
        let mut game = state.game.lock().await;
        game.players = spieler; 
        for clientmessage in spielablauf {
            let clientprint = clientmessage.clone();
            println!("Clientmessage:{:?}",clientprint);
            test_Client_message_ACTION_handling(&mut *game, &state, clientmessage).await;
            println!("Phase:{:?}",game.phase);
        }
        let remaining:Vec<String> = game.players.iter().filter(|p| {p.lebend}).map(|p| p.name.clone()).collect();
        assert_eq!(["W1", "S", "H", "W2"],*remaining);
        assert_eq!(Some(Winner::Werwolf),game.check_win());
        assert_eq!(2,game.runden);
    }

#[cfg(test)]
mod tests{
    use super::*;
    use tokio::sync::{Mutex, broadcast};
    use std::sync::Arc;
    #[tokio::test]
    async fn add_user_msg() {
        let (tx, _rx) = broadcast::channel(32);
        let state = AppState {
            game: Arc::new(Mutex::new(Game::new())),
            game_started: Arc::new(Mutex::new(false)),
            server_ip: "127.0.0.1".to_string(),
            play_dev: Arc::new(Mutex::new(Vec::new())),
            tx,
        };

        let username = "Nutzer".to_string();

        let mut game = state.game.lock().await;
        let mut play_dev = state.play_dev.lock().await;

        let token = uuid::Uuid::new_v4().to_string();
        play_dev.push(PlayerDevice { name: username.clone(), token: token.clone() });
        game.add_player(username.clone());

        assert_eq!(game.players.len(), 1);
        assert_eq!(play_dev.len(), 1);
        assert_eq!(play_dev[0].name, "Nutzer");
        assert!(!*state.game_started.lock().await, "Spiel sollte noch nicht gestartet sein");
    }


    #[tokio::test]
    async fn not_existing_player_gets_lynched(){
        let mut game = Game::new();
        game.add_player("Nutzer".into());
        let result = handle_vote(&mut game,"Nutzer".to_string(), "NichtNutzer".to_string(), ActionKind::DorfLyncht).await;

        assert_eq!(result,Err("Spieler nicht gefunden oder nicht lebendig".to_string()));
    }


    #[tokio::test]
    async fn not_existing_player_cannot_vote(){
        let mut game = Game::new();
        game.add_player("Nutzer".into());
        let result = handle_vote(&mut game,"NichtNutzer".to_string(), "Nutzer".to_string(), ActionKind::DorfLyncht).await;

        assert_eq!(result,Err("Spieler nicht gefunden oder nicht lebendig".to_string()));
    }


    #[tokio::test]
    async fn player_cannot_vote_twice(){
        let mut game = Game::new();
        game.add_player("Nutzer1".into());
        game.add_player("Nutzer2".into());
        let _ = handle_vote(&mut game,"Nutzer1".to_string(), "Nutzer2".to_string(), ActionKind::DorfLyncht).await;
        let result = handle_vote(&mut game,"Nutzer1".to_string(), "Nutzer2".to_string(), ActionKind::DorfLyncht).await;

        assert_eq!(result,Err("Du hast schon abgestimmt".to_string()));
    }


    #[tokio::test]
    async fn wrong_does_not_change_game_state(){
        let mut game = Game::new();
        game.add_player("Nutzer".into());
        let before = game.clone();
        let _ = handle_vote(&mut game,"Nutzer".to_string(), "NichtNutzer".to_string(), ActionKind::DorfLyncht).await;

        assert_eq!(game, before);
    }
    

    #[tokio::test]
        async fn reject_duplicate_name() {
        let (tx, _rx) = broadcast::channel(32);
        let state = AppState {
            game: Arc::new(Mutex::new(Game::new())),
            game_started: Arc::new(Mutex::new(false)),
            server_ip: "127.0.0.1".to_string(),
            play_dev: Arc::new(Mutex::new(Vec::new())),
            tx,
        };
        let (client_tx, mut _client_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let username = "Nutzer".to_string();
        handle_message(&state, ClientMessage::AddUser{username: username.clone()}, &client_tx).await.unwrap();
        let result = handle_message(&state, ClientMessage::AddUser{username: username.clone()}, &client_tx).await;
        
        assert_eq!(result, Err("Name existiert bereits".to_string()));
        let game =state.game.lock().await;
        assert_eq!(game.players.len(),1);
        
    }
}
