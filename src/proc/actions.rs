//!
//! Identify pass quality
//! HanishKVC, 2022
//!
//! TODO:
//! * Track for goals/halftime/etal and avoid providing -ve scoring
//!   due to any of these shifting side that will kick
//! * Add support for providing scores for tackle/catch/etal
//!

use loggerk::{ldebug, log_d};
use sdl2::{pixels::Color, render::BlendMode};

use crate::sdlx::SdlX;
use crate::playdata::Action;
use crate::entities;


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
const SCORE_GOAL: f32 = 1.0;

const HA_LOOKBACK_MAX: usize = 4;
const SCORE_SELF_PASS_RATIO: f32 = 0.05;

#[derive(Debug)]
/// Maintain the scoring related to a player
struct Score {
    /// The overall score
    score: f32,
    /// The number of kicks
    kicks: usize,
    /// The number of tackles
    tackles: usize,
    /// The number of catchs
    catchs: usize,
    /// The total distance traversed
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
    lplayers: Vec<(usize, Score, Pos)>,
    rplayers: Vec<(usize, Score, Pos)>,
}

impl Players {

    fn new(lcnt: usize, rcnt: usize) -> Players {
        let mut players = Players {
            lplayers: Vec::new(),
            rplayers: Vec::new(),
        };
        for i in 0..lcnt {
            players.lplayers.push((i, Score::default(), (99.0,99.0)));
        }
        for i in 0..rcnt {
            players.rplayers.push((i, Score::default(), (99.0,99.0)));
        }
        return players;
    }

    /// Help update the score of a specific player
    fn score(&mut self, side: char, playerid: usize, score: f32) {
        if side == entities::SIDE_L {
            self.lplayers[playerid].1.score += score;
        } else {
            self.rplayers[playerid].1.score += score;
        }
    }

    /// Help update the score of a specific player
    fn count_increment(&mut self, side: char, playerid: usize, atype: Action) {
        let player;
        if side == entities::SIDE_L {
            player = &mut self.lplayers[playerid];
        } else {
            player = &mut self.rplayers[playerid];
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
        ldebug!(&format!("DBUG:{}:{}:{}:{}", MTAG, side, playerid, stype));
    }

    fn dist_update_from_pos(&mut self, side: char, playerid: usize, npos: Pos) {
        let player;
        if side == entities::SIDE_L {
            player = &mut self.lplayers[playerid];
        } else {
            player = &mut self.rplayers[playerid];
        }
        let opos = player.2;
        if opos.0 == 99.0 && opos.1 == 99.0 {
            player.2 = npos;
            return;
        }
        let dx = npos.0-opos.0;
        let dy = npos.1-opos.1;
        let d = dx*dx + dy*dy;
        player.1.dist += d.sqrt();
        player.2 = npos;
    }

    /// Return the max player score for each of the teams
    fn score_max(&self) -> (f32, f32) {
        let mut amax = f32::MIN;
        for i in 0..self.lplayers.len() {
            let player = &self.lplayers[i];
            if amax < player.1.score {
                amax = player.1.score;
            }
        }
        let mut bmax = f32::MIN;
        for i in 0..self.rplayers.len() {
            let player = &self.rplayers[i];
            if bmax < player.1.score {
                bmax = player.1.score;
            }
        }
        (amax, bmax)
    }

    /// Return the max player distance traversed for each of the teams
    fn dist_max(&self) -> (f32, f32) {
        let mut amax = f32::MIN;
        for i in 0..self.lplayers.len() {
            let player = &self.lplayers[i];
            if amax < player.1.dist {
                amax = player.1.dist;
            }
        }
        let mut bmax = f32::MIN;
        for i in 0..self.rplayers.len() {
            let player = &self.rplayers[i];
            if bmax < player.1.dist {
                bmax = player.1.dist;
            }
        }
        (amax, bmax)
    }

}


#[derive(Debug, Clone, PartialEq)]
pub enum AIAction {
    None,
    Kick,
    Tackle,
    Catch,
    Goal,
}

impl AIAction {

