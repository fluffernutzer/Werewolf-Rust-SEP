use askama::Template;
use axum::{
    extract::{Path, State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::{Html, Redirect, Response},
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{os::macos::raw::stat, sync::Arc};
use tokio::sync::{Mutex, broadcast};
use urlencoding::encode;
use webbrowser;
use base64::{Engine as _, engine::general_purpose};
use qrcode::QrCode;

use crate::{AppState, PlayerDevice, generate_qr, logic::{Game, HexenAktion, Phase, Spieler, Winner}, roles::Rolle};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    players: Vec<PlayerTemplate<'a>>,
    phase: String,
    qr_code: String,
}

#[derive(Template)]
#[template(path = "user.html")]
struct UserTemplate<'a> {
    username: &'a str,
    rolle: &'a str,
    players: Vec<PlayerTemplate<'a>>,
    phase: String,
}

#[derive(Serialize)]
struct PlayerTemplate<'a> {
    name: &'a str,
    rolle: &'a str,
    status: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]

pub enum ClientMessage<'a> {
    StartGame,
    AddUser{ username: String },
    ReadyStatus { username: String, ready: bool },
    TagAction { direction: ActionForm },
    WerwolfAction { direction: ActionForm },
    SeherAction { direction: ActionForm },
    HexenAktion{aktion:HexenAktion, extra_target:&'a str},
    ChatMessage { sender: String, message: String },
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionForm {
    pub actor: String,
    pub target: String,
}

pub enum  ActionKind{
    DorfLyncht,
    WerwolfFrisst,
    SeherSieht,
    HexeHext,
}
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                 println!("Empfangene JSON-Nachricht: {}", text); // Debug-Ausgabe
                if let Ok(client_message) = serde_json::from_str::<ClientMessage>(&text) {
                    println!("Client Message empfangen:{}", text);
                    let mut game = state.game.lock().await;
                    match client_message {
                        ClientMessage::StartGame => {
                            if !*state.game_started.lock().await {
                                *state.game_started.lock().await = true;
                                game.phase = Phase::Tag;
                                game.runden = 1;
                                let _ = game.verteile_rollen();

                                for p in game.players.iter() {
                                    let safe_username = encode(&p.name);
                                    let url = format!("http://127.0.0.1:7878/{}", safe_username);
                                    let _ = webbrowser::open(&url);
                                }
                                
                            }
                            
                        }
                        
                        ClientMessage::ReadyStatus { username, ready } => {
                            if let Some(player) = game.players.iter_mut().find(|p| p.name == username) {
                                player.ready_state = ready;
                            }
                            if game.players.iter().all(|p| p.ready_state) {
                                *state.game_started.lock().await = true;
                                game.phase = Phase::Tag;
                                game.runden = 1;
                                let _ = game.verteile_rollen();
                                for p in game.players.iter() {
                                    let safe_username = encode(&p.name);
                                    let url = format!("http://{}:7878/{}", state.server_ip, encode(&p.name));
                                    let _ = webbrowser::open(&url);
                                }
                            }
                        }
                        ClientMessage::AddUser { username } => {
                            let token = uuid::Uuid::new_v4().to_string();
                            let player = PlayerDevice {
                                name: username.clone(),
                                token: token.clone(),};
                            state.play_dev.lock().await.push(player);
                            game.add_player(username.clone());

                        }
                        ClientMessage::TagAction { direction } => {
                            if let Phase::Tag = game.phase {
                                game.tag_lynchen(&direction.target);
                                //let _ = handle_vote(& mut game, &direction.actor, &direction.target, ActionKind::DorfLyncht);
                                game.runden +=1;
                            }
                        }
                        ClientMessage::WerwolfAction { direction } => {
                            if let Phase::WerwölfePhase = game.phase {
                                let _ = match game.werwolf_toetet(&direction.actor,&direction.target){
                                    Ok(()) => println!("Tötung ausgeführt"),
                                    Err(String) => println!("Fehler beim töten"),
                                };
                                game.runden +=1;
                            }
                        }
                        ClientMessage::SeherAction { direction } => {
                            if let Phase::SeherPhase = game.phase {
                                    let rolle   =match  game.seher_schaut(&direction.target){
                                        Ok(rolle) => Some(rolle),
                                        Err(msg) =>None,
                                    };
                                    game.last_seher_result = Some((direction.target.clone(), rolle.unwrap()));
                                    game.runden +=1; 
                            }
                        }
                        ClientMessage::HexenAktion { aktion, extra_target } => {
                            let _ = game.hexe_arbeitet(aktion, extra_target);
                            game.runden +=1;
                        }
                        ClientMessage::ChatMessage { sender, message } => {
                            let chat_message = json!({
                                "type": "CHAT_MESSAGE",
                                "data": {
                                    "sender": sender,
                                    "message": message,
                                }
                            });
                            let chat_message_str = serde_json::to_string(&chat_message).expect("Fehler beim Serialisieren der Chat-Nachricht");
                            let _ = state.tx.send(chat_message_str);
                        }
                        
                    }}else {
                    eprintln!("Fehler beim Deserialisieren der Nachricht: {}", text);
                    }
            }
            send_game_state(&state).await;
        }
                    
    });

    
    while let Ok(msg) = rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

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
        }
    });

    let message_str = serde_json::to_string(&message).expect("Fehler beim Serialisieren des GameState");
    println!("Sende GameState: {}", message_str); // Debug-Ausgabe
    let _ = state.tx.send(message_str);

    if let Some(winner) = win && *game_started {
        let winner_message = json!({
            "type": "WINNER",
            "winner": format!("{:?}", winner)
        });
        let winner_message_str = serde_json::to_string(&winner_message).expect("Fehler beim Serialisieren der Winner-Nachricht");
        println!("Sende Winner-Nachricht: {}", winner_message_str);
        let _ = state.tx.send(winner_message_str);
    }
}


