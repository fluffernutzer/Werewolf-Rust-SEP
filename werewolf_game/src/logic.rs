use std::io;
use std::fmt;
use rand::seq::SliceRandom;
use rand::thread_rng;
#[derive(Debug, Clone)]
pub enum Phase {
    Tag,
    Nacht,
}
#[derive(Debug,Clone)]
pub enum Rolle {
    Dorfbewohner,
    Werwolf,
    Seher,
}
#[derive(Debug, Clone)]
pub struct Spieler {
    pub name: String,
    pub rolle: Rolle,
    pub lebend: bool,
}
#[derive(Debug, Clone)]
pub struct Game {
    pub players: Vec<Spieler>,
    pub phase: Phase,
    pub runden: u32,
}

impl Spieler {
    pub fn new(name: String, rolle: Rolle) -> Self {
        Spieler {
            name,
            rolle,
            lebend: true,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: Vec::new(),
            phase: Phase::Tag,
            runden: 1,
        }
    }

    pub fn add_player(&mut self, name: String) {
        self.players.push(Spieler {
            name,
            rolle: Rolle::Werwolf,
            lebend: true,
        });
    }

    pub fn werwolf_toetet(&mut self, name: &str) {
    if let Some(p) = self.players.iter_mut().find(|p| p.name == name) {
        p.lebend = false;
        println!("(NACHT) {} wurde vom Werwolf getötet", name);
        }
    }

    pub fn dorf_toetet(&mut self, name: &str) {
        if let Some(p) = self.players.iter_mut().find(|p| p.name == name) {
            p.lebend = false;
            println!("(TAG) {} wurde getötet", name);
        }
    }
  
    pub fn rolle_von(&self, name: &str) -> Option<&Rolle> {
        self.players
            .iter()
            .find(|p| p.name == name)
            .map(|p| &p.rolle)
    }
  
    pub fn verteile_rollen(&mut self) {
        if self.players.is_empty() {
            return;
        }
        self.players[0].rolle = Rolle::Werwolf;

        if self.players.len() > 1 {
            self.players[1].rolle = Rolle::Seher;
        }
    }
  
    pub fn naechste_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Tag => Phase::Nacht,
            Phase::Nacht => {
                self.runden += 1;
                Phase::Tag
            }
        };
    }
 
    pub fn seher_schaut(&self, name: &str) -> &Rolle {
        println!("(NACHT) Seher überprüft {}", name);
        return self.rolle_von(name).unwrap();
    }
    
}