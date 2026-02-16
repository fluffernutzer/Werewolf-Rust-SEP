use std::collections::HashMap;
use std::fmt::Display;
use futures::future::err;
use log::info;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use crate::roles::Rolle;
use crate::roles::Team;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Tag,
    //Nacht,
    AmorPhase,
    WerwölfePhase,
    SeherPhase,
    HexePhase,
    PriesterPhase,
    DoktorPhase,
}

#[derive(Debug,Copy, Clone,serde::Serialize,serde::Deserialize)]
pub enum HexenAktion{
    Heilen,
    NichtsTun,
    Vergiften,
}

#[derive(Debug, Clone)]
pub struct Spieler {
    pub name: String,
    pub team: Team,
    pub rolle: Rolle,
    pub lebend: bool,
    pub bereits_gesehen:bool,
    //Für Websocket-Abstimmungen/Bereit zum Spielen: 
    pub ready_state: bool,
    pub has_voted: bool,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub players: Vec<Spieler>,
    pub phase: Phase,
    pub runden: u32,
    pub heiltrank_genutzt:bool,
    pub bereits_getoetet: bool,
    pub tag_opfer: Option<String>,
    pub nacht_opfer: Option<String>,
    pub hexe_opfer:Option<String>,
    pub geheilter_von_hexe:Option<String>,
    pub liebender_1:Option<String>,
    pub liebender_2:Option<String>,
    pub liebende_aktiv:bool,
    pub amor_hat_gewaehlt:bool,
    pub jaeger_ziel:Option<String>,
    pub last_seher_result:Option<(String,Rolle)>,
    //pub amor_done:bool,
    //pub werwoelfe_done:bool,
    //pub seher_done:bool,
    pub hexe_done:bool,
    //pub abstimmung_done:bool,
    pub geschuetzter_von_doktor:Option<String>,
    pub priester_hat_geworfen: bool,
    pub abstimmung_done:bool,
    //
    pub votes: HashMap<String,Vec<String>>,
    //pub abstimmung_done:bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Winner {
    Dorf,
    Werwolf,
}

impl Spieler {
    pub fn new(name: String, team: Team, rolle: Rolle, lebend:bool) -> Self {
        Spieler {
            name,
            team: rolle.team(),
            rolle,
            lebend:true,
            bereits_gesehen:false,
            ready_state:false,
            has_voted:false,
        }
    }
}


impl Game {
    pub fn new() -> Self {
        Game {
            players: Vec::new(),
            phase: Phase::Tag,
            runden: 1,
            heiltrank_genutzt: false,
            bereits_getoetet: false,
            tag_opfer: None,
            nacht_opfer:None,
            hexe_opfer:None,
            geheilter_von_hexe:None,
            liebender_1:None,
            liebender_2:None,
            liebende_aktiv:false,
            amor_hat_gewaehlt:false,
            jaeger_ziel:None,
            last_seher_result:None,
            //amor_done:false,
            //werwoelfe_done:false,
            //seher_done:false,
            hexe_done:false,
            //abstimmung_done:false,
            geschuetzter_von_doktor:None,
            priester_hat_geworfen: false, 
            abstimmung_done:false,
            //
            votes: HashMap::new(),
            //abstimmung_done:false,

        }
    }

    pub fn add_player(&mut self, name: String) {
        self.players.push(Spieler {
            name,
            team: Team::TeamDorf, //Platzhalter wird noch durchgemischt
            rolle: Rolle::Dorfbewohner, //Platzhalter wird noch durchgemischt
            lebend: true,
            bereits_gesehen:false,
            ready_state:false,
            has_voted:false,
        });
    }

    pub fn verteile_rollen(&mut self)->Result<(),String>{
        let mut rng= thread_rng();

        let anzahl_spieler=self.players.len();
        if anzahl_spieler <3{
            return Err ("Es müssen mindestens 3 Spieler vorhanden sein.".to_string());
        }
        let anzahl_werwoelfe= anzahl_spieler/3;
        //kann noch erweitert werden für mehr rollen
        let rollen_steps=vec![
            (4, Rolle::Seher),
            (5,Rolle::Hexe),
            (6,Rolle::Amor),
            (7,Rolle::Jäger),
            (8,Rolle::Doktor),
            (8,Rolle::Priester),
        ];

        let mut roles=Vec::new();

        for _ in 0..anzahl_werwoelfe{
            roles.push(Rolle::Werwolf);
        }
       for (min_anzahl,rolle)in rollen_steps{
        if anzahl_spieler>=min_anzahl{
            roles.push(rolle);
        }
       }
       while roles.len()<anzahl_spieler{
        roles.push(Rolle::Dorfbewohner);
       }

        roles.shuffle(&mut rng);

        for (player, role) in self.players.iter_mut().zip(roles.into_iter()) {
            player.rolle = role;
            player.team=role.team();
        }
        Ok(())
    }