pub async fn index(State(state): State<AppState>) -> Html<String> {
    let game = state.game.lock().await;
    let qr_svg=generate_qr(&state.server_ip);
    let qr_code_base64 = general_purpose::STANDARD.encode(qr_svg.as_bytes());

    let players: Vec<PlayerTemplate> = game.players
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

pub async fn show_user(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Html<String> {
    let game = state.game.lock().await;

    let rolle = match game.rolle_von(&username) {
        Some(rolle) => match rolle {
                    Rolle::Werwolf => "Werwolf",
                    Rolle::Seher => "Seher",
                    Rolle::Hexe => "Hexe",
                    Rolle::Amor => "Amor",
                    Rolle::Jäger => "Jäger",
                    _ => "Dorfbewohner",
                },
        None => "?",
    };

    let players: Vec<PlayerTemplate> = game.players
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
    actor: &str,
    target: &str,
    action: ActionKind,
) -> Result<(), String> {
    // 1. Setze has_voted für den Spieler
    if let Some(player) = game.players.iter_mut().find(|p| p.name == actor && p.lebend) {
        player.has_voted = true;
    } else {
        return Err("Spieler nicht gefunden oder nicht lebendig".to_string());
    }

    // 2. Sammle die Namen der berechtigten Spieler (ohne game.phase erneut zu borgen)
    let eligible_player_names: Vec<String> = game.players.iter()
        .filter(|p| {
            p.lebend && match game.phase {
                Phase::Tag => true,
                Phase::WerwölfePhase => p.rolle == Rolle::Werwolf,
                Phase::SeherPhase => p.rolle == Rolle::Seher,
                Phase::HexePhase => p.rolle == Rolle::Hexe,
                Phase::AmorPhase => p.rolle == Rolle::Amor,
            }
        })
        .map(|p| p.name.clone())
        .collect();

    // 3. Prüfe, ob alle berechtigten Spieler abgestimmt haben
    let all_voted = eligible_player_names.iter()
        .all(|name| game.players.iter().find(|p| p.name == *name).unwrap().has_voted);

    // 4. Falls alle abgestimmt haben, führe die Aktion aus
    if all_voted {
        match action {
            ActionKind::DorfLyncht => game.tag_lynchen(target),
            ActionKind::WerwolfFrisst => game.werwolf_toetet("Werwölfe", target)?,
            ActionKind::HexeHext =>(),
            ActionKind::SeherSieht =>(),
        };

        // 5. Setze has_voted für alle berechtigten Spieler zurück
        for player in game.players.iter_mut() {
            if eligible_player_names.contains(&player.name) {
                player.has_voted = false;
            }
        }
    }

    Ok(())
}
fn dorf_lyncht (game: &mut Game, actor: &str, target: &str) -> Result<(), String> {
    // Logik für Werwolf-Tötung
    game.tag_lynchen(target);
    game.phase_change();
    Ok(())
}