    /// (TheScore, OwnPrevRatio,OwnCurRatio, OtherPrevRatio,OwnCurRatio)
    ///
    /// Wrt Goal the curside is the side which got the goal and curplayerid is unknown by default
    /// the prev player who delivered the kick leading to the goal is the player who scored the goal
    pub fn scoring(&self) -> (f32, f32,f32, f32,f32) {
        match self {
            AIAction::None => (0.0, 0.0,0.0, 0.0,0.0),
            AIAction::Kick => (0.6, 0.5,0.5, -0.8,0.8),
            AIAction::Tackle => (0.2, 0.0,0.0, 1.0,0.0),
            AIAction::Catch => (1.0, 0.0,0.0, 0.2,0.8),
            AIAction::Goal => (1.0, 1.0,0.0, -1.0,0.0),
        }
    }

}


#[derive(Debug, Clone)]
pub struct ActionData {
    pub time: usize,
    side: char,
    playerid: usize,
    #[allow(dead_code)]
    pub pos: (f32, f32),
    action: AIAction,
}

impl ActionData {

    pub fn new(time: usize, side: char, playerid: usize, pos: (f32,f32), action: AIAction) -> ActionData {
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
/// Contains info about game actions and inturn performance of the players
/// NOTE: Movement is not a action, but only a perf characteristics
pub struct ActionsInfo {
    players: Players,
    /// Contains significant game actions
    actions: Vec<ActionData>,
    /// Contains all game actions, even same actions which are too near in time.
    pub rawactions: Vec<ActionData>,
}

impl ActionsInfo {

    pub fn new(acnt: usize, bcnt: usize) -> ActionsInfo {
        ActionsInfo {
            players: Players::new(acnt, bcnt),
            actions: Vec::new(),
            rawactions: Vec::new(),
        }
    }

    /// Search through the actions list/vec in reverse order, till one finds
    /// a action that one is looking for, or the amount of records to check
    /// is exhausted.
    ///
    /// CheckMax: If 0, then check through all records, or else only check the
    /// specified number of records.
    fn find_actiondata_rev(&mut self, act: AIAction, checkmax: usize) -> Option<ActionData> {
        let mut checkcnt = 0;
        for i in (0..self.actions.len()-1).rev() {
            checkcnt += 1;
            if (checkmax > 0) && (checkcnt > checkmax) {
                break;
            }
            let checkact = &self.actions[i];
            if checkact.action == act {
                return Some(checkact.clone());
            }
        }
        None
    }

    /// Add a kick data and inturn adjust the scores
    /// If a kick has changed sides, then
    /// * penalise prev side player and reward current side player
    ///   * However if prev was a goal, dont penalise prev
    ///   * TODO: This needs to account for half-time/...
    /// Else
    /// * if same player, reward to some extent
    ///   provided ball was maintained for a sufficiently minimum time
    /// * if diff players, reward both players for a good pass.
    /// * NOTE: Small score_self_pass wrt self goal case, is not explicitly avoided for now.
    ///
    /// ALERT: prev and current actions dont matter wrt current list of actions,
    /// except for the self pass case. However in future, if new actions are
    /// added, the logical flow will have to be evaluated and updated if reqd.
    ///
    /// NOTE: Scoring wrt goal +ve or -ve is handled in handle_action itself.
    ///
    pub fn handle_kick(&mut self, kick: ActionData) {
        let ik = self.actions.len();
        if ik > 0 {
            let prev = &self.actions[ik-1];
            if prev.side != kick.side {
                if prev.action != AIAction::Goal {
                    self.players.score(prev.side, prev.playerid, SCORE_BAD_PASS);
                }
                self.players.score(kick.side, kick.playerid, SCORE_HIJACK_PASS);
            } else {
                if prev.playerid == kick.playerid {
                    if prev.action == kick.action {
                        let dtime = kick.time as isize - prev.time as isize;
                        if dtime < SELF_PASS_MINTIME {
                            ldebug!(&format!("DBUG:{}:{}:{}:Skipping TOO SOON repeat (self pass) kick????:{}:{}:{}", MTAG, kick.side, kick.playerid, prev.time, kick.time, dtime));
                            return;
                        }
                    }
                    self.players.score(prev.side, prev.playerid, SCORE_SELF_PASS);
                } else {
                    if prev.action != AIAction::Goal { // ie not a self goal
                        self.players.score(prev.side, prev.playerid, SCORE_GOOD_PASS_SENT);
                    }
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
                            ldebug!(&format!("DBUG:{}:{}:{}:Skipping TOO SOON repeat tackle????:{}:{}:{}", MTAG, tackle.side, tackle.playerid, prev.time, tackle.time, dtime));
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

    /// Goal action is handled here itself,
    /// other are passed to respective handlers
    ///
    /// TODO: If there is a catch inbetween, a kick and a goal,
    /// then there is some issue with data, and the goal cant be
    /// attributed to the immidiate previous kick. However this
    /// is not taken care of currently.
    pub fn handle_action(&mut self, actiond: ActionData) {
        let mut bupdatedist = true;
        match actiond.action {
            AIAction::Kick => {
                self.rawactions.push(actiond.clone());
                self.handle_kick(actiond.clone());
            },
            AIAction::Tackle => {
                self.rawactions.push(actiond.clone());
                self.handle_tackle(actiond.clone());
            },
            AIAction::Catch => {
                self.rawactions.push(actiond.clone());
                self.handle_catch(actiond.clone());
            },
            AIAction::Goal => {
                bupdatedist = false;
                let ik = self.actions.len();
                let mut curact = actiond.clone();
                if ik > 0 {
                    let prevact = self.find_actiondata_rev(AIAction::Kick, 4)
                        .expect(&format!("DBUG:{}:HandleAction:Goal {:?}:No immidiate prev kick found:PrevAction {:?}", MTAG, curact, self.actions[ik-1]));
                    if curact.playerid == entities::XPLAYERID_UNKNOWN {
                        // Fill the player responsible for the goal bcas
                        // One doesnt know whether a kick will become a goal or not
                        // at the time of the kick, in general.
                        curact.playerid = prevact.playerid;
                    } else {
                        eprintln!("WARN:{}:HandleAction:Goal {:?}:Player already set; PrevAction kick {:?}", MTAG, curact, prevact);
                    }
                    if prevact.side == curact.side {
                        // A successful goal
                        self.players.score(curact.side, curact.playerid, SCORE_GOAL);
                    } else {
                        // a self goal !?!
                        self.players.score(curact.side, curact.playerid, -SCORE_GOAL);
                    }
                }
                self.rawactions.push(curact.clone());
                self.actions.push(curact);
            }
            AIAction::None => {
                // usual player movements on the field
            }
        }
        if bupdatedist {
            self.players.dist_update_from_pos(actiond.side, actiond.playerid, actiond.pos);
        }
    }

    #[allow(dead_code)]
    fn summary_simple(&self) {
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            eprintln!("DBUG:{}:A:{:02}:{}", MTAG, player.0, player.1.score);
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            eprintln!("DBUG:{}:B:{:02}:{}", MTAG, player.0, player.1.score);
        }
    }

    fn summary_asciiart(&self) {
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            eprint!("DBUG:{}:A:{:02}:", MTAG, player.0);
            for _j in 0..player.1.score.round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
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
        let (mut lmax, mut rmax) = self.players.score_max();
        if summarytype == 'A' {
            lmax = lmax.max(rmax);
            rmax = lmax;
        }
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            sx.nn_fill_rect(0.05, 0.05*(i as f32 + 4.0), 0.4*(player.1.score/lmax), 0.04)
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            sx.nn_fill_rect(0.55, 0.05*(i as f32 + 4.0), 0.4*(player.1.score/rmax), 0.04)
        }
    }

    pub fn summary_dist_sdl(&self, sx: &mut SdlX, summarytype: char) {
        let (mut lmax, mut rmax) = self.players.dist_max();
        if summarytype == 'D' {
            lmax = lmax.max(rmax);
            rmax = lmax;
        }
        let xs = 0.05;
        let xw = 0.4;
        let xu = xw/self.players.lplayers.len() as f32;
        let yb = 0.8;
        let yh = 0.1;
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.1.dist/lmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
        }
        let xs = 0.55;
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.1.dist/rmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
        }
    }

    pub fn summary(&self) {
        self.summary_asciiart();
    }

}

pub enum HAReturn {
    /// Continue checking the history further back
    Continue,
    /// Stop checking the history at this point.
    /// Inturn indicate whether to save the current action or not
    Break(bool),
}

impl ActionsInfo {

    pub fn handle_kickx(&mut self, curactd: &mut ActionData, prevactd: &ActionData) -> HAReturn {
        let score = curactd.action.scoring();
        match prevactd.action {
            AIAction::None => panic!("DBUG:{}:HandleKick:Unexpect None{:?}->Kick{:?}", MTAG, prevactd, curactd),
            AIAction::Kick | AIAction::Catch | AIAction::Tackle => {
                if prevactd.side == curactd.side {
                    let mut ppscore = score.0 * score.1;
                    let mut cpscore = score.0 * score.2;
                    if (prevactd.playerid == curactd.playerid) && (prevactd.action == AIAction::Kick) {
                        let dtime = curactd.time-prevactd.time;
                        if dtime < SELF_PASS_MINTIME as usize {
                            ldebug!(&format!("DBUG:{}:{}:{}:Skipping TOO SOON repeat (self pass) kick????:{}:{}:{}", MTAG, curactd.side, curactd.playerid, prevactd.time, curactd.time, dtime));
                            return HAReturn::Break(false);
                        }
                        ppscore *= SCORE_SELF_PASS_RATIO;
                        cpscore *= SCORE_SELF_PASS_RATIO;
                    }
                    self.players.score(prevactd.side, prevactd.playerid, ppscore);
                    self.players.score(curactd.side, curactd.playerid, cpscore);
                } else {
                    let pscore = score.0 * score.3;
                    self.players.score(prevactd.side, prevactd.playerid, pscore);
                    let pscore = score.0 * score.4;
                    self.players.score(curactd.side, curactd.playerid, pscore);
                }
                return HAReturn::Break(true);
            },
            AIAction::Goal => {
                if prevactd.side == curactd.side {
                    // After a side gets a goal, the otherside should kick
                    // The person who has kicked currently has taken ball from the other side immidiately itself!
                    // This shouldnt occur normally???
                    panic!("DBUG:{}:HandleKick:Goal->Kick wrt same side???:{:?}-{:?}", MTAG, prevactd, curactd);
                } else {
                    // This is like a no effort kick potentially, ie after a goal, so low score
                    let pscore = score.0 * score.2 * SCORE_SELF_PASS_RATIO;
                    self.players.score(curactd.side, curactd.playerid, pscore);
                    return HAReturn::Break(true);
                }
            },
        }
    }

    #[allow(dead_code)]
    pub fn handle_actionx(&mut self, mut curactd: ActionData) {
        let mut bupdate_actions = false;
        let mut bupdate_rawactions = true;
        let mut bupdate_dist = true;
        let mut lookbackcnt = 0;
        for i in (0..self.actions.len()-1).rev() {
            lookbackcnt += 1;
            if lookbackcnt > HA_LOOKBACK_MAX {
                break;
            }
            let prevactd = self.actions[i].clone();
            match curactd.action {
                AIAction::None => {
                    bupdate_rawactions = false;
                    break;
                },
                AIAction::Kick => {
                    if let HAReturn::Break(save) = self.handle_kickx(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Tackle => todo!(),
                AIAction::Catch => todo!(),
                AIAction::Goal => {
                    bupdate_dist = false;
                },
            }
        }
        if bupdate_dist {
            self.players.dist_update_from_pos(curactd.side, curactd.playerid, curactd.pos);
        }
        if bupdate_actions {
            self.actions.push(curactd.clone());
        }
        if bupdate_rawactions {
            self.rawactions.push(curactd);
        }
    }

}
