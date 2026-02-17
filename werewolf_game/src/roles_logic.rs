use crate::Game;
use crate::Phase;
use crate::logic::HexenAktion;
use crate::roles::Rolle;
use crate::roles::Team;
  
impl Game{
   pub fn werwolf_toetet(&mut self, actor_name:&str, victim_name: &str) ->Result<(),String>{
         if self.phase!=Phase::WerwölfePhase{
            log::error!("Die Werwölfe sind gerade nicht dran.");
            return Err("Die Werwölfe sind gerade nicht dran.".into());
        }
        let lebende_werwoelfe= self.players.iter().any(|p| p.rolle==Rolle::Werwolf&&p.lebend);
        if !lebende_werwoelfe{
            log::error!("Es gibt keine lebenden Werwölfe mehr");
            return Err("Es gibt keine lebenden Werwölfe mehr".into());
        }
        let werwolf=self
                .players
                .iter()
                .find(|p|p.name==actor_name&&p.rolle==Rolle::Werwolf)
                .ok_or("Du bist kein Werwolf.")?;

        if !werwolf.lebend{
            log::error!("Nur lebende Werwölfe dürfen wählen.");
            return Err("Nur lebende Werwölfe dürfen wählen.".into());
        }

        let target=self.players
                    .iter()
                    .find(|p| p.name==victim_name)
                    .ok_or("Zielperson existiert nicht.")?;

        if !target.lebend{
            log::error!("Der Spieler ist bereits tot.");
            return Err("Der Spieler ist bereits tot.".into());
        }
         if actor_name==victim_name{
            log::error!("Man kann sich nicht selbst wählen.");
            return Err("Man kann sich nicht selbst wählen.".into());
         }

        log::info!("(NACHT) Werwölfe greifen {} an", victim_name);
        //println!("(NACHT) Werwölfe greifen {} an", victim_name);
        self.nacht_opfer=Some(victim_name.to_string());
        //self.werwoelfe_done=true;
        self.phase_change();
        Ok(())
    }


    pub fn seher_schaut(&mut self, target_name: &str) -> Result<Rolle,String> {
        if self.phase!=Phase::SeherPhase{
            log::error!("Seher ist nicht dran.");
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
            log::error!("Opfer lebt nicht mehr");
            return Err("Opfer lebt nicht mehr".into());
        }
        let seher =&mut self.players[seher_index];

        if !seher.lebend{
            log::error!("Der Seher lebt nicht mehr und kann deswegen nicht mehr sehen.");
            return Err("Der Seher lebt nicht mehr und kann deswegen nicht mehr sehen.".into());
        }
        if seher.bereits_gesehen{
            log::error!("Der Seher hat bereits einmal diese Runde gesehen.");
            return Err("Der Seher hat bereits einmal diese Runde gesehen.".into());
        }
        
        log::info!("(NACHT) Seher überprüft {}", target_name);
        //println!("(NACHT) Seher überprüft {}", target_name);

       seher.bereits_gesehen=true;
        //self.seher_done=true;
        
        self.phase_change();
        Ok(target_rolle)
    }


    pub fn hexe_arbeitet(&mut self, aktion:HexenAktion, actor_name:&str, extra_target:&str)->Result<(),String>{
        let hexe=self.players
                        .iter()
                        .find(|p| p.rolle==Rolle::Hexe)
                        .ok_or("Es gibt keine Hexe im Spiel.")?;
                if !hexe.lebend{
                    log::error!("Du bist aus dem Spiel schon raus.");
                    return Err("Du bist aus dem Spiel schon raus.".into());
                }
                
        match aktion {
            HexenAktion::Heilen=> {
                if self.heiltrank_genutzt{
                    log::error!("Hexe hat ihren Heiltrank bereits einmal benutzt.");
                    return Err("Hexe hat ihren Heiltrank bereits einmal benutzt.".into());
                }
                
                let opfer_name=self.nacht_opfer.as_ref().ok_or("Es gibt kein Opfer.")?;

                self.heiltrank_genutzt=true;
                //self.hexe_done=true;
                self.geheilter_von_hexe=Some(opfer_name.to_string());
                self.phase_change();
                Ok(())
            }

            HexenAktion::Vergiften=>{
                 if self.bereits_getoetet{
                    log::error!("Die Hexe darf nur einmal im Spiel jemanden vergiften.");
                    return Err("Die Hexe darf nur einmal im Spiel jemanden vergiften.".into());
                }
               let zusaetzliches_opfer=self.players
                        .iter_mut()
                        .find(|p|p.name==extra_target)
                        .ok_or("Opfer konnte nicht gefunden werden")?;

                if !zusaetzliches_opfer.lebend{
                    log::error!("Das Opfer ist bereits tot.");
                    return Err("Das Opfer ist bereits tot.".into());
                }
                if extra_target==actor_name{
                    log::error!("Du kannst dich nicht selber vergiften.");
                    return Err("Du kannst dich nicht selber vergiften.".into());
                }
                self.hexe_opfer=Some(extra_target.to_string());
                self.bereits_getoetet=true;
                log::info!("Hexe tötet noch dazu: {}", extra_target);
                //println!("Hexe tötet noch dazu: {}", extra_target);
                self.phase_change();
                Ok(())
            }

            HexenAktion::NichtsTun=>{
                log::info!("Hexe tut nichts.");
                //println!("Hexe tut nichts.");
                self.phase_change();
                Ok(())
            }

        }
    }


