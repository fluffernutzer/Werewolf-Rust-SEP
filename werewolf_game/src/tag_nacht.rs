use std::io;

#[derive(Debug, Clone)]
pub enum Phase {
    Tag,
    Nacht,
}

#[derive(Debug, Clone)]
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

impl Spieler {
    pub fn new(name: String, rolle: Rolle) -> Self {
        Spieler {
            name,
            rolle,
            lebend: true,
        }
    }
}

pub struct Game {
    pub players: Vec<Spieler>,
    pub phase: Phase,
    pub runden: u32,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: Vec::new(),
            phase: Phase::Tag,
            runden: 1,
        }
    }

    pub fn add_player(&mut self, name: String, rolle: Rolle) {
        self.players.push(Spieler::new(name, rolle));
    }

    pub fn status_anzeige(&self) {
        println!("\n=== Runde {} ===", self.runden);
        println!("Phase: {:?}", self.phase);
        
        let mut dorf_count = 0;
        let mut wolf_count = 0;
        
        for player in &self.players {
            if player.lebend {
                match player.rolle {
                    Rolle::Werwolf => wolf_count += 1,
                    _ => dorf_count += 1,
                }
            }
        }
        
        println!("Dorfbewohner: {}", dorf_count);
        println!("Werwölfe: {}", wolf_count);
    }

    pub fn zeige_lebende_spieler(&self) {
        println!("\nLebende Spieler:");
        for player in &self.players {
            if player.lebend {
                println!("- {}", player.name);
            }
        }
    }

    pub fn finde_und_toete_spieler(&mut self) {
        // Zeige lebende Spieler
        self.zeige_lebende_spieler();
        
        // Frage nach Namen
        println!("\nWelcher Spieler soll ausscheiden? (Name eingeben):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Fehler beim Lesen");
        let name = input.trim();
        
        // Suche und töte Spieler
        let mut gefunden = false;
        for player in &mut self.players {
            if player.lebend && player.name == name {
                player.lebend = false;
                println!("{} wurde ausgewählt und ist jetzt tot!", name);
                println!("Rolle: {:?}", player.rolle);
                gefunden = true;
                break;
            }
        }
        
        if !gefunden {
            println!("Spieler nicht gefunden oder bereits tot. Versuche es nochmal.");
            self.finde_und_toete_spieler(); // Rekursion, bis gültiger Name
        }
    }

    pub fn nacht_aktionen(&mut self) {
        println!("\n=== NACHT ===");
        
        // Werwölfe wachen auf
        println!("Werwölfe wachen auf...");
        let mut werwoelfe = Vec::new();
        for player in &self.players {
            if player.lebend {
                if let Rolle::Werwolf = player.rolle {
                    werwoelfe.push(player.name.clone());
                }
            }
        }
        
        if !werwoelfe.is_empty() {
            println!("Werwölfe: {:?}", werwoelfe);
            println!("Die Werwölfe wählen ein Opfer...");
            self.finde_und_toete_spieler();
        } else {
            println!("Keine Werwölfe mehr am Leben.");
        }
        
        // Seherin wacht auf
        println!("\nSeherin wacht auf...");
        for player in &self.players {
            if player.lebend {
                if let Rolle::Seher = player.rolle {
                    println!("Seherin {} ist wach.", player.name);
                    
                    // Seherin wählt Spieler zum Überprüfen
                    self.zeige_lebende_spieler();
                    println!("Welchen Spieler möchtest du überprüfen? (Name eingeben):");
                    
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).expect("Fehler beim Lesen");
                    let name = input.trim();
                    
                    for target in &self.players {
                        if target.lebend && target.name == name {
                            println!("{} ist ein {:?}", name, target.rolle);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn tag_aktionen(&mut self) {
        println!("\n=== TAG ===");
        println!("Alle Dorfbewohner sind wach.");
        
        // Dorfbewohner diskutieren und wählen
        println!("Die Dorfbewohner diskutieren und müssen jemanden hinrichten...");
        self.finde_und_toete_spieler();
    }

    pub fn spiel_ende_pruefen(&self) -> bool {
        let mut dorf_count = 0;
        let mut wolf_count = 0;
        
        for player in &self.players {
            if player.lebend {
                match player.rolle {
                    Rolle::Werwolf => wolf_count += 1,
                    _ => dorf_count += 1,
                }
            }
        }
        
        if wolf_count == 0 {
            println!("\n🎉 DIE DORFBEWOHNER HABEN GEWONNEN!");
            return true;
        }
        
        if wolf_count >= dorf_count {
            println!("\n🐺 DIE WERWÖLFE HABEN GEWONNEN!");
            return true;
        }
        
        false
    }

    pub fn starte_spiel(&mut self) {
        println!("=== WERWOLF-SPIEL STARTET ===");
        
        // Initiale Verteilung anzeigen
        println!("\nSpieler und ihre Rollen:");
        for player in &self.players {
            println!("- {}: {:?}", player.name, player.rolle);
        }
        
        // Hauptspielschleife
        loop {
            self.status_anzeige();
            
            match self.phase {
                Phase::Tag => {
                    self.tag_aktionen();
                    if self.spiel_ende_pruefen() { break; }
                    println!("\n--- Nacht beginnt ---");
                    self.phase = Phase::Nacht;
                }
                Phase::Nacht => {
                    self.nacht_aktionen();
                    if self.spiel_ende_pruefen() { break; }
                    println!("\n--- Tag beginnt ---");
                    self.phase = Phase::Tag;
                    self.runden += 1;
                }
            }
            
            println!("\nDrücke Enter für nächste Phase...");
            let mut _pause = String::new();
            io::stdin().read_line(&mut _pause).expect("Fehler");
        }
    }
}

fn main() {
    let mut game = Game::new();
    
    // Einfache Spielerinitialisierung
    game.add_player("Martin".to_string(), Rolle::Dorfbewohner);
    game.add_player("Elif".to_string(), Rolle::Werwolf);
    game.add_player("Laura".to_string(), Rolle::Seher);
    game.add_player("Lisanne".to_string(), Rolle::Dorfbewohner);
    
    game.starte_spiel();
}
