//!
//! Identify pass quality
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, render::BlendMode};

use crate::sdlx::SdlX;


const SCORE_BAD_PASS: f32 = -0.5;
const SCORE_GOOD_PASS: f32 = 1.0;
const SCORE_SELF_PASS: f32 = 0.05;

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

    /// Help update the score of a specific player
    fn score(&mut self, side: char, playerid: usize, score: f32) {
        if side == 'a' {
            self.aplayers[playerid].1 += score;
        } else {
            self.bplayers[playerid].1 += score;
        }
    }

    /// Return the max player score for each of the teams
    fn score_max(&self) -> (f32, f32) {
        let mut amax = f32::MIN;
        for i in 0..self.aplayers.len() {
            let player = self.aplayers[i];
            if amax < player.1 {
                amax = player.1;
            }
        }
        let mut bmax = f32::MIN;
        for i in 0..self.bplayers.len() {
            let player = self.bplayers[i];
            if bmax < player.1 {
                bmax = player.1;
            }
        }
        (amax, bmax)
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

    pub fn new(time: usize, side: char, playerid: usize, pos: (f32,f32)) -> KickData {
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

    /// Add a kick data and inturn adjust the scores
    pub fn add_kick(&mut self, kick: KickData) {
        let ik = self.kicks.len();
        if ik > 0 {
            let prev = &self.kicks[ik-1];
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

    fn summary_simple(&self) {
        for i in 0..self.players.aplayers.len() {
            let player = self.players.aplayers[i];
            eprintln!("DBUG:PPGND:Proc:Passes:A:{:02}:{}", player.0, player.1);
        }
        for i in 0..self.players.bplayers.len() {
            let player = self.players.bplayers[i];
            eprintln!("DBUG:PPGND:Proc:Passes:B:{:02}:{}", player.0, player.1);
        }
    }

    fn summary_asciiart(&self) {
        for i in 0..self.players.aplayers.len() {
            let player = self.players.aplayers[i];
            eprint!("DBUG:PPGND:Proc:Passes:A:{:02}:", player.0);
            for _j in 0..player.1.round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for i in 0..self.players.bplayers.len() {
            let player = self.players.bplayers[i];
            eprint!("DBUG:PPGND:Proc:Passes:B:{:02}:", player.0);
            for _j in 0..player.1.round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
    }

    /// Graphics Summary (a relative graph)
    /// Take the max score across players wrt each team and
    /// plot score bar relative to that max score.
    pub fn summary_sdl(&self, sx: &mut SdlX) {
        // let (amax, bmax) = (20.0, 20.0);
        let (amax, bmax) = self.players.score_max();
        for i in 0..self.players.aplayers.len() {
            let player = self.players.aplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            sx.nn_fill_rect(0.05, 0.05*(i as f32 + 4.0), 0.4*(player.1/amax), 0.04)
        }
        for i in 0..self.players.bplayers.len() {
            let player = self.players.bplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            sx.nn_fill_rect(0.55, 0.05*(i as f32 + 4.0), 0.4*(player.1/bmax), 0.04)
        }
    }

    pub fn summary(&self) {
        self.summary_asciiart();
    }

}
