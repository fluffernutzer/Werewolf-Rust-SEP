use rand::seq::SliceRandom;
use rand::thread_rng;

fn verteile_rollen(players:&mut[Player])->Result<(),String>{
    let mut rng= thread_rng();

    let anzahl_spieler=players.len();
    if anzahl_spieler <3{
        return Err ("Es müssen mindestens 3 Spieler vorhanden sein.".to_string());
    }
    let anzahl_werwoelfe= anzahl_spieler/3;
    let anzahl_dorfbewohner= anzahl_spieler-anzahl_werwoelfe-1;

    let mut roles=Vec::new();
    for _ in 0..anzahl_werwoelfe{
        roles.push(Role::Werwolf);
    }
    for _ in 0..anzahl_dorfbewohner{
        roles.push(Role::Dorfbewohner);
    }
    roles.push(Role::Seher);

    roles.shuffle(&mut rng);
    for (player, role) in players.iter_mut().zip(roles.into_iter()) {
        player.role = role;
    }
    Ok(())
}