pub enum Phase{
    Tag,
    Nacht,
}
pub enum Rolle{
    Dorfbewohner,
    Werwolf,
    Seher,
}
pub struct Spieler{
    pub name: String,
    pub rolle: Rolle,
    pub lebend: bool,
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
}