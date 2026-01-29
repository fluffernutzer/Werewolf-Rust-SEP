use rand::seq::SliceRandom;
use rand::thread_rng;
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
}

#[derive(Debug, Clone)]
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
    pub liebender_1:Option<String>,
    pub liebender_2:Option<String>,
    pub liebende_aktiv:bool,
    pub amor_hat_gewaehlt:bool,
    pub jaeger_ziel:Option<String>,
    pub last_seher_result:Option<(String,Rolle)>,
    pub amor_done:bool,
    pub werwoelfe_done:bool,
    pub seher_done:bool,
    pub hexe_done:bool,
    pub abstimmung_done:bool,
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
            liebender_1:None,
            liebender_2:None,
            liebende_aktiv:false,
            amor_hat_gewaehlt:false,
            jaeger_ziel:None,
            last_seher_result:None,
            amor_done:false,
            werwoelfe_done:false,
            seher_done:false,
            hexe_done:false,
            abstimmung_done:false,

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
        println!("(TAG) Dorf lyncht {}", name);
        self.spieler_stirbt(name);
        self.phase_change();
    }
    

    pub fn werwolf_toetet(&mut self, actor_name:&str, victim_name: &str) ->Result<(),String>{
         if self.phase!=Phase::WerwölfePhase{
            return Err("Die Werwölfe sind gerade nicht dran.".into());
        }
        let lebende_werwoelfe= self.players.iter().any(|p| p.rolle==Rolle::Werwolf&&p.lebend);
        if !lebende_werwoelfe{
            return Err("Es gibt keine lebenden Werwölfe mehr".into());
        }
        let werwolf=self
                .players
                .iter()
                .find(|p|p.name==actor_name&&p.rolle==Rolle::Werwolf)
                .ok_or("Du bist kein Werwolf.")?;

        if !werwolf.lebend{
            return Err("Nur lebende Werwölfe dürfen wählen.".into());
        }


        let target=self.players
                    .iter()
                    .find(|p| p.name==victim_name)
                    .ok_or("Zielperson existiert nicht.")?;

        if !target.lebend{
            return Err("Der Spieler ist bereits tot.".into());
        }
         if actor_name==victim_name{
            return Err("Man kann sich nicht selbst wählen.".into());
         }

        println!("(NACHT) Werwölfe greifen {} an", victim_name);
        self.spieler_stirbt(&victim_name);
        self.werwoelfe_done=true;
        self.phase_change();
        Ok(())
    }



