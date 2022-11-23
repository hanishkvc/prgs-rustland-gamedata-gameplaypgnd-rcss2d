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


const MTAG: &str = "PPGND:ProcActions";

const REPEAT_TACKLE_MINTIME: isize = 10;
const SELF_PASS_MINTIME: isize = 10;

//
// The scores are set such that a tackle followed by a
// kick by the other side, will still lead to a +ve score
// for the person who mounted the tackle (TACKLE-BADPASS)
//
const SCORE_BAD_PASS: f32 = -0.4;
const SCORE_HIJACK_PASS: f32 = -SCORE_BAD_PASS;
const SCORE_GOOD_PASS_SENT: f32 = 0.8;
const SCORE_GOOD_PASS_GOT: f32 = 0.4;
const SCORE_SELF_PASS: f32 = 0.06;
const SCORE_TACKLE: f32 = 0.6;
const SCORE_CATCH: f32 = 1.0;


#[derive(Debug)]
struct Score {
    score: f32,
    kicks: usize,
    tackles: usize,
    catchs: usize,
    dist: f32,
}

impl Score {

    fn new(score: f32, kicks: usize, tackles: usize, catchs: usize, dist: f32) -> Score {
        Score {
            score: score,
            kicks: kicks,
            tackles: tackles,
            catchs: catchs,
            dist: dist,
        }
    }

    fn default() -> Score {
        return Score::new(0.0, 0, 0, 0, 0.0);
    }

}


type Pos = (f32, f32);

#[derive(Debug)]
struct Players {
    aplayers: Vec<(usize, Score, Pos)>,
    bplayers: Vec<(usize, Score, Pos)>,
}

impl Players {

