//!
//! Identify pass quality
//! HanishKVC, 2022
//!


const SCORE_BAD_PASS: f32 = -0.5;
const SCORE_GOOD_PASS: f32 = 1.0;
const SCORE_SELF_PASS: f32 = 0.1;

#[derive(Debug)]
struct Players {
    aplayers: Vec<(usize, f32)>,
    bplayers: Vec<(usize, f32)>,
}

impl Players {

    fn new(acnt: usize, bcnt: usize) -> Players {
        let mut     players = Players {
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

    fn score(&mut self, side: char, playerid: usize, score: f32) {
        if side == 'a' {
            self.aplayers[playerid].1 += score;
        } else {
            self.bplayers[playerid].1 += score;
        }
    }

}

#[derive(Debug)]
pub struct KickData {
    time: usize,
    side: char,
    playerid: usize,
    pos: (f32, f32),
}

impl KickData {

    fn new(time: usize, side: char, playerid: usize, pos: (f32,f32)) -> KickData {
        KickData {
            time: time,
            side: side,
            playerid: playerid,
            pos: pos,
        }
    }

}

#[derive(Debug)]
pub struct Passes {
    players: Players,
    kicks: Vec<KickData>,
}

impl Passes {

    pub fn new(acnt: usize, bcnt: usize) -> Passes {
        Passes {
            players: Players::new(acnt, bcnt),
            kicks: Vec::new(),
        }
    }

    pub fn add_kick(&mut self, kick: KickData) {
        let ik = self.kicks.len();
        if ik > 0 {
            let prev = &self.kicks[ik];
            if prev.side != kick.side {
                self.players.score(prev.side, prev.playerid, SCORE_BAD_PASS);
            } else {
                if prev.playerid == kick.playerid {
                    self.players.score(prev.side, prev.playerid, SCORE_SELF_PASS);
                } else {
                    self.players.score(prev.side, prev.playerid, SCORE_GOOD_PASS);
                }
            }
        }
        self.kicks.push(kick);
    }

}