    pub fn amor_waehlt (&mut self, target_1: &str, target_2: &str)->Result<(),String>{
        if self.amor_hat_gewaehlt {
            log::error!("Amor kann nur einmal wählen.");
            return Err("Amor kann nur einmal wählen.".into());
        }
        if self.phase!= Phase::AmorPhase{
            log::error!("Amor ist gerade nicht dran.");
            return Err("Amor ist gerade nicht dran.".into());
        }
        if target_1==target_2{
            log::error!("Die Liebenden müssen zwei verschiedene Personen sein.");
            return Err ("Die Liebenden müssen zwei verschiedene Personen sein.".into());
        }

        let index1= match self.players.iter().position(|p| p.name==target_1){
            Some(i)=>i,
            None =>{
                log::error!("Spieler {} exisitiert nicht",target_1);
                //println!("Spieler {} exisitiert nicht",target_1);
                return Err("Spieler existiert nicht".into());
            }
        };
        let index2=match self.players.iter().position(|p| p.name==target_2){
            Some(i)=>i,
            None=>{
                log::error!("Der Spieler {} existiert nicht",target_2);
                //println!("Der Spieler {} existiert nicht",target_2);
                return Err("Spieler existiert nicht.".into());
            }
        };
        
        if !self.players[index1].lebend||!self.players[index2].lebend{
            log::error!("Beide Liebenden müssen am Leben sein.");
            return Err("Beide Liebenden müssen am Leben sein.".into());
        }

         self.players[index1].team=Team::TeamLiebende;
         self.players[index2].team=Team::TeamLiebende;

        self.liebender_1=Some (target_1.to_string());
        self.liebender_2=Some (target_2.to_string());
        self.liebende_aktiv=true;
        self.amor_hat_gewaehlt=true;
        self.phase_change();
        
        log::info!("Amor hat '{}' und '{}' zu Liebenden gemacht!", target_1, target_2);
        //println!("Amor hat '{}' und '{}' zu Liebenden gemacht!", target_1, target_2); 
        Ok(())
    }


    pub fn doktor_schuetzt(&mut self, geschuetzter_name:&str)->Result<(),String>{
        let doktor=self.players
        .iter()
        .find(|p| p.rolle==Rolle::Doktor)
        .ok_or("Es gibt keinen Doktor (mehr) im Spiel.")?;

        if !doktor.lebend{
            log::error!("Der Doktor ist nicht mehr am Leben.");
            return Err("Der Doktor ist nicht mehr am Leben.".into());
        }

        let geschuetzter=self.players
                                        .iter()
                                        .find(|p|p.name==geschuetzter_name)
                                        .ok_or("Die Person existiert nicht.")?;
        if !geschuetzter.lebend{
            log::error!("Der Schützling muss am Leben sein");
            return Err("Der Schützling muss am Leben sein".into());
        }

        self.geschuetzter_von_doktor=Some(geschuetzter_name.to_string());
        self.phase_change(); //Nötig um im Spiel voran zukommen? 
        Ok(())
    }

    pub fn priester_wirft(&mut self, actor_name: &str, target_name: Option<&str>) -> Result<(), String> {
        
        if self.phase != Phase::PriesterPhase{
            log::error!("Der Priester ist gerade nicht dran.");
            return Err("Der Priester ist gerade nicht dran.".into());
        }

        let priester_index = self.players
            .iter()
            .position(|p| p.rolle == Rolle::Priester && p.lebend)
            .ok_or("Es gibt keinen lebenden Priester.".to_string())?;

        if self.priester_hat_geworfen{
            log::error!("Der Priester hat bereits heiliges Wasser geworfen.");
            return Err("Der Priester hat bereits heiliges Wasser geworfen.".into());
        }

        if let Some(ziel) = target_name {
            let ziel_index = self.players
                .iter()
                .position(|p| p.name == ziel)
                .ok_or("Ziel existiert nicht.".to_string())?;
            
            if !self.players[ziel_index].lebend{
                log::error!("Das Ziel ist bereits tot.");
                return Err("Das Ziel ist bereits tot.".into());
            }
            if self.players[priester_index].name == ziel{
                log::error!("Der Priester kann sich nicht selbst mit heiligen Wasser bewerfen.");
                return Err("Der Priester kann sich nicht selbst mit heiligen Wasser bewerfen.".into());
            }

            let ziel_team = self.players[ziel_index].team.clone();

            if ziel_team == Team::TeamWerwolf{
                log::info!("(NACHT) Priester wirft heiliges Wasser auf {} ! Es ist ein Werwolf!", ziel);
                //println!("(NACHT) Priester wirft heiliges Wasser auf {} ! Es ist ein Werwolf!", ziel);
                self.spieler_stirbt(ziel);
            } else {
                log::info!("(NACHT) Priester wirft heiliges Wasser auf {} ! Es ist leider kein Werwolf; der Priester stirbt.", ziel);
                //println!("(NACHT) Priester wirft heiliges Wasser auf {} ! Es ist leider kein Werwolf; der Priester stirbt.", ziel);
                self.spieler_stirbt(actor_name);
            }
        } else {
            log::info!("(NACHT) Priester tut nichts.");
            println!("(NACHT) Priester tut nichts.");
        }
        
        self.priester_hat_geworfen = true;

        self.phase_change();
        Ok(())
     
    }
}   