    fn new(acnt: usize, bcnt: usize) -> Players {
        let mut players = Players {
            aplayers: Vec::new(),
            bplayers: Vec::new(),
        };
        for i in 0..acnt {
            players.aplayers.push((i, Score::default(), (99.0,99.0)));
        }
        for i in 0..bcnt {
            players.bplayers.push((i, Score::default(), (99.0,99.0)));
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
        let stype;
        match atype {
            Action::None => todo!(),
            Action::Kick(_) => {
                stype = "Kick";
                player.1.kicks += 1;
            },
            Action::Catch(_) => {
                stype = "Catch";
                player.1.catchs += 1;
            },
            Action::Tackle(_) => {
                stype = "Tackle";
                player.1.tackles += 1;
            },
        }
        eprintln!("DBUG:{}:{}:{}:{}", MTAG, side, playerid, stype);
    }

    fn dist_update_from_pos(&mut self, side: char, playerid: usize, npos: Pos) {
        let player;
        if side == 'a' {
            player = &mut self.aplayers[playerid];
        } else {
            player = &mut self.bplayers[playerid];
        }
        let opos = player.2;
        if opos.0 == 99.0 && opos.1 == 99.0 {
            player.2 = npos;
            return;
        }
        let dx = npos.0-opos.0;
        let dy = npos.1-opos.1;
        let d = dx*dx + dy*dy;
        player.1.dist += d;
        player.2 = npos;
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

    /// Return the max player distance traversed for each of the teams
    fn dist_max(&self) -> (f32, f32) {
        let mut amax = f32::MIN;
        for i in 0..self.aplayers.len() {
            let player = &self.aplayers[i];
            if amax < player.1.dist {
                amax = player.1.dist;
            }
        }
        let mut bmax = f32::MIN;
        for i in 0..self.bplayers.len() {
            let player = &self.bplayers[i];
            if bmax < player.1.dist {
                bmax = player.1.dist;
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
    action: Action,
}

impl ActionData {

    pub fn new(time: usize, side: char, playerid: usize, pos: (f32,f32), action: Action) -> ActionData {
        ActionData {
            time: time,
            side: side,
            playerid: playerid,
            pos: pos,
            action: action,
        }
    }

}

#[derive(Debug)]
pub struct ActionsInfo {
    players: Players,
    actions: Vec<ActionData>,
}

impl ActionsInfo {

    pub fn new(acnt: usize, bcnt: usize) -> ActionsInfo {
        ActionsInfo {
            players: Players::new(acnt, bcnt),
            actions: Vec::new(),
        }
    }

    /// Add a kick data and inturn adjust the scores
    /// If a kick has changed sides, then
    /// * penalise prev side player and reward current side player
    /// * TODO: This needs to account for goal/half-time/...
    /// Else
    /// * if same player, reward to some extent
    ///   provided ball was maintained for a sufficiently minimum time
    /// * if diff players, reward both players for a good pass.
    ///
    /// ALERT: prev and current actions dont matter wrt current list of actions,
    /// except for the self pass case. However in future, if new actions are
    /// added, the logical flow will have to be evaluated and updated if reqd.
    pub fn handle_kick(&mut self, kick: ActionData) {
        let ik = self.actions.len();
        if ik > 0 {
            let prev = &self.actions[ik-1];
            if prev.side != kick.side {
                self.players.score(prev.side, prev.playerid, SCORE_BAD_PASS);
                self.players.score(kick.side, kick.playerid, SCORE_HIJACK_PASS);
            } else {
                if prev.playerid == kick.playerid {
                    if prev.action == kick.action {
                        let dtime = kick.time as isize - prev.time as isize;
                        if dtime < SELF_PASS_MINTIME {
                            eprintln!("DBUG:{}:{}:{}:Skipping TOO SOON repeat (self pass) kick????:{}:{}:{}", MTAG, kick.side, kick.playerid, prev.time, kick.time, dtime);
                            return;
                        }
                    }
                    self.players.score(prev.side, prev.playerid, SCORE_SELF_PASS);
                } else {
                    self.players.score(prev.side, prev.playerid, SCORE_GOOD_PASS_SENT);
                    self.players.score(kick.side, kick.playerid, SCORE_GOOD_PASS_GOT);
                }
            }
        }
        self.players.count_increment(kick.side, kick.playerid, Action::Kick(true));
        self.actions.push(kick);
    }

    /// Assumes a merged (be it kicks/tackles) actions vector.
    /// If the same player has repeat adjacent tackles, within a predefined short time,
    /// then the repeated tackle will be ignored.
    pub fn handle_tackle(&mut self, tackle: ActionData) {
        let it = self.actions.len();
        if it > 0 {
            let prev = &self.actions[it-1];
            if prev.side == tackle.side {
                if prev.playerid == tackle.playerid {
                    if prev.action == tackle.action {
                        let dtime = tackle.time as isize - prev.time as isize;
                        if dtime < REPEAT_TACKLE_MINTIME {
                            eprintln!("DBUG:{}:{}:{}:Skipping TOO SOON repeat tackle????:{}:{}:{}", MTAG, tackle.side, tackle.playerid, prev.time, tackle.time, dtime);
                            return;
                        }
                    }
                }
            }
        }
        self.players.score(tackle.side, tackle.playerid, SCORE_TACKLE);
        self.players.count_increment(tackle.side, tackle.playerid, Action::Tackle(true));
        self.actions.push(tackle);
    }

    pub fn handle_catch(&mut self, catch: ActionData) {
        self.players.score(catch.side, catch.playerid, SCORE_CATCH);
        self.players.count_increment(catch.side, catch.playerid, Action::Catch(true));
    }

    pub fn handle_action(&mut self, actiond: ActionData) {
        self.players.dist_update_from_pos(actiond.side, actiond.playerid, actiond.pos);
        match actiond.action {
            Action::Kick(_) => {
                self.handle_kick(actiond);
            },
            Action::Tackle(_) => {
                self.handle_tackle(actiond);
            },
            Action::Catch(_) => {
                self.handle_catch(actiond);
            },
            Action::None => {

            }
        }
    }

    #[allow(dead_code)]
    fn summary_simple(&self) {
        for i in 0..self.players.aplayers.len() {
            let player = &self.players.aplayers[i];
            eprintln!("DBUG:{}:A:{:02}:{}", MTAG, player.0, player.1.score);
        }
        for i in 0..self.players.bplayers.len() {
            let player = &self.players.bplayers[i];
            eprintln!("DBUG:{}:B:{:02}:{}", MTAG, player.0, player.1.score);
        }
    }

    fn summary_asciiart(&self) {
        for i in 0..self.players.aplayers.len() {
            let player = &self.players.aplayers[i];
            eprint!("DBUG:{}:A:{:02}:", MTAG, player.0);
            for _j in 0..player.1.score.round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for i in 0..self.players.bplayers.len() {
            let player = &self.players.bplayers[i];
            eprint!("DBUG:{}:B:{:02}:", MTAG, player.0);
            for _j in 0..player.1.score.round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
    }

    /// Graphics Summary (a relative performance graph)
    /// Take the max score across players wrt each team and
    /// plot score bar relative to that max score.
    ///
    /// SummaryType if 'a' => Bar relative to max in each team
    /// SummaryType if 'A' => Bar relative to max across both teams
    pub fn summary_score_sdl(&self, sx: &mut SdlX, summarytype: char) {
        // let (amax, bmax) = (20.0, 20.0);
        let (mut amax, mut bmax) = self.players.score_max();
        if summarytype == 'A' {
            amax = amax.max(bmax);
            bmax = amax;
        }
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

    pub fn summary_dist_sdl(&self, sx: &mut SdlX, summarytype: char) {
        let (mut amax, mut bmax) = self.players.dist_max();
        if summarytype == 'D' {
            amax = amax.max(bmax);
            bmax = amax;
        }
        let xs = 0.05;
        let xw = 0.4;
        let xu = xw/self.players.aplayers.len() as f32;
        let yb = 0.8;
        let yh = 0.1;
        for i in 0..self.players.aplayers.len() {
            let player = &self.players.aplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.1.dist/amax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
        }
        let xs = 0.55;
        for i in 0..self.players.bplayers.len() {
            let player = &self.players.bplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.1.dist/bmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
        }
    }

    pub fn summary(&self) {
        self.summary_asciiart();
    }

}
