//!
//! Identify pass quality
//! HanishKVC, 2022
//!
//! TODO:
//! * Track for halftime/etal and avoid providing -ve scoring
//!   due to any of these shifting side that will kick
//! * Allow -ve scoring to goalie, if they allow a goal to occur.
//! * Account cards/penalty during scoring.
//!

use std::fmt::Display;

use loggerk::{ldebug, log_d};
use sdl2::{pixels::Color, render::BlendMode};

use crate::sdlx::SdlX;
use crate::{entities, playdata};


const MTAG: &str = "PPGND:ProcActions";

/// Repeat consecutive tackle action records wrt the same player
/// is ignored for this duration intervals.
const REPEAT_TACKLE_MINTIME: isize = 10;
/// Dabling the ball around the field by the same player,
/// is ignored for this duration based intervals.
const SELF_PASS_MINTIME: isize = 10;
/// How much back in time to swim to find players to reward wrt goal.
const GOAL_CHAIN_TIME: usize = 30;
/// How long back should handle_action go when looking into the sequence of actions.
/// This needs to be same or larger than GAOL_CHAIN_TIME, ideally, in a simple sense.
const HA_LOOKBACK_MAX: usize = 30;

const SCORE_SELF_PASS_RATIO: f32 = 0.05;
/// Let goalie get a lesser penalty wrt self goal due to missed/unsuccessful catch,
/// bcas they have atleast tried to catch the goal related kick from other/own side.
const SCORE_GOALIE_MISSED_CATCH_PENALTY_RATIO: f32 = 0.5;
/// Scoring ratio for Otherside in a goal chain
const SCORE_GOALCHAIN_OTHERSIDE_BEYOND_IMMIDIATE_RATIO: f32 = 0.3;

#[derive(Debug)]
/// Maintain the scoring related to a player
struct Score {
    /// The overall actions related score
    ascore: f32,
    /// The number of kicks
    kicks: usize,
    /// The number of tackles
    tackles: usize,
    /// The number of catchs
    catchs: usize,
    /// The total distance traversed
    dist: f32,
    /// Card issued if any
    card: playdata::Card,
}

impl Score {

    fn new(ascore: f32, kicks: usize, tackles: usize, catchs: usize, dist: f32, card: playdata::Card) -> Score {
        Score {
            ascore: ascore,
            kicks: kicks,
            tackles: tackles,
            catchs: catchs,
            dist: dist,
            card: card,
        }
    }

    fn default() -> Score {
        return Score::new(0.0, 0, 0, 0, 0.0, playdata::Card::None);
    }

    fn score(&self, inc_cardscore: bool) -> f32 {
        let cardscore;
        if inc_cardscore {
            cardscore = match self.card {
                playdata::Card::None => 0.0,
                playdata::Card::Yellow => -1.5,
                playdata::Card::Red => -3.0,
            };
        } else {
            cardscore = 0.0;
        }
        return self.ascore + cardscore;
    }

}


type Pos = (f32, f32);

#[derive(Debug)]
struct Players {
    inc_cardscore: bool,
    lplayers: Vec<(usize, Score, Pos)>,
    rplayers: Vec<(usize, Score, Pos)>,
}

impl Players {

