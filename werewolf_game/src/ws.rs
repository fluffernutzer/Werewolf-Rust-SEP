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
//use qrcode::QrCode;
//use std::sync::Arc;
use tokio::sync::{mpsc};
//use urlencoding::encode;
//use webbrowser;

use crate::{
    AppState, PlayerDevice, generate_qr,
    logic::{Game, HexenAktion, Phase},
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]

pub enum ClientMessage<'a> {
    StartGame,
    ResetGame,
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
        direction: ActionForm,
    },
    HexenAktion{direction: ActionForm, hexen_aktion:HexenAktion, extra_target:&'a str},
    AmorAktion {direction:ActionForm, target1: &'a str, target2:&'a str },
    DoktorAction { direction: ActionForm },
    PriesterAction { actor: &'a str, target: Option<&'a str> },
    ChatMessage {
        sender: String,
        message: String,
    },
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionForm {
    pub actor: String,
    pub target: String,
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
pub async fn handle_message( state: &AppState,client_message: ClientMessage<'_>,client_tx: &mpsc::UnboundedSender<String>) -> Result<(), String> {
    let mut game = state.game.lock().await;
    let recv_state = state.clone();
    
    match client_message {

                ClientMessage::StartGame => {
                    if !*recv_state.game_started.lock().await {
                        *recv_state.game_started.lock().await = true;

                        game.phase = Phase::Tag;
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
                                println!("Zurüclsetzen beendet")

                }
                ClientMessage::ReadyStatus { username, ready } => {
                    if let Some(player) = game.players.iter_mut().find(|p| p.name == username) {
                        player.ready_state = ready;
                    }

                    if game.players.iter().all(|p| p.ready_state) {
                        *recv_state.game_started.lock().await = true;

                        game.phase = Phase::Tag;
                        game.runden = 1;
                        let _ = game.verteile_rollen();

                        let _ = recv_state.tx.send(serde_json::json!({
                            "type": "GAME_STARTED"
                        }).to_string());
                    }
                }

                ClientMessage::AddUser { username } => {
                    if game.players.iter().any(|p| p.name == username) {
                        return Err("Name existiert bereits".to_string());
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
                        //let update = game.tag_lynchen(&direction.target);
                                let _ = handle_vote(& mut game, &direction.actor, &direction.target, ActionKind::DorfLyncht);
                                //game.runden +=1;
                    }
                }

                ClientMessage::WerwolfAction { direction } => {
                    if let Phase::WerwölfePhase = game.phase {
                        /*let _ = match game.werwolf_toetet(&direction.actor,&direction.target){
                                    Ok(()) => println!("Tötung ausgeführt"),
                                    Err(String) => println!("Fehler beim töten"),
                                };
                                
                                game.runden +=1;*/
                                let _ = handle_vote(& mut game, &direction.actor, &direction.target, ActionKind::WerwolfFrisst);
                    }
                }

                ClientMessage::SeherAction { direction } => {
                    if let Phase::SeherPhase = game.phase {
                        if let Ok(rolle) = game.seher_schaut(&direction.target) {
                            game.last_seher_result =
                                Some((direction.target.clone(), rolle));
                        }
                        game.runden += 1;
                    }
                }

                ClientMessage::HexenAktion {direction, hexen_aktion , extra_target } => {
                            let _ = game.hexe_arbeitet(hexen_aktion, &direction.actor ,extra_target);
                            //println!("An Hexe übergeben: {:?},{},{}",hexenAktion, &direction.actor ,extra_target);
                            game.runden +=1;
                }
                ClientMessage::AmorAktion { direction:_, target1, target2 } =>{
                            let _ = game.amor_waehlt(target1, target2);
                            game.runden +=1;
                }
                ClientMessage::DoktorAction { direction } => {
                            let _ = game.doktor_schuetzt(&direction.target);
                            game.runden +=1;
                }
                ClientMessage::PriesterAction { actor, target} => {
                            let _ = game.priester_wirft(&actor, target);
                            game.runden +=1;
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

async fn handle_vote(game: &mut Game,actor: &str,target: &str,action: ActionKind ) -> Result<(), String> {
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

    let eligible_player_names: Vec<String> = game
        .players
        .iter()
        .filter(|p| {
            p.lebend && match game.phase {
                Phase::Tag => true,
                Phase::WerwölfePhase => p.rolle == Rolle::Werwolf,
                Phase::SeherPhase => p.rolle == Rolle::Seher,
                Phase::HexePhase => p.rolle == Rolle::Hexe,
                Phase::AmorPhase => p.rolle == Rolle::Amor,
                Phase::PriesterPhase=> p.rolle == Rolle::Priester,
                Phase::DoktorPhase=>p.rolle==Rolle::Doktor,
            }
        })
        .map(|p| p.name.clone())
        .collect();
    println!("erlaubte Stimmen:{:?}",eligible_player_names);
    game.votes.entry(target.to_string()).or_default().push(actor.to_string());
    println!("Aktuelle Stimmen: {:?}", game.votes);
    let all_voted = eligible_player_names.iter().all(|name| {
        game.players
            .iter()
            .find(|p| p.name == *name)
            .unwrap()
            .has_voted
    });

    if all_voted {
        let final_target = game.votes.iter()
        .max_by_key(|(_, voters)| voters.len())
        .map(|(target, _)| target.clone());
        if let Some(target) = final_target {
            match action {
                ActionKind::DorfLyncht => game.tag_lynchen(&target),
                ActionKind::WerwolfFrisst => game.werwolf_toetet(&actor, &target)?,
                //ActionKind::HexeHext => (),
                //ActionKind::SeherSieht => (),
        }};
        
        game.runden +=1;
        game.votes.clear();

        for player in game.players.iter_mut() {
            if eligible_player_names.contains(&player.name) {
                player.has_voted = false;
            }
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
        let result = handle_vote(&mut game,"Nutzer", "NichtNutzer", ActionKind::DorfLyncht).await;

        assert_eq!(result,Err("Spieler nicht gefunden oder nicht lebendig".to_string()));
    }


    #[tokio::test]
    async fn not_existing_player_cannot_vote(){
        let mut game = Game::new();
        game.add_player("Nutzer".into());
        let result = handle_vote(&mut game,"NichtNutzer", "Nutzer", ActionKind::DorfLyncht).await;

        assert_eq!(result,Err("Spieler nicht gefunden oder nicht lebendig".to_string()));
    }


    #[tokio::test]
    async fn player_cannot_vote_twice(){
        let mut game = Game::new();
        game.add_player("Nutzer1".into());
        game.add_player("Nutzer2".into());
        let _ = handle_vote(&mut game,"Nutzer1", "Nutzer2", ActionKind::DorfLyncht).await;
        let result = handle_vote(&mut game,"Nutzer1", "Nutzer2", ActionKind::DorfLyncht).await;

        assert_eq!(result,Err("Du hast schon abgestimmt".to_string()));
    }


    #[tokio::test]
    async fn wrong_does_not_change_game_state(){
        let mut game = Game::new();
        game.add_player("Nutzer".into());
        let before = game.clone();
        let _ = handle_vote(&mut game,"Nutzer", "NichtNutzer", ActionKind::DorfLyncht).await;

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