pub fn seher_schaut(&mut self, target_name: &str) -> Result<Rolle,String> {
        if self.phase!=Phase::SeherPhase{
           return Err("Seher ist nicht dran.".into());
        }
        let seher_index=self.players
        .iter()
        .position(|p| p.rolle==Rolle::Seher).ok_or("Kein Seher im Spiel")?;

        let target_index=self.players
        .iter()
        .position(|p|p.name==target_name).ok_or("Ziel existiert nicht.")?;

        let target_rolle=self.players[target_index].rolle;
        let target_lebend=self.players[target_index].lebend;
        if !target_lebend{
            return Err("Opfer lebt nicht mehr".into());
        }
        let seher =&mut self.players[seher_index];

        if !seher.lebend{
            return Err("Der Seher lebt nicht mehr und kann deswegen nicht mehr sehen.".into());
        }
        if seher.bereits_gesehen{
            return Err("Der Seher hat bereits einmal diese Runde gesehen.".into());
        }
        
        println!("(NACHT) Seher überprüft {}", target_name);

       seher.bereits_gesehen=true;
        self.seher_done=true;
        
        self.phase_change();
        Ok(target_rolle)
    }


    
    //Hexe darf nur einmal heilen und nur einmal töten

    pub fn hexe_arbeitet(&mut self, aktion:HexenAktion, actor_name:&str, extra_target:&str)->Result<(),String>{
        let hexe=self.players
                        .iter()
                        .find(|p| p.rolle==Rolle::Hexe)
                        .ok_or("Es gibt keine Hexe im Spiel.")?;
                if !hexe.lebend{
                    return Err("Du bist aus dem Spiel schon raus.".into());
                }
                
        match aktion {
            HexenAktion::Heilen=> {
                if self.heiltrank_genutzt{
                    return Err("Hexe hat ihren Heiltrank bereits einmal benutzt.".into());
                }
                
                let opfer_name=self.nacht_opfer.as_ref().ok_or("Es gibt kein Opfer.")?;
                let geheilter=self.players
                        .iter_mut()
                        .find(|p| p.name==*opfer_name)
                        .ok_or("Spieler nicht gefunden.")?;

                geheilter.lebend=true;

                println!("(Nacht) Hexe heilt {}", opfer_name);

                self.nacht_opfer=None;
                self.heiltrank_genutzt=true;
                self.hexe_done=true;
                self.phase_change();
                Ok(())
            }

            HexenAktion::Vergiften=>{
                 if self.bereits_getoetet{
                    return Err("Die Hexe darf nur einmal im Spiel jemanden vergiften.".into());
                }
               let zusaetzliches_opfer=self.players
                        .iter_mut()
                        .find(|p|p.name==extra_target)
                        .ok_or("Opfer konnte nicht gefunden werden")?;

                if !zusaetzliches_opfer.lebend{
                    return Err("Das Opfer ist bereits tot.".into());
                }
                if extra_target==actor_name{
                    return Err("Du kannst dich nicht selber vergiften.".into());
                }
                self.hexe_opfer=Some(extra_target.to_string());
                self.spieler_stirbt(&extra_target);
                self.bereits_getoetet=true;
                println!("Hexe tötet noch dazu: {}", extra_target);
                self.hexe_done=true;
                self.phase_change();
                Ok(())
            }

            HexenAktion::NichtsTun=>{
                println!("Hexe tut nichts.");
                self.hexe_done=true;
                self.phase_change();
                Ok(())
            }

        }
    }

     pub fn amor_waehlt (&mut self, target_1: &str, target_2: &str){
        if self.amor_hat_gewaehlt {
            println!("Amor hat bereits ein Liebespaar gewählt.");
            return;
        }
        if self.phase!= Phase::AmorPhase{
            println!("amor ist nicht dran.");
            return;
        }
        if target_1==target_2{
            println!("die liebenden dürfen nicht die selbe person sein");
            return;
        }

        let index1= match self.players.iter().position(|p| p.name==target_1){
            Some(i)=>i,
            None =>{
                println!("Spieler {} exisitiert nicht",target_1);
                return;
            }
        };
        let index2=match self.players.iter().position(|p| p.name==target_2){
            Some(i)=>i,
            None=>{
                println!("Der Spieler {} existiert nicht",target_2);
                return;
            }
        };
        
        if !self.players[index1].lebend||!self.players[index2].lebend{
            println!("Die liebenden müssen am leben sein");
            return;
        }

        { let player1 = &mut self.players[index1];
             player1.team = Team::TeamLiebende;
            }
        { let player2 = &mut self.players[index2];
            player2.team = Team::TeamLiebende;
         }

        self.liebender_1=Some (target_1.to_string());
        self.liebender_2=Some (target_2.to_string());
        self.liebende_aktiv=true;
        self.amor_hat_gewaehlt=true;
        self.phase_change();
        
        

        println!("Amor hat '{}' und '{}' zu Liebenden gemacht!", target_1, target_2); 
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

    fn spieler_stirbt(&mut self, verstorbener:&str){
        let player=self.players.iter_mut().find(|p| p.name == verstorbener);
        if player.is_none(){
            println!("Spieler {}existiert nicht", verstorbener);
            return;
        }
        let victim = player.unwrap();
        if !victim.lebend{
            println!("spieler bereits tot.");
            return;
        }

        victim.lebend=false;
        println!("Spieler {} ist gestorben.", verstorbener);

        if victim.rolle==Rolle::Jäger{
            println!("{} war der Jäger und schießt nun.", verstorbener);
            //brauche ziel vom frontend

            if let Some(ziel)= self.jaeger_ziel.clone(){
                println!("Der Jäger erschießt {}.",ziel);
                self.spieler_stirbt(&ziel);
            }else {
                println!("Der Jäger hat niemanden ausgewählt.");
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
                    println!("{} stirbt vor Kummer.", liebespartner_name);
                    self.spieler_stirbt(&liebespartner_name);
                }
        }
        }
        self.liebende_aktiv=false;
        
       if let Some(winner) = self.check_win() {
        println!("SPIEL BEENDET: {:?} gewinnt!", winner);
        } 

    }
}
