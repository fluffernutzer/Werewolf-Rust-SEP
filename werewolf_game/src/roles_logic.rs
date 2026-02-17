use crate::Game;
use crate::Phase;
use crate::logic::HexenAktion;
use crate::roles::Rolle;
use crate::roles::Team;

impl Game {
    pub fn werwolf_toetet(&mut self, actor_name: &str, victim_name: &str) -> Result<(), String> {
        if self.phase != Phase::WerwölfePhase {
            return Err("Die Werwölfe sind gerade nicht dran.".into());
        }
        let lebende_werwoelfe = self
            .players
            .iter()
            .any(|p| p.rolle == Rolle::Werwolf && p.lebend);
        if !lebende_werwoelfe {
            return Err("Es gibt keine lebenden Werwölfe mehr".into());
        }
        let werwolf = self
            .players
            .iter()
            .find(|p| p.name == actor_name && p.rolle == Rolle::Werwolf)
            .ok_or("Du bist kein Werwolf.")?;

        if !werwolf.lebend {
            return Err("Nur lebende Werwölfe dürfen wählen.".into());
        }

        let target = self
            .players
            .iter()
            .find(|p| p.name == victim_name)
            .ok_or("Zielperson existiert nicht.")?;

        if !target.lebend {
            return Err("Der Spieler ist bereits tot.".into());
        }
        if actor_name == victim_name {
            return Err("Man kann sich nicht selbst wählen.".into());
        }

        println!("(NACHT) Werwölfe greifen {} an", victim_name);
        self.nacht_opfer = Some(victim_name.to_string());
        //self.werwoelfe_done=true;
        self.phase_change();
        Ok(())
    }

    pub fn seher_schaut(&mut self, target_name: &str) -> Result<Rolle, String> {
        if self.phase != Phase::SeherPhase {
            return Err("Seher ist nicht dran.".into());
        }
        let seher_index = self
            .players
            .iter()
            .position(|p| p.rolle == Rolle::Seher)
            .ok_or("Kein Seher im Spiel")?;

        let target_index = self
            .players
            .iter()
            .position(|p| p.name == target_name)
            .ok_or("Ziel existiert nicht.")?;

        let target_rolle = self.players[target_index].rolle;
        let target_lebend = self.players[target_index].lebend;
        if !target_lebend {
            return Err("Opfer lebt nicht mehr".into());
        }
        let seher = &mut self.players[seher_index];

        if !seher.lebend {
            return Err("Der Seher lebt nicht mehr und kann deswegen nicht mehr sehen.".into());
        }
        if seher.bereits_gesehen {
            return Err("Der Seher hat bereits einmal diese Runde gesehen.".into());
        }

        println!("(NACHT) Seher überprüft {}", target_name);

        seher.bereits_gesehen = true;
        //self.seher_done=true;

        self.phase_change();
        Ok(target_rolle)
    }

    pub fn hexe_arbeitet(
        &mut self,
        aktion: HexenAktion,
        actor_name: &str,
        extra_target: &str,
    ) -> Result<(), String> {
        let hexe = self
            .players
            .iter()
            .find(|p| p.rolle == Rolle::Hexe)
            .ok_or("Es gibt keine Hexe im Spiel.")?;
        if !hexe.lebend {
            return Err("Du bist aus dem Spiel schon raus.".into());
        }

        match aktion {
            HexenAktion::Heilen => {
                if self.heiltrank_genutzt {
                    return Err("Hexe hat ihren Heiltrank bereits einmal benutzt.".into());
                }

                let opfer_name = self.nacht_opfer.as_ref().ok_or("Es gibt kein Opfer.")?;

                self.heiltrank_genutzt = true;
                //self.hexe_done=true;
                self.geheilter_von_hexe = Some(opfer_name.to_string());
                self.phase_change();
                Ok(())
            }

            HexenAktion::Vergiften => {
                if self.bereits_getoetet {
                    return Err("Die Hexe darf nur einmal im Spiel jemanden vergiften.".into());
                }
                let zusaetzliches_opfer = self
                    .players
                    .iter_mut()
                    .find(|p| p.name == extra_target)
                    .ok_or("Opfer konnte nicht gefunden werden")?;

                if !zusaetzliches_opfer.lebend {
                    return Err("Das Opfer ist bereits tot.".into());
                }
                if extra_target == actor_name {
                    return Err("Du kannst dich nicht selber vergiften.".into());
                }
                self.hexe_opfer = Some(extra_target.to_string());
                self.bereits_getoetet = true;
                println!("Hexe tötet noch dazu: {}", extra_target);
                self.phase_change();
                Ok(())
            }

            HexenAktion::NichtsTun => {
                println!("Hexe tut nichts.");
                self.phase_change();
                Ok(())
            }
        }
    }

    pub fn amor_waehlt(&mut self, target_1: &str, target_2: &str) -> Result<(), String> {
        if self.amor_hat_gewaehlt {
            return Err("Amor kann nur einmal wählen.".into());
        }
        if self.phase != Phase::AmorPhase {
            return Err("Amor ist gerade nicht dran.".into());
        }
        if target_1 == target_2 {
            return Err("Die Liebenden müssen zwei verschiedene Personen sein.".into());
        }

        let index1 = match self.players.iter().position(|p| p.name == target_1) {
            Some(i) => i,
            None => {
                println!("Spieler {} exisitiert nicht", target_1);
                return Err("Spieler existiert nicht".into());
            }
        };
        let index2 = match self.players.iter().position(|p| p.name == target_2) {
            Some(i) => i,
            None => {
                println!("Der Spieler {} existiert nicht", target_2);
                return Err("Spieler existiert nicht.".into());
            }
        };

        if !self.players[index1].lebend || !self.players[index2].lebend {
            return Err("Beide Liebenden müssen am Leben sein.".into());
        }

        self.players[index1].team = Team::TeamLiebende;
        self.players[index2].team = Team::TeamLiebende;

        self.liebender_1 = Some(target_1.to_string());
        self.liebender_2 = Some(target_2.to_string());
        self.liebende_aktiv = true;
        self.amor_hat_gewaehlt = true;
        self.phase_change();

        println!(
            "Amor hat '{}' und '{}' zu Liebenden gemacht!",
            target_1, target_2
        );
        Ok(())
    }

    pub fn doktor_schuetzt(&mut self, geschuetzter_name: &str) -> Result<(), String> {
        let doktor = self
            .players
            .iter()
            .find(|p| p.rolle == Rolle::Doktor)
            .ok_or("Es gibt keinen Doktor (mehr) im Spiel.")?;

        if !doktor.lebend {
            return Err("Der Doktor ist nicht mehr am Leben.".into());
        }

        let geschuetzter = self
            .players
            .iter()
            .find(|p| p.name == geschuetzter_name)
            .ok_or("Die Person existiert nicht.")?;
        if !geschuetzter.lebend {
            return Err("Der Schützling muss am Leben sein".into());
        }

        self.geschuetzter_von_doktor = Some(geschuetzter_name.to_string());
        Ok(())
    }
}
