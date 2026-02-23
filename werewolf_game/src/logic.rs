use std::collections::HashMap;
//use std::fmt::Display;
//use futures::future::err;
//use log::info;
//use std::fmt::Display;
//use futures::future::err;
//use log::info;
//use std::fmt::Display;
////use futures::future::err;
////use log::info;
use rand::seq::SliceRandom;
use rand::rng;
//use serde::Serialize;
use rand::rng;
//use serde::Serialize;
use crate::roles::Rolle;
use crate::roles::Team;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Tag,
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

#[derive(Debug, Clone, PartialEq)]
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
#[derive(Debug, Clone, PartialEq)]
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
    //pub hexe_done:bool,
    //pub hexe_done:bool,
    //pub abstimmung_done:bool,
    pub geschuetzter_von_doktor:Option<String>,
    pub priester_hat_geworfen: bool,
    //pub abstimmung_done:bool,
    
    //pub abstimmung_done:bool,
    
    pub votes: HashMap<String,Vec<String>>,
    
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Winner {
    Dorf,
    Werwolf,
    Liebende,
}


/*impl Spieler {
    pub fn new(name: String, _team: Team, rolle: Rolle, _lebend:bool) -> Self {
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
}*/



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
            ////hexe_done:false,
            //abstimmung_done:false,
            geschuetzter_von_doktor:None,
            priester_hat_geworfen: false, 
            ////abstimmung_done:false,
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
        if anzahl_spieler>15{
            return Err ("Es dürfen maximal 16 Spieler mitspielen.".to_string());
        }
        let anzahl_werwoelfe= anzahl_spieler/3;
        
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
        if self.runden==1{
            log::info!("Runde 1: Niemand wird gelyncht.");
        self.phase_change();
        } else {
        self.tag_opfer = Some(name.to_string());
        log::info!("(TAG) Dorf lyncht {}", name);
        self.spieler_stirbt(name);
        self.phase_change();
    }}
    pub fn check_win(&self) -> Option<Winner> {
        let mut werwoelfe = 0;
        let mut dorf = 0;
        let mut liebende = 0;

        for p in &self.players {
            if p.lebend {
                match p.team {
                    Team::TeamWerwolf => werwoelfe += 1,
                    Team::TeamDorf => dorf += 1,
                    Team::TeamLiebende => liebende += 1,
                }
            }
        }

        if werwoelfe > 0 && dorf == 0 && liebende == 0 {
            return Some(Winner::Werwolf);
        }
        if dorf > 0 && werwoelfe == 0 && liebende == 0 {
            return Some(Winner::Dorf);
        }
        if liebende > 0 && werwoelfe == 0 && dorf == 0 {
            return Some(Winner::Liebende);
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



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_new_game(){
        let game=Game::new();

        assert_eq!(game.players.len(), 0);
        assert_eq!(game.phase, Phase::Tag);
        assert_eq!(game.heiltrank_genutzt, false);
        assert_eq!(game.bereits_getoetet, false);
        assert_eq!(game.tag_opfer, None);
        assert_eq!(game.nacht_opfer, None);
        assert_eq!(game.hexe_opfer, None);
        assert_eq!(game.geheilter_von_hexe, None);
        assert_eq!(game.liebende_aktiv, false);  
        assert_eq!(game.amor_hat_gewaehlt,false);
        assert_eq!(game.jaeger_ziel,None);
        assert_eq!(game.geschuetzter_von_doktor, None);
        assert_eq!(game.priester_hat_geworfen,false); 
         
    }

    #[test]
    fn test_add_player(){
        let mut game=Game::new();
        
        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        assert_eq!(game.players.len(),3);
        
    }

    #[test]
    fn test_verteile_rollen(){
        let mut game=Game::new();
        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        let result=game.verteile_rollen();
        assert!(result.is_ok());

        let anzahl_werwoelfe=game.players
                                        .iter()
                                        .filter(|p| matches!(p.rolle, Rolle::Werwolf))
                                        .count();
        assert_eq!(anzahl_werwoelfe,1);
    }

    #[test]
    fn test_verteile_rollen_2(){
        let mut game=Game::new();
        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        

        let result=game.verteile_rollen();
        assert!(result.is_err());
    }

    #[test]
    fn test_rolle_von(){
        let mut game=Game::new();

        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.verteile_rollen();

        let rolle=game.rolle_von("Name1");

        assert!(rolle.is_some());

    }

    #[test]
    fn test_rolle_von2(){
        let mut game=Game::new();

        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.verteile_rollen();

        let rolle=game.rolle_von("Unbekannt");

        assert!(rolle.is_none());
        
        }

    #[test]
    fn test_tag_lynchen(){
        let mut game=Game::new();

        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.verteile_rollen().unwrap();

        assert_eq!(game.phase, Phase::Tag);

        game.tag_lynchen("Name1");

        assert!(game.tag_opfer.is_some());
        assert_eq!(game.tag_opfer.as_ref().unwrap(),"Name1");
        assert!(!game.players[0].lebend);
        assert_eq!(game.phase, Phase::WerwölfePhase);
       
      
    }

    #[test]
    fn test_check_win_dorf(){
        let mut game=Game::new();

        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.verteile_rollen().unwrap();

        for p in game.players.iter_mut(){
            if p.rolle==Rolle::Werwolf{
                p.lebend=false;
            }
        }

        let result=game.check_win();

        assert_eq!(result, Some(Winner::Dorf));

    }

    #[test]
    fn test_check_win_werwoelfe(){
        let mut game=Game::new();

        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.verteile_rollen().unwrap();
        for p in game.players.iter_mut(){
            if p.rolle!=Rolle::Werwolf{
                p.lebend=false;
            }
        }

        let result=game.check_win();

        assert_eq!(result, Some(Winner::Werwolf));

    }

    #[test]
    fn test_liebespaar(){
        let mut game=Game::new();
        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.liebender_1=Some("Name2".into());
        game.liebender_2=Some("Name3".into());
        game.liebende_aktiv=true;

        game.spieler_stirbt("Name2");

        assert!(!game.players[1].lebend);
        assert!(!game.players[2].lebend);

    }
     
    #[test]
    fn test_jaeger(){
        let mut game=Game::new();
        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.players[0].rolle=Rolle::Jäger;
        game.jaeger_ziel=Some("Name3".into());
        game.spieler_stirbt("Name1");

        assert!(!game.players[2].lebend);
    }

    #[test]
    fn test_spieler_stirbt_normaler_fall(){
        let mut game=Game::new();
        game.add_player("Name1".to_string());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.spieler_stirbt("Name1");

        assert!(!game.players[0].lebend);
    }

    #[test]
    fn spieler_stirbt_doppelt(){
        let mut game=Game::new();
        game.add_player("Name1".into());
        game.add_player("Name2".to_string());
        game.add_player("Name3".to_string());

        game.spieler_stirbt("Name1"); 
        game.spieler_stirbt("Name1");

        assert!(!game.players[0].lebend);
    }

    #[test]
    fn test_werwolf_toetet() {
        let mut game = Game::new();
        game.add_player("Wolf".into());
        game.add_player("Opfer".into());
        game.players[0].rolle = Rolle::Werwolf;
        game.phase = Phase::WerwölfePhase;
        
        let result = game.werwolf_toetet("Wolf", "Opfer");
        assert!(result.is_ok());
        assert_eq!(game.nacht_opfer, Some("Opfer".to_string()));
    }

    #[test]
    fn test_seher_schaut() {
        let mut game = Game::new();
        game.add_player("Seher".into());
        game.add_player("Ziel".into());
        game.players[0].rolle = Rolle::Seher;
        game.players[1].rolle = Rolle::Werwolf;
        game.phase = Phase::SeherPhase;
        
        let rolle = game.seher_schaut("Ziel").unwrap();
        assert_eq!(rolle, Rolle::Werwolf);
    }

    #[test]
    fn test_hexe_heilt() {
        let mut game = Game::new();
        game.add_player("Hexe".into());
        game.players[0].rolle = Rolle::Hexe;
        game.phase = Phase::HexePhase;
        game.nacht_opfer = Some("X".to_string());
        
        let result = game.hexe_arbeitet(HexenAktion::Heilen, "Hexe", "");
        assert!(result.is_ok());
        assert!(game.heiltrank_genutzt);
    }

    #[test]
    fn test_hexe_vergiftet() {
        let mut game = Game::new();
        game.add_player("Hexe".into());
        game.add_player("Ziel".into());
        game.players[0].rolle = Rolle::Hexe;
        game.phase = Phase::HexePhase;
        
        let result = game.hexe_arbeitet(HexenAktion::Vergiften, "Hexe", "Ziel");
        assert!(result.is_ok());
        assert_eq!(game.hexe_opfer, Some("Ziel".to_string()));
    }

    #[test]
    fn test_amor_verliebt() {
        let mut game = Game::new();
        game.add_player("Amor".into());
        game.add_player("A".into());
        game.add_player("B".into());
        game.players[0].rolle = Rolle::Amor;
        game.phase = Phase::AmorPhase;
        
        let result = game.amor_waehlt("A", "B");
        assert!(result.is_ok());
        assert_eq!(game.liebender_1, Some("A".to_string()));
        assert_eq!(game.liebender_2, Some("B".to_string()));
    }

    #[test]
    fn test_jaeger_schiesst() {
        let mut game = Game::new();
        game.add_player("Jäger".into());
        game.add_player("Ziel".into());
        game.players[0].rolle = Rolle::Jäger;
        game.jaeger_ziel = Some("Ziel".to_string());
        
        game.spieler_stirbt("Jäger");
        assert!(!game.players[1].lebend);
    }

    #[test]
    fn test_doktor_schuetzt() {
        let mut game = Game::new();
        game.add_player("Doktor".into());
        game.add_player("Patient".into());
        game.players[0].rolle = Rolle::Doktor;
        game.phase = Phase::DoktorPhase;
        
        let _ = game.doktor_schuetzt("Patient");
        assert_eq!(game.geschuetzter_von_doktor, Some("Patient".to_string()));
    }

    #[test]
    fn test_priester_tötet_werwolf() {
        let mut game = Game::new();
        game.add_player("Priester".into());
        game.add_player("Wolf".into());
        game.players[0].rolle = Rolle::Priester;
        game.players[1].rolle = Rolle::Werwolf;
        game.phase = Phase::PriesterPhase;
        
        let result = game.priester_wirft("Priester", Some("Wolf"));
        assert!(result.is_ok());
        assert!(!game.players[1].lebend);
    }

    #[test]
    fn test_priester_stirbt_bei_falschem_ziel() {
        let mut game = Game::new();
        game.add_player("Priester".into());
        game.add_player("Dorf".into());
        game.players[0].rolle = Rolle::Priester;
        game.players[1].rolle = Rolle::Dorfbewohner;
        game.phase = Phase::PriesterPhase;
        
        let result = game.priester_wirft("Priester", Some("Dorf"));
        assert!(result.is_ok());
        assert!(!game.players[0].lebend); // Priester tot
        assert!(game.players[1].lebend);  // Dorf lebt
    }

    #[test]
    fn test_liebespaar_stirbt_zusammen() {
        let mut game = Game::new();
        game.add_player("A".into());
        game.add_player("B".into());
        game.liebender_1 = Some("A".to_string());
        game.liebender_2 = Some("B".to_string());
        
        game.spieler_stirbt("A");
        assert!(!game.players[1].lebend);
    }

    #[test]
    fn test_liebende_gewinnen() {
        let mut game = Game::new();
        game.add_player("L1".into());
        game.add_player("L2".into());
        for p in &mut game.players {
            p.lebend = true;
            p.team = Team::TeamLiebende;
        }
        assert_eq!(game.check_win(), Some(Winner::Liebende));
    }
}