    fn new(lcnt: usize, rcnt: usize, inc_cardscore: bool) -> Players {
        let mut players = Players {
            inc_cardscore: inc_cardscore,
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
    fn card(&mut self, side: char, playerid: usize, card: playdata::Card) {
        if playerid >= entities::XPLAYERID_START {
            ldebug!(&format!("WARN:{}:Players:Card:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        } else {
            eprintln!("DBUG:{}:Players:Card:{}{:02}:{}", MTAG, side, playerid, card);
        }
        if side == entities::SIDE_L {
            self.lplayers[playerid].1.card = card;
        } else {
            self.rplayers[playerid].1.card = card;
        }
    }

    /// Help update the actions related score of a specific player
    fn ascore(&mut self, side: char, playerid: usize, score: f32) {
        if playerid >= entities::XPLAYERID_START {
            ldebug!(&format!("WARN:{}:Players:Score:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        } else {
            eprintln!("DBUG:{}:Players:Score:{}{:02}:{}", MTAG, side, playerid, score);
        }
        if side == entities::SIDE_L {
            self.lplayers[playerid].1.ascore += score;
        } else {
            self.rplayers[playerid].1.ascore += score;
        }
    }

    /// Help update the count wrt specified action of a specific player
    fn count_increment(&mut self, side: char, playerid: usize, atype: AIAction) {
        if playerid >= entities::XPLAYERID_START {
            ldebug!(&format!("WARN:{}:Players:CountInc:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        }
        let player;
        if side == entities::SIDE_L {
            player = &mut self.lplayers[playerid];
        } else {
            player = &mut self.rplayers[playerid];
        }
        let stype;
        match atype {
            AIAction::None => stype = "None",
            AIAction::Kick => {
                stype = "Kick";
                player.1.kicks += 1;
            },
            AIAction::Catch => {
                stype = "Catch";
                player.1.catchs += 1;
            },
            AIAction::Tackle => {
                stype = "Tackle";
                player.1.tackles += 1;
            },
            AIAction::Goal => stype = "Goal",
        }
        ldebug!(&format!("DBUG:{}:CountInc:{}{:02}:{}", MTAG, side, playerid, stype));
    }

    fn dist_update_from_pos(&mut self, side: char, playerid: usize, npos: Pos) {
        if playerid >= entities::XPLAYERID_START {
            ldebug!(&format!("WARN:{}:Players:DistUpdateFromPos:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        }
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

    /// Return the min and max player score for each of the teams
    fn score_minmax(&self) -> ((f32,f32), (f32,f32)) {
        let mut lmax = f32::MIN;
        let mut lmin = f32::MAX;
        for i in 0..self.lplayers.len() {
            let player = &self.lplayers[i];
            if lmax < player.1.score(self.inc_cardscore) {
                lmax = player.1.score(self.inc_cardscore);
            }
            if lmin > player.1.score(self.inc_cardscore) {
                lmin = player.1.score(self.inc_cardscore);
            }
        }
        let mut rmax = f32::MIN;
        let mut rmin = f32::MAX;
        for i in 0..self.rplayers.len() {
            let player = &self.rplayers[i];
            if rmax < player.1.score(self.inc_cardscore) {
                rmax = player.1.score(self.inc_cardscore);
            }
            if rmin > player.1.score(self.inc_cardscore) {
                rmin = player.1.score(self.inc_cardscore);
            }
        }
        ((lmin,lmax), (rmin,rmax))
    }

    /// Return the max player distance traversed for each of the teams
    fn dist_max(&self) -> (f32, f32) {
        let mut lmax = f32::MIN;
        for i in 0..self.lplayers.len() {
            let player = &self.lplayers[i];
            if lmax < player.1.dist {
                lmax = player.1.dist;
            }
        }
        let mut rmax = f32::MIN;
        for i in 0..self.rplayers.len() {
            let player = &self.rplayers[i];
            if rmax < player.1.dist {
                rmax = player.1.dist;
            }
        }
        (lmax, rmax)
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

impl Display for AIAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            AIAction::None => "None",
            AIAction::Kick => "Kick",
            AIAction::Tackle => "Tackle",
            AIAction::Catch => "Catch",
            AIAction::Goal => "Goal",
        };
        f.write_str(data)
    }
}

impl AIAction {

    /// (TheScore, OwnPrevRatio,OwnCurRatio, OtherPrevRatio,OwnCurRatio)
    ///
    /// Wrt Goal the curside is the side which got the goal and curplayerid is unknown by default
    /// the prev player who delivered the kick leading to the goal is the player who scored the goal
    ///
    /// TOTHINK: Should scores be set such that a tackle followed by a kick by the other side, still
    /// lead to a +ve score for the person who mounted the tackle (TACKLE+BADPASS) ???
    ///
    pub fn scoring(&self) -> (f32, f32,f32, f32,f32) {
        match self {
            AIAction::None => (0.0, 0.0,0.0, 0.0,0.0),
            AIAction::Kick => (0.6, 0.5,0.5, -0.8,0.8),
            AIAction::Tackle => (0.4, 0.5,0.5, -0.4,0.6),
            AIAction::Catch => (1.0, 0.4,0.4, 0.2,0.8), // Give some score to otherprev player bcas they tried to achieve a goal
            AIAction::Goal => (1.0, 1.0,0.0, -1.0,0.0),
        }
    }

}


#[derive(Debug, Clone)]
/// Maintain the required info wrt a game action.
pub struct ActionData {
    pub time: usize,
    side: char,
    playerid: usize,
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

    fn print(&self, print_aia_none: bool) {
        let mut bprint = true;
        match self.action {
            AIAction::None => {
                bprint = print_aia_none;
            },
            _ => (),
        }
        if bprint {
            eprintln!("DBUG:{}:ActionData:{}", MTAG, self);
        }
    }

}

impl Display for ActionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}:{}{:02}:{}:({},{})]", self.time, self.side, self.playerid, self.action, self.pos.0, self.pos.1))
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

    #[allow(dead_code)]
    /// Search through the actions list/vec in reverse order, till one finds
    /// a action that one is looking for, or the amount of records to check
    /// is exhausted.
    ///
    /// CheckMax: If 0, then check through all records, or else only check the
    /// specified number of records.
    fn find_actiondata_rev(&mut self, act: AIAction, checkmax: usize) -> Option<ActionData> {
        let mut checkcnt = 0;
        for i in (0..self.actions.len()).rev() {
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

    fn summary_simple(&self) {
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            eprintln!("DBUG:{}:L{:02}:{}", MTAG, player.0, player.1.score());
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            eprintln!("DBUG:{}:R{:02}:{}", MTAG, player.0, player.1.score());
        }
    }

    fn summary_asciiart(&self) {
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            eprint!("DBUG:{}:L{:02}:", MTAG, player.0);
            for _j in 0..player.1.score().round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            eprint!("DBUG:{}:R{:02}:", MTAG, player.0);
            for _j in 0..player.1.score().round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
    }

    /// Graphics Summary (a relative performance graph)
    /// Take the max score across players wrt each team and
    /// plot score bar relative to that max score.
    ///
    /// SummaryType if 'T' => Bar relative to max in each team
    /// SummaryType if 'A' => Bar relative to max across both teams
    pub fn summary_score_sdl(&self, sx: &mut SdlX, summarytype: char) {
        // let (amax, bmax) = (20.0, 20.0);
        let ((mut lmin, mut lmax), (mut rmin, mut rmax)) = self.players.score_minmax();
        if summarytype == 'A' {
            lmax = lmax.max(rmax);
            rmax = lmax;
            lmin = lmin.min(rmin);
            rmin = lmin;
        }
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let lpscore = player.1.score() - lmin;
            sx.nn_fill_rect(0.05, 0.05*(i as f32 + 4.0), 0.4*(lpscore/(lmax-lmin)), 0.04)
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let rpscore = player.1.score() - rmin;
            sx.nn_fill_rect(0.55, 0.05*(i as f32 + 4.0), 0.4*(rpscore/(rmax-rmin)), 0.04)
        }
    }

    /// SummaryType if 'T' => Bar relative to max in each team
    /// SummaryType if 'A' => Bar relative to max across both teams
    pub fn summary_dist_sdl(&self, sx: &mut SdlX, summarytype: char) {
        let (mut lmax, mut rmax) = self.players.dist_max();
        if summarytype == 'A' {
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
        self.summary_simple();
    }

}


/// Helps tell what should one do after the handle helper returns
pub enum HAReturn {
    /// Continue searching/checking the history further back, cas not yet done
    #[allow(dead_code)]
    ContinueSearch,
    /// Stop checking the history at this point, ie be done with it.
    /// Inturn indicate whether to save the current action or not
    Done(bool),
}

impl ActionsInfo {

    /// Score wrt good and bad passes (ie btw members of same team or not)
    /// Lower score wrt self pass (ie same player keeping the ball going)
    /// Lower scoring for kick after a goal
    pub fn handle_kick(&mut self, curactd: &mut ActionData, prevactd: &ActionData) -> HAReturn {
        let score = curactd.action.scoring();
        match prevactd.action {
            AIAction::None => panic!("DBUG:{}:HandleKick:Unexpect None{}->Kick{}", MTAG, prevactd, curactd),
            AIAction::Kick | AIAction::Catch | AIAction::Tackle => {
                if prevactd.side == curactd.side {
                    let mut ppscore = score.0 * score.1;
                    let mut cpscore = score.0 * score.2;
                    if (prevactd.playerid == curactd.playerid) && (prevactd.action == AIAction::Kick) {
                        let dtime = curactd.time-prevactd.time;
                        if dtime < SELF_PASS_MINTIME as usize {
                            ldebug!(&format!("DBUG:{}:{}{:02}:HandleKick:Skipping TOO SOON repeat (self pass) kick????:{}:{}:{}", MTAG, curactd.side, curactd.playerid, prevactd.time, curactd.time, dtime));
                            return HAReturn::Done(false);
                        }
                        ppscore *= SCORE_SELF_PASS_RATIO;
                        cpscore *= SCORE_SELF_PASS_RATIO;
                    }
                    self.players.ascore(prevactd.side, prevactd.playerid, ppscore);
                    self.players.ascore(curactd.side, curactd.playerid, cpscore);
                } else {
                    let pscore = score.0 * score.3;
                    self.players.ascore(prevactd.side, prevactd.playerid, pscore);
                    let pscore = score.0 * score.4;
                    self.players.ascore(curactd.side, curactd.playerid, pscore);
                }
                return HAReturn::Done(true);
            },
            AIAction::Goal => {
                if prevactd.side == curactd.side {
                    // After a side gets a goal, the otherside should kick
                    // The person who has kicked currently has taken ball from the other side immidiately itself!
                    // This shouldnt occur normally???
                    panic!("DBUG:{}:HandleKick:Goal{}->Kick{}, wrt same side???", MTAG, prevactd, curactd);
                } else {
                    // This is like a no effort kick potentially, ie after a goal, so low score
                    let pscore = score.0 * score.2 * SCORE_SELF_PASS_RATIO;
                    self.players.ascore(curactd.side, curactd.playerid, pscore);
                    return HAReturn::Done(true);
                }
            },
        }
    }

    /// Handle a Goal Action, by trying to find the kick or tackle which might have lead to the goal.
    /// * Inturn set the player responsible for the goal.
    /// Allow for a catch action not succeeding in stopping a goal.
    /// Allow the chain of actions leading to the goal to be identified and the players scored to some extent.
    /// * -ve scoring wrt other side players is restricted to
    ///   * the self goal situation of a immidiate prev action to the goal action.
    ///   * and atmost the 1st beyond-self-goal other side player, which also terminates the chain
    /// * the chain is terminated when we have gone back through the alloted/specified time/cnt.
    ///
    /// TODO: Need to check if Tackle is related to a possible contact with Ball (by checking for ball to be very near),
    /// as tackle action data wrt rcss may also involve contact btw oppositie side players and no ball in picture,
    /// potentially (need to check this bit more).
    fn handle_goal(&mut self, curactd: &mut ActionData, prevactd: &ActionData, lookbackcnt: usize) -> HAReturn {
        let score = curactd.action.scoring();
        match prevactd.action {
            AIAction::None => {
                panic!("DBUG:{}:HandleGoal:{}:None {} shouldnt be in actions list", MTAG, curactd, prevactd);
            },
            AIAction::Goal => {
                if lookbackcnt <= 1 {
                    panic!("DBUG:{}:HandleGoal:Goal{}->Goal{} shouldnt occur", MTAG, prevactd, curactd);
                }
                return HAReturn::Done(true);
            },
            AIAction::Kick | AIAction::Tackle | AIAction::Catch => {
                if lookbackcnt <= 1 {
                    if curactd.playerid >= entities::XPLAYERID_START {
                        // Fill the player responsible for the goal bcas
                        // One doesnt know whether a kick will become a goal or not
                        // at the time of the kick, in general.
                        curactd.playerid = prevactd.playerid;
                        eprintln!("INFO:{}:HandleGoal:{}:Player updated; PrevAction {}", MTAG, curactd, prevactd);
                    } else {
                        eprintln!("WARN:{}:HandleGoal:{}:Player already set; PrevAction {}", MTAG, curactd, prevactd);
                    }
                }
                if prevactd.side == curactd.side {
                    // A successful goal
                    // Nearest player scores more compared to farther players, wrt the chain of actions leading to the goal
                    let pscore = score.0 * score.1 * (1.0/lookbackcnt as f32);
                    let pid = if lookbackcnt <= 1 { curactd.playerid } else { prevactd.playerid };
                    self.players.ascore(curactd.side, pid, pscore);
                    if (curactd.time - prevactd.time) > GOAL_CHAIN_TIME {
                        return HAReturn::Done(true);
                    }
                    return HAReturn::ContinueSearch;
                } else {
                    let mut pscore = score.0 * score.3 * (1.0/lookbackcnt as f32);
                    if lookbackcnt <= 1 {
                        // a self goal !?!
                        curactd.playerid += entities::XPLAYERID_OOPS_OTHERSIDE_START;
                        eprintln!("WARN:{}:HandleGoal:{}:Player updated - SelfGoal; PrevAction {}", MTAG, curactd, prevactd);
                        if prevactd.action == AIAction::Catch {
                            pscore *= SCORE_GOALIE_MISSED_CATCH_PENALTY_RATIO;
                        }
                        self.players.ascore(prevactd.side, prevactd.playerid, pscore);
                        return HAReturn::ContinueSearch;
                    }
                    pscore *= SCORE_GOALCHAIN_OTHERSIDE_BEYOND_IMMIDIATE_RATIO;
                    self.players.ascore(prevactd.side, prevactd.playerid, pscore);
                    HAReturn::Done(true)
                }
            },
        }
    }

    /// Score tackle involving same side and opposite (slightly higher scoring) side.
    /// Filter same player adjacent (wrt time) tackle actions to a minimum number.
    ///   Retain action only if still present over a long period of time, without
    ///   any other player/actions in between.
    fn handle_tackle(&mut self, curactd: &mut ActionData, prevactd: &ActionData) -> HAReturn {
        let score = curactd.action.scoring();
        match prevactd.action {
            AIAction::None => {
                panic!("DBUG:{}:HandleTackle:None{}->Tackle{} shouldnt occur", MTAG, prevactd, curactd);
            },
            AIAction::Kick => {
                let ppscore;
                let cpscore;
                if prevactd.side == curactd.side {
                    ppscore = score.0 * score.1;
                    cpscore = score.0 * score.2;
                } else {
                    ppscore = score.0 * score.3;
                    cpscore = score.0 * score.4;
                }
                self.players.ascore(prevactd.side, prevactd.playerid, ppscore);
                self.players.ascore(curactd.side, curactd.playerid, cpscore);
                return HAReturn::Done(true);
            },
            AIAction::Tackle => {
                let ppscore;
                let cpscore;
                if prevactd.side == curactd.side {
                    if prevactd.playerid == curactd.playerid {
                        let dtime = curactd.time-prevactd.time;
                        if dtime < REPEAT_TACKLE_MINTIME as usize {
                            ldebug!(&format!("DBUG:{}:{}{:02}:HandleTackle:Skipping TOO SOON repeat tackle data!?!:{}:{}:{}", MTAG, curactd.side, curactd.playerid, prevactd.time, curactd.time, dtime));
                            return HAReturn::Done(false);
                        }
                    }
                    ppscore = score.0 * score.1;
                    cpscore = score.0 * score.2;
                } else {
                    ppscore = score.0 * score.3;
                    cpscore = score.0 * score.4;
                }
                self.players.ascore(prevactd.side, prevactd.playerid, ppscore);
                self.players.ascore(curactd.side, curactd.playerid, cpscore);
                return HAReturn::Done(true);
            },
            AIAction::Catch | AIAction::Goal => {
                // Shouldnt occur (there should be a kick after these), however if it occurs, ignore for now
                eprintln!("WARN:{}:HandleTackle:Catch/Goal{}->Tackle{} shouldnt occur, ignoring...", MTAG, prevactd, curactd);
                return HAReturn::Done(false);
            },
        }
    }

    /// Score the players in the immidiate sequence leading to a catch.
    ///   Even if a action leads to the catch in the next time step, still
    ///   give that player some +ve score, bcas they tried to hit a goal.
    ///   NOTE: This is managed based on the scoring assigned wrt catch seqs.
    fn handle_catch(&mut self, curactd: &mut ActionData, prevactd: &ActionData) -> HAReturn {
        let score = curactd.action.scoring();
        match prevactd.action {
            AIAction::None => {
                panic!("DBUG:{}:HandleCatch:None{}->Catch{} shouldnt occur", MTAG, prevactd, curactd);
            },
            AIAction::Kick | AIAction::Tackle => {
                let ppscore;
                let cpscore;
                if prevactd.side == curactd.side {
                    ppscore = score.0 * score.1;
                    cpscore = score.0 * score.2;
                } else {
                    ppscore = score.0 * score.3;
                    cpscore = score.0 * score.4;
                }
                self.players.ascore(prevactd.side, prevactd.playerid, ppscore);
                self.players.ascore(curactd.side, curactd.playerid, cpscore);
                return HAReturn::Done(true);
            },
            AIAction::Catch | AIAction::Goal => {
                // Shouldnt occur (there should be a kick after these), however if it occurs, ignore for now
                eprintln!("WARN:{}:HandleCatch:Catch/Goal{}->Catch{} shouldnt occur, ignoring...", MTAG, prevactd, curactd);
                return HAReturn::Done(false);
            },
        }
    }

    /// Handle the passed action by
    /// * looking at the sequence leading to it and scoring players accordingly.
    ///   * normally 1 step back,
    ///   * multi step wrt goal chaining.
    /// * updating action related counters
    /// * maintaing a list of raw and filtered list/vec of actions
    pub fn handle_action(&mut self, mut curactd: ActionData) {
        curactd.print(false);
        let mut bupdate_actions = false;
        let mut bupdate_rawactions = true;
        let mut bupdate_dist = true;
        let mut lookbackcnt = 0;
        for i in (0..self.actions.len()).rev() {
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
                    if let HAReturn::Done(save) = self.handle_kick(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Tackle => {
                    if let HAReturn::Done(save) = self.handle_tackle(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Catch => {
                    if let HAReturn::Done(save) = self.handle_catch(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Goal => {
                    // Allow 1st lookback check to filter out this goal if required.
                    // Else If one goes beyond 1st lookback, then the goal is always saved into actions.
                    bupdate_dist = false;
                    if lookbackcnt > 1 {
                        bupdate_actions = true;
                    }
                    if let HAReturn::Done(save) = self.handle_goal(&mut curactd, &prevactd, lookbackcnt) {
                        if lookbackcnt <= 1 {
                            bupdate_actions = save;
                        }
                        break;
                    }
                },
            }
        }
        // Handle the special case of 1st action
        if self.actions.len() == 0 {
            if let AIAction::Kick = curactd.action {
                bupdate_actions = true;
            } else if let AIAction::None = curactd.action {
                bupdate_rawactions = false;
            }
        }
        // Update things as required.
        if bupdate_dist {
            self.players.dist_update_from_pos(curactd.side, curactd.playerid, curactd.pos);
        }
        if bupdate_actions {
            self.actions.push(curactd.clone());
        }
        if bupdate_rawactions {
            self.players.count_increment(curactd.side, curactd.playerid, curactd.action.clone());
            ldebug!(&format!("DBUG:{}:RawActions:{}", MTAG, curactd));
            self.rawactions.push(curactd);
        }
    }

}

impl ActionsInfo {

    pub fn handle_card(&mut self, side: char, playerid: usize, card: playdata::Card) {
        self.players.card(side, playerid, card);
    }

}
