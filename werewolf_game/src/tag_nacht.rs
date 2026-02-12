use crate::logic::{Game, Phase};
use crate::roles::Rolle;
use std::io;



impl Game{

  pub fn has_role(&self, rolle: Rolle)->bool{
        self.players.iter().any(|p|p.rolle==rolle&&p.lebend)
    }

    pub fn phase_change(&mut self){
        if let Phase::Tag=self.phase{
            if self.has_role(Rolle::Amor){
                self.phase=Phase::AmorPhase;
                return;
            } else if self.has_role(Rolle::Werwolf){
                self.phase=Phase::WerwölfePhase;
                return;
            } else if self.has_role(Rolle::Seher){
                self.phase=Phase::SeherPhase;
                return;
            } else if self.has_role(Rolle::Hexe){
                self.phase=Phase::HexePhase;
                return;
            } else if self.has_role(Rolle::Doktor) {
                self.phase=Phase::DoktorPhase;
                return;
            }else {
                self.phase=Phase::Tag;
                return;
            }
        }
        if let Phase::AmorPhase=self.phase{
            if self.has_role(Rolle::Werwolf){
                self.phase=Phase::WerwölfePhase;
                return;
            } else if self.has_role(Rolle::Seher){
                self.phase=Phase::SeherPhase;
                return;
            } else if self.has_role(Rolle::Hexe){
                self.phase=Phase::HexePhase;
                return;
            } else if self.has_role(Rolle::Doktor) {
                self.phase=Phase::DoktorPhase;
                return;
            } else {
                self.phase=Phase::Tag;
                return;
            }
        }
        if let Phase::WerwölfePhase=self.phase{
             if self.has_role(Rolle::Seher){
                self.phase=Phase::SeherPhase;
                return;
            } else if self.has_role(Rolle::Hexe){
                self.phase=Phase::HexePhase;
                return;
            } else if self.has_role(Rolle::Doktor) {
                self.phase=Phase::DoktorPhase;
                return;
            } else {
                self.phase=Phase::Tag;
                return;
            }
        }
        if let Phase::SeherPhase=self.phase{
            if self.has_role(Rolle::Hexe){
                self.phase=Phase::HexePhase;
                return;
            }else if self.has_role(Rolle::Doktor) {
                self.phase=Phase::DoktorPhase;
                return;
            } else {
                self.phase=Phase::Tag;
                return;
            }
        }
        if let Phase::HexePhase=self.phase{
            if self.has_role(Rolle::Doktor){
                self.phase=Phase::DoktorPhase;
                return;
            } else {
                self.phase=Phase::Tag;
                return;
            }
        }
        if let Phase::DoktorPhase=self.phase{
            self.nacht_aufloesung();
            self.phase=Phase::Tag;
            self.runden += 1;
            return;
        }
        }
    }

 
/* pub fn check_win(game: &Game) -> Option<String> {
    let mut dorf = 0;
    let mut wolfs = 0;

    for p in &game.players {
        if p.lebend {
            match p.rolle {
                Rolle::Werwolf => wolfs += 1,
                _ => dorf += 1,
            }
        }
    }

    if wolfs == 0 {
        return Some("Dorf gewinnt".to_string());
    }

    if wolfs >= dorf {
        return Some("Werwölfe gewinnen".to_string());
    }

    None
} */


