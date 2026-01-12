use crate::logic::{Game, Phase};
use crate::roles::Rolle;
use std::io;

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

/* pub fn advance_phase(game: &mut Game) {
    game.phase = match game.phase {
        Phase::Tag => Phase::Nacht,
        Phase::Nacht => {
            game.runden += 1;
            Phase::Tag
        }
    };
} */
