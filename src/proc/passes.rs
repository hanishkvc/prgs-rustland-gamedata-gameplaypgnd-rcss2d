//!
//! Identify pass quality
//! HanishKVC, 2022
//!

struct Players {
    aplayers: Vec<(usize, f32)>,
    bplayers: Vec<(usize, f32)>,
}

impl Players {

    fn new(acnt: usize, bcnt: usize) -> Players {
        let mut players = Players {
            aplayers: Vec::new(),
            bplayers: Vec::new(),
        };
        for i in 0..acnt {
            players.aplayers.push((i, 0.0));
        }
        for i in 0..bcnt {
            players.bplayers.push((i, 0.0));
        }
        return players;
    }

}

/// (time, (side, playerid), (nx, ny))
type KickData = (u32, (char, usize), (f32, f32));

pub struct Passes {
    players: Players,
    kicks: Vec<KickData>,
}