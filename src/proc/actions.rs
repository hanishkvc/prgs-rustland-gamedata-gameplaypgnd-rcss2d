//!
//! Identify pass quality
//! HanishKVC, 2022
//!
//! TODO:
//! * Track for goals/halftime/etal and avoid providing -ve scoring
//!   due to any of these shifting side that will kick
//! * Add support for providing scores for tackle/catch/etal
//!

use sdl2::{pixels::Color, render::BlendMode};

use crate::sdlx::SdlX;
use crate::playdata::Action;


const SELF_PASS_MINTIME: isize = 10;
const SCORE_BAD_PASS: f32 = -0.5;
const SCORE_HIJACK_PASS: f32 = -SCORE_BAD_PASS;
const SCORE_GOOD_PASS_SENT: f32 = 1.0;
const SCORE_GOOD_PASS_GOT: f32 = 0.4;
const SCORE_SELF_PASS: f32 = 0.05;
const SCORE_TACKLE: f32 = 0.5;
const SCORE_CATCH: f32 = 1.0;


#[derive(Debug)]
struct Score {
    score: f32,
    kicks: usize,
    tackles: usize,
    catchs: usize,
}

impl Score {

    fn new(score: f32, kicks: usize, tackles: usize, catchs: usize) -> Score {
        Score {
            score: score,
            kicks: kicks,
            tackles: tackles,
            catchs: catchs,
        }
    }

    fn default() -> Score {
        return Score::new(0.0, 0, 0, 0);
    }

}

#[derive(Debug)]
struct Players {
    aplayers: Vec<(usize, Score)>,
    bplayers: Vec<(usize, Score)>,
}

impl Players {

    fn new(acnt: usize, bcnt: usize) -> Players {
        let mut players = Players {
            aplayers: Vec::new(),
            bplayers: Vec::new(),
        };
        for i in 0..acnt {
            players.aplayers.push((i, Score::default()));
        }
        for i in 0..bcnt {
            players.bplayers.push((i, Score::default()));
        }
        return players;
    }

    /// Help update the score of a specific player
    fn score(&mut self, side: char, playerid: usize, score: f32) {
        if side == 'a' {
            self.aplayers[playerid].1.score += score;
        } else {
            self.bplayers[playerid].1.score += score;
        }
    }

    /// Help update the score of a specific player
    fn count_increment(&mut self, side: char, playerid: usize, atype: Action) {
        let player;
        if side == 'a' {
            player = &mut self.aplayers[playerid];
        } else {
            player = &mut self.bplayers[playerid];
        }
        match atype {
            Action::None => todo!(),
            Action::Kick(_) => player.1.kicks += 1,
            Action::Catch(_) => player.1.catchs += 1,
            Action::Tackle(_) => player.1.tackles += 1,
        }
    }

    /// Return the max player score for each of the teams
    fn score_max(&self) -> (f32, f32) {
        let mut amax = f32::MIN;
        for i in 0..self.aplayers.len() {
            let player = &self.aplayers[i];
            if amax < player.1.score {
                amax = player.1.score;
            }
        }
        let mut bmax = f32::MIN;
        for i in 0..self.bplayers.len() {
            let player = &self.bplayers[i];
            if bmax < player.1.score {
                bmax = player.1.score;
            }
        }
        (amax, bmax)
    }

}


#[derive(Debug)]
pub struct ActionData {
    time: usize,
    side: char,
    playerid: usize,
    #[allow(dead_code)]
    pos: (f32, f32),
}

impl ActionData {

    pub fn new(time: usize, side: char, playerid: usize, pos: (f32,f32)) -> ActionData {
        ActionData {
            time: time,
            side: side,
            playerid: playerid,
            pos: pos,
        }
    }

}

#[derive(Debug)]
pub struct ActionsInfo {
    players: Players,
    kicks: Vec<ActionData>,
}

impl ActionsInfo {

    pub fn new(acnt: usize, bcnt: usize) -> ActionsInfo {
        ActionsInfo {
            players: Players::new(acnt, bcnt),
            kicks: Vec::new(),
        }
    }

    /// Add a kick data and inturn adjust the scores
    /// If a kick has changed sides, then
    /// * penalise prev side player and reward current side player
    /// * TODO: This needs to account for goal/half-time/...
    /// Else
    /// * if same player, reward to some extent
    ///   * provided ball maintained for a minimum sufficient time
    /// * if new player, reward prev player for a good pass.
    pub fn handle_kick(&mut self, kick: ActionData) {
        let ik = self.kicks.len();
        if ik > 0 {
            let prev = &self.kicks[ik-1];
            if prev.side != kick.side {
                self.players.score(prev.side, prev.playerid, SCORE_BAD_PASS);
                self.players.score(kick.side, kick.playerid, SCORE_HIJACK_PASS);
            } else {
                if prev.playerid == kick.playerid {
                    let dtime = kick.time as isize - prev.time as isize;
                    if dtime < SELF_PASS_MINTIME {
                        return;
                    }
                    self.players.score(prev.side, prev.playerid, SCORE_SELF_PASS);
                } else {
                    self.players.score(prev.side, prev.playerid, SCORE_GOOD_PASS_SENT);
                    self.players.score(kick.side, kick.playerid, SCORE_GOOD_PASS_GOT);
                }
            }
        }
        self.players.count_increment(kick.side, kick.playerid, Action::Kick(true));
        self.kicks.push(kick);
    }

    pub fn handle_tackle(&mut self, tackle: ActionData) {
        self.players.score(tackle.side, tackle.playerid, SCORE_TACKLE);
        self.players.count_increment(tackle.side, tackle.playerid, Action::Tackle(true));
    }

    pub fn handle_catch(&mut self, catch: ActionData) {
        self.players.score(catch.side, catch.playerid, SCORE_CATCH);
        self.players.count_increment(catch.side, catch.playerid, Action::Catch(true));
    }

    #[allow(dead_code)]
    fn summary_simple(&self) {
        for i in 0..self.players.aplayers.len() {
            let player = &self.players.aplayers[i];
            eprintln!("DBUG:PPGND:Proc:Passes:A:{:02}:{}", player.0, player.1.score);
        }
        for i in 0..self.players.bplayers.len() {
            let player = &self.players.bplayers[i];
            eprintln!("DBUG:PPGND:Proc:Passes:B:{:02}:{}", player.0, player.1.score);
        }
    }

    fn summary_asciiart(&self) {
        for i in 0..self.players.aplayers.len() {
            let player = &self.players.aplayers[i];
            eprint!("DBUG:PPGND:Proc:Passes:A:{:02}:", player.0);
            for _j in 0..player.1.score.round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for i in 0..self.players.bplayers.len() {
            let player = &self.players.bplayers[i];
            eprint!("DBUG:PPGND:Proc:Passes:B:{:02}:", player.0);
            for _j in 0..player.1.score.round() as usize {
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
            let player = &self.players.aplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            sx.nn_fill_rect(0.05, 0.05*(i as f32 + 4.0), 0.4*(player.1.score/amax), 0.04)
        }
        for i in 0..self.players.bplayers.len() {
            let player = &self.players.bplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            sx.nn_fill_rect(0.55, 0.05*(i as f32 + 4.0), 0.4*(player.1.score/bmax), 0.04)
        }
    }

    pub fn summary(&self) {
        self.summary_asciiart();
    }

}
