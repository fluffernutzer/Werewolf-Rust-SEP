use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::roles::Rolle;
use crate::roles::Team;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Tag,
    Nacht,
    AmorPhase,
    WerwölfePhase,
    SeherPhase,
    HexePhase,
}

#[derive(Debug, Clone)]
pub struct Spieler {
    pub name: String,
    pub team: Team,
    pub rolle: Rolle,
    pub lebend: bool,
    pub bereits_gesehen:bool,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub players: Vec<Spieler>,
    pub phase: Phase,
    pub runden: u32,
    pub heiltrank_genutzt:bool,
    pub bereits_getoetet: bool,
    pub nacht_opfer: Option<String>,
    pub liebender_1:Option<String>,
    pub liebender_2:Option<String>,
    pub liebende_aktiv:bool,
    pub amor_hat_gewaehlt:bool,
    pub jaeger_ziel:Option<String>,
    pub last_seher_result:Option<(String,Rolle)>,
}

impl Spieler {
    pub fn new(name: String, team: Team, rolle: Rolle, lebend:bool) -> Self {
        Spieler {
            name,
            team: rolle.team(),
            rolle,
            lebend:true,
            bereits_gesehen:false,
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
            nacht_opfer:None,
            liebender_1:None,
            liebender_2:None,
            liebende_aktiv:false,
            amor_hat_gewaehlt:false,
            jaeger_ziel:None,
            last_seher_result:None,
        }
    }

    pub fn add_player(&mut self, name: String) {
        self.players.push(Spieler {
            name,
            team: Team::TeamDorf, //Platzhalter wird noch durchgemischt
            rolle: Rolle::Dorfbewohner, //Platzhalter wird noch durchgemischt
            lebend: true,
            bereits_gesehen:false,
        });
    }

    pub fn verteile_rollen(&mut self)->Result<(),String>{
        let mut rng= thread_rng();

        let anzahl_spieler=self.players.len();
        if anzahl_spieler <3{
            return Err ("Es müssen mindestens 3 Spieler vorhanden sein.".to_string());
        }
        let anzahl_werwoelfe= anzahl_spieler/3;
        let anzahl_dorfbewohner= anzahl_spieler-anzahl_werwoelfe-1;

        let mut roles=Vec::new();

        for _ in 0..anzahl_werwoelfe{
            roles.push(Rolle::Werwolf);
        }
        for _ in 0..anzahl_dorfbewohner{
            roles.push(Rolle::Dorfbewohner);
        }
        roles.push(Rolle::Seher);

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
  /*  pub fn verteile_rollen(&mut self) {
        if self.players.is_empty() {
            return;
        }
        self.players[0].rolle = Rolle::Werwolf;

        if self.players.len() > 1 {
            self.players[1].rolle = Rolle::Seher;
        }
    }*/


    pub fn naechste_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Tag => 
            if self.runden==1{
                Phase::AmorPhase
            } else {
                Phase::WerwölfePhase
            }
            Phase::AmorPhase=>Phase::WerwölfePhase,
            Phase::WerwölfePhase=> Phase::SeherPhase,
            Phase::SeherPhase=>Phase::HexePhase,
            Phase::HexePhase=>{
            //Phase::Nacht => {
                self.runden += 1;
                Phase::Tag
            }
            Phase::Nacht=>Phase::WerwölfePhase,
        };
    }


    pub fn tag_lynchen(&mut self, name: &str) {
        println!("(TAG) Dorf lyncht {}", name);
    }

    pub fn werwolf_toetet(&mut self, victim_name: &str) {
         if self.phase!=Phase::WerwölfePhase{
            println!("es ist gerade keine werwolf phase");
            return;
        }
        let lebende_werwoelfe= self.players.iter().any(|p| p.rolle==Rolle::Werwolf&&p.lebend);
        if !lebende_werwoelfe{
            println!("es gibt keine werwölfe zum wählen.");
            return;
        }
        let target_opt=self.players.iter().find(|p| p.name==victim_name);
        let target= match target_opt{
            Some (p)=>p,
            None=>{
                println!("spieler {} existiert nciht.",victim_name);
                return;
            }
        };
        if !target.lebend{
            println!("das opfer {} ist schon tot.", victim_name);
            return;
        }
        println!("(NACHT) Werwölfe greifen {} an", victim_name);
        self.spieler_stirbt(&victim_name);
        self.naechste_phase();
        println!("(NACHT) Werwolf tötet {}", victim_name);
    }

    pub fn seher_schaut(&mut self, target_name: &str) -> Option<Rolle> {
        if self.phase!=Phase::SeherPhase{
            println!("Der Seher ist nicht dran.");
            return None;
        }
        let seher_index=self.players
        .iter()
        .position(|p| p.rolle==Rolle::Seher)?;

        let target_index=self.players
        .iter()
        .position(|p|p.name==target_name)?;

        let target_rolle=self.players[target_index].rolle;
        let target_lebend=self.players[target_index].lebend;
        if !target_lebend{
            println!("Die Person, die der Seher sehen will muss leben");
            return None;
        }
        let seher =&mut self.players[seher_index];

        if !seher.lebend{
            println!("Seher ist nicht mehr am Leben");
            return None;
        }
        if seher.bereits_gesehen{
            println!("der seher hat sich bereits eine Karte in dieser Runde angesehen");
            return None;
        }
        
        println!("(NACHT) Seher überprüft {}", target_name);

        seher.bereits_gesehen=true;

        self.naechste_phase();
        Some(target_rolle)
    }
    
    //Hexe darf nur einmal heilen und nur einmal töten
    pub fn hexe_heilt(&mut self){
        if self.heiltrank_genutzt{
            println!("Hexe hat ihren Heiltrank bereits einmal benutzt.");
            return;
        }
        let Some (opfer_name)=&self.nacht_opfer else {
            println!("es gibt kein Opfer in der Nacht zum heilen");
            return;
        };
        if let Some(player) = self.players.iter_mut().find(|p| p.name == *opfer_name){
            player.lebend=true;
            self.heiltrank_genutzt=true;
            println!("(Nacht) Hexe heilt {}", opfer_name);
        }else {
            println!("Spieler nicht gefunden");
        }
        self.naechste_phase();
    }

    pub fn hexe_toetet(&mut self, extra_target:&str){
        if self.bereits_getoetet{
            println!("Die Hexe hat bereits einmal jemanden getötet.");
            return;
        }
        if let Some(player)= self.players.iter_mut().find(|p| p.name==extra_target){
            player.lebend=false;
            self.bereits_getoetet=true;
            println!("Hexe tötet noch dazu: {}", extra_target);
        } else {
            println!("Spieler nicht gefunden.");
        }
        self.naechste_phase();
    }

    pub fn hexe_tut_nichts(&mut self){
        println!("Hexe tut nichts.");
        self.naechste_phase();
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
        
        

        println!("Amor hat '{}' und '{}' zu Liebenden gemacht!", target_1, target_2);
        
        self.phase=Phase::WerwölfePhase;
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
    }

}