    pub fn rolle_von(&self, name: &str) -> Option<&Rolle> {
        self.players
            .iter()
            .find(|p| p.name == name)
            .map(|p| &p.rolle)
    }

    pub fn tag_lynchen(&mut self, name: &str) {
        /*if self.runden==1{
            println!("(TAG) In Runde 1 wird nicht gelyncht.");
        self.phase_change();
        } else {
        self.nacht_opfer=Some(name.to_string());
        println!("(TAG) Dorf lyncht {}", name);
        self.abstimmung_done=true;
        self.phase_change();}}*/
        self.tag_opfer = Some(name.to_string());
        //println!("(TAG) Dorf lyncht {}", name);
        log::info!("(TAG) Dorf lyncht {}", name);
        self.spieler_stirbt(name);
        self.phase_change();
    }
    pub fn check_win(&self) -> Option<Winner> {
        let lebende_werwoelfe = self.players
            .iter()
            .filter(|p| p.lebend && p.team == Team::TeamWerwolf)
            .count();

        let lebende_dorfspieler = self.players
            .iter()
            .filter(|p| p.lebend && p.team != Team::TeamWerwolf)
            .count();

        if lebende_werwoelfe == 0 {
            return Some(Winner::Dorf);
        }

        if lebende_werwoelfe >= lebende_dorfspieler {
            return Some(Winner::Werwolf);
        }

        None
    }

    pub fn spieler_stirbt(&mut self, verstorbener:&str){
        let player=self.players.iter_mut().find(|p| p.name == verstorbener);
        if player.is_none(){
            log::info!("Spieler {}existiert nicht", verstorbener);
            //println!("Spieler {}existiert nicht", verstorbener);
            return;
        }
        let victim = player.unwrap();
        if !victim.lebend{
            log::info!("spieler bereits tot.");
            //println!("spieler bereits tot.");
            return;
        }

        victim.lebend=false;
        log::info!("Spieler {} ist gestorben.", verstorbener);
        println!("Spieler {} ist gestorben.", verstorbener);

        if victim.rolle==Rolle::Jäger{
            log::info!("{} war der Jäger und schießt nun.", verstorbener);
            //println!("{} war der Jäger und schießt nun.", verstorbener);
            //brauche ziel vom frontend

            if let Some(ziel)= self.jaeger_ziel.clone(){
                log::info!("Der Jäger erschießt {}.",ziel);
                //println!("Der Jäger erschießt {}.",ziel);
                self.spieler_stirbt(&ziel);
            }else {
                log::info!("Der Jäger hat niemanden ausgewählt.");
                //println!("Der Jäger hat niemanden ausgewählt.");
            }
        }

        let ist_liebender_1 = self.liebender_1.as_ref().map(|s| s == verstorbener).unwrap_or(false); 
        let ist_liebender_2 = self.liebender_2.as_ref().map(|s| s == verstorbener).unwrap_or(false);
        
        if !(ist_liebender_1||ist_liebender_2){
            return;
        }

        let liebespartner = if ist_liebender_1{
            self.liebender_2.clone()
        } else {
            self.liebender_1.clone()
        };

        if let Some (liebespartner_name)= liebespartner{
            if let Some(p) = self.players.iter().find(|p| p.name == liebespartner_name) {
                if p.lebend{
                    log::info!("{} stirbt vor Kummer.", liebespartner_name);
                    //println!("{} stirbt vor Kummer.", liebespartner_name);
                    self.spieler_stirbt(&liebespartner_name);
                }
        }
        }
        self.liebende_aktiv=false;
        
       if let Some(winner) = self.check_win() {
        log::info!("SPIEL BEENDET: {:?} gewinnt!", winner);
        //println!("SPIEL BEENDET: {:?} gewinnt!", winner);
        } 

    }

    pub fn nacht_aufloesung(&mut self){
        
        let opfer_name=self.nacht_opfer.clone();
        if let Some(opfer)=opfer_name{
            if self.geheilter_von_hexe.as_ref()==Some(&opfer){
                println!("{} wurde von der hexe geheilt.",opfer);
            } else if self.geschuetzter_von_doktor.as_ref()==Some(&opfer){
                println!("{} wurde von dem Doktor beschützt.", opfer);
            } else {
            self.spieler_stirbt(&opfer);
        }}

        let zusaetzliches_opfer_name=self.hexe_opfer.clone();
        if let Some(zusaetzliches_opfer)=zusaetzliches_opfer_name{
            if self.geschuetzter_von_doktor.as_ref()==Some(&zusaetzliches_opfer){
                println!("{} wurde von dem Doktor beschützt.", zusaetzliches_opfer);
            } else {
                self.spieler_stirbt(&zusaetzliches_opfer);
            }
        }

        self.nacht_opfer=None;
        self.hexe_opfer=None;
        self.geheilter_von_hexe=None;
        self.geschuetzter_von_doktor=None;
    }
}
