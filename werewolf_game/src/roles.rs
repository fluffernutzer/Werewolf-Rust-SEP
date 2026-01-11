//use std::io;
//use std::fmt;

#[derive(Debug,Clone, PartialEq, Eq, Copy)]
pub enum Rolle {
    Dorfbewohner,
    Werwolf,
    Seher,
    Hexe,
    Jäger,
    Amor,
}

#[derive(Debug,Clone)]
pub enum Team{
    TeamWerwolf,
    TeamDorf,
    TeamLiebende,
}

impl Rolle{
pub fn team(&self)->Team{
    match self{
        Rolle::Werwolf=>Team::TeamWerwolf,
        Rolle::Seher=>Team::TeamDorf,
        Rolle::Hexe=>Team::TeamDorf,
        Rolle::Jäger=>Team::TeamDorf,
        Rolle::Amor=>Team::TeamDorf,
        Rolle::Dorfbewohner=>Team::TeamDorf,
    }
}}
/*#[derive(Debug, Clone)]
pub enum Phase {
    Tag,
    Nacht,
    Amor_Phase,
    Werwölfe_Phase,
    Sehr_Phase,
    Hexe_Phase,
}

pub enum Team{
    Team_Werwolf,
    Team_Dorf,
    Team_Liebende,
}
#[derive(Debug,Clone)]
pub enum Rolle {
    Dorfbewohner,
    Werwolf,
    Seher,
    Hexe,
    Jäger,
    Amor,
}*/
/*#[derive(Debug, Clone)]
pub struct Spieler {
    pub name: String,
    pub team: Team,
    pub rolle: Rolle,
    pub lebend: bool,
}*/
/*#[derive(Debug, Clone)]
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
    pub bereits_gesehen:bool,
    pub last_seher_result:Option<(String,Rolle)>,
}
*/

/*impl Spieler {
    pub fn new(name: String, team:Team, rolle: Rolle, lebend: bool) -> Self {
        Spieler {
            name,
            team,
            rolle,
            lebend,
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
            bereits_gesehen:false,
            last_seher_result:None,
        }
    }


    pub fn add_player(&mut self, name: String) {
        self.players.push(Spieler {
            name,
            rolle: Rolle::Dorfbewohner,
            lebend: true,
        });
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
            Phase::Tag => Phase::Nacht,
            Phase::Nacht => {
                self.runden += 1;
                Phase::Tag
            }
        };
    }


    pub fn tag_lynchen(&mut self, name: &str) {
        println!("(TAG) Dorf lyncht {}", name);
    }

*/
