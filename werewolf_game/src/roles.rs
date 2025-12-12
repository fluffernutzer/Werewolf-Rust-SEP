
use std::fmt;



#[derive(Debug)]
enum Team {
    Dorf,
    Werwölfe,
}
#[derive(Debug)]
enum Role{
    Dorfbewohner,
    Werwolf,
    Seher,
}

impl fmt::Display for Role{
    fn fmt (&self, f:&mut fmt ::Formatter)->fmt ::Result{
        match self{
            Role::Dorfbewohner=> write!(f,"Dorfbewohner"),
            Role::Seher=>write!(f,"Seher"),
            Role::Werwolf=>write!(f,"Werwolf"),
        }
    }
}
#[derive(Debug)]
struct Player{
    role:Role,
    alive:bool,
    awake:bool,
    team:Team,
}


impl Player{
    fn is_alive(&self)->bool{
        return self.alive;
    }
    fn gets_killed (&mut self){
        self.alive=false;
    }
    fn falls_asleep(&mut self){
        self.awake=false;
    }
    fn wakes_up(&mut self){
        self.awake=true;
    }
    fn get_team(&self)->&Team{
        return &self.team;
    }
    //special powers
    fn attack(&self, victim:&mut Player){
        match self.role{
            Role::Werwolf=>{
                match victim.role{
                    Role::Werwolf=>{println!("Du kannst nicht einen Werwolf töten!");}
                    _=> {
                    if victim.is_alive(){
                    victim.gets_killed();
                    println!("{} wurde von einem Werwolf getötet!", victim.role);
                    }}
                }
            }
            _ =>{
                println!("Nur Werwölfe können jemanden umbringen!");
            }
        }
    }
    fn see (&self, revealed_player:&Player){
        match self.role{
            Role::Seher=>{
                if revealed_player.is_alive(){
                    match revealed_player.team{
                        Team::Dorf=> println!("Die Person ist gut!"),
                        Team::Werwölfe=>println!("Die Person ist böse!"),
                    }
                }
            }
            _=>{
                println!("Nur der Seher darf sich die Karte eines Mitspielers ansehen!");
            }
        }
    }
}

