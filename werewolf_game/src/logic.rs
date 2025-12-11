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
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: Vec::new(),
        }
    }

    pub fn add_player(&mut self, name: String) {
        self.players.push(Spieler {
            name,
            rolle: Rolle::Dorfbewohner,
            lebend: true,
        });
    }
}