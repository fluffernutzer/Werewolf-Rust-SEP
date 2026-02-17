//use std::io;
//use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Rolle {
    Dorfbewohner,
    Werwolf,
    Seher,
    Hexe,
    Jäger,
    Amor,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Team {
    TeamWerwolf,
    TeamDorf,
    TeamLiebende,
}

<<<<<<< Updated upstream
impl Rolle{
pub fn team(&self)->Team{
    match self{
        Rolle::Werwolf=>Team::TeamWerwolf,
        Rolle::Seher=>Team::TeamDorf,
        Rolle::Hexe=>Team::TeamDorf,
        Rolle::Jäger=>Team::TeamDorf,
        Rolle::Amor=>Team::TeamDorf,
        Rolle::Dorfbewohner=>Team::TeamDorf,
=======
impl Rolle {
    pub fn team(&self) -> Team {
        match self {
            Rolle::Werwolf => Team::TeamWerwolf,
            Rolle::Seher => Team::TeamDorf,
            Rolle::Hexe => Team::TeamDorf,
            Rolle::Jäger => Team::TeamDorf,
            Rolle::Amor => Team::TeamDorf,
            Rolle::Dorfbewohner => Team::TeamDorf,
            Rolle::Doktor => Team::TeamDorf,
        }
>>>>>>> Stashed changes
    }
}
