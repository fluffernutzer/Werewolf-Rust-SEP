use crate::logic::{Game, Rolle, Phase};
use std::io;

pub fn run_game_loop(game: &mut Game) {
    println!("=== Werwolf: Game Loop gestartet ===");

    loop {
        println!("\n=== Runde {} ===", game.runden);
        println!("Phase: {:?}", game.phase);

        match game.phase {
            Phase::Tag => {
                println!("TAG: Diskussion & Abstimmung");
                if check_win(game) {
                    break;
                }
                game.phase = Phase::Nacht;
            }
            Phase::Nacht => {
                println!("NACHT: Werwölfe & Seher");
                if check_win(game) {
                    break;
                }
                game.phase = Phase::Tag;
                game.runden += 1;
            }
        }
    }

    println!("=== Spiel beendet ===");
}

fn check_win(game: &Game) -> bool {
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
        println!("🎉 Dorf gewinnt!");
        return true;
    }

    if wolfs >= dorf {
        println!("🐺 Werwölfe gewinnen!");
        return true;
    }

    false
}
