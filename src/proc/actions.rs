//!
//! Identify pass quality
//! HanishKVC, 2022
//!
//! TODO:
//! * Track for halftime/etal and avoid providing -ve scoring
//!   due to any of these shifting side that will kick
//! * Allow -ve scoring to goalie, if they allow a goal to occur
//!   ie beyond the failed catch situations.
//! * Account penalties beyond cards during scoring.
//!

use std::fmt::Display;

use loggerk::{ldebug, log_d};
use sdl2::{pixels::Color, render::BlendMode};

use crate::sdlx::{SdlX, XRect};
use crate::{entities, playdata};


const MTAG: &str = "GPPGND:ProcActions";

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

/// Relative summary graphs wrt Best in respective Team
pub const SUMMARY_RELATIVE_TEAM: char = 'T';
/// Relative summary graphs wrt Best across both teams
pub const SUMMARY_RELATIVE_ALL: char = 'A';

#[derive(Debug)]
/// Maintain the scoring related to a player
struct Score {
    /// The overall actions related score
    ascore: f32,
    /// Vector of time,ascoredelta
    vtimeascore_indiv: Vec<(usize, f32)>,
    /// Vector of time vs ascore cumulative
    vtimeascore_cumul: Vec<(usize,f32)>,
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
            vtimeascore_indiv: Vec::new(),
            vtimeascore_cumul: Vec::new(),
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


    fn ascore_update(&mut self, time: usize, ascoredelta: f32) {
        self.ascore += ascoredelta;
        self.vtimeascore_indiv.push((time, ascoredelta));
        self.vtimeascore_cumul.push((time, self.ascore));
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
struct Player {
    id: usize,
    score: Score,
    pos: Pos,
}

impl Player {
    fn new(playerid: usize, score: Score, pos: Pos) -> Player {
        Player {
            id: playerid,
            score: score,
            pos: pos,
        }
    }
}

#[derive(Debug)]
struct Players {
    lplayers: Vec<Player>,
    rplayers: Vec<Player>,
}

impl Players {

    fn new(lcnt: usize, rcnt: usize) -> Players {
        let mut players = Players {
            lplayers: Vec::new(),
            rplayers: Vec::new(),
        };
        for i in 0..lcnt {
            players.lplayers.push(Player::new(i, Score::default(), (99.0,99.0)));
        }
        for i in 0..rcnt {
            players.rplayers.push(Player::new(i, Score::default(), (99.0,99.0)));
        }
        return players;
    }

    /// Get the specified player
    fn get_player<'a>(&'a self, side: char, playerid: usize) -> &'a Player {
        let player = if side == entities::SIDE_L { &self.lplayers[playerid] } else { &self.rplayers[playerid] };
        return player;
    }

    /*
    #[allow(dead_code)]
    /// Get the specified player's score struct
    fn get_player_score_set<'a>(&'a self, side: char, playerid: usize) -> &'a Score {
        let player = self.get_player(side, playerid);
        return &player.score;
    }
    */

    /// Help update the score of a specific player
    fn card(&mut self, side: char, playerid: usize, card: playdata::Card) {
        if playerid >= entities::XPLAYERID_START {
            ldebug!(&format!("WARN:{}:Players:Card:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        } else {
            eprintln!("DBUG:{}:Players:Card:{}{:02}:{}", MTAG, side, playerid, card);
        }
        if side == entities::SIDE_L {
            self.lplayers[playerid].score.card = card;
        } else {
            self.rplayers[playerid].score.card = card;
        }
    }

    /// Help update the actions related score of a specific player
    fn ascore_update(&mut self, time: usize, side: char, playerid: usize, ascore: f32) {
        if playerid >= entities::XPLAYERID_START {
            ldebug!(&format!("WARN:{}:Players:Score:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        } else {
            eprintln!("DBUG:{}:Players:Score:{}{:02}:{}", MTAG, side, playerid, ascore);
        }
        if side == entities::SIDE_L {
            self.lplayers[playerid].score.ascore_update(time, ascore);
        } else {
            self.rplayers[playerid].score.ascore_update(time, ascore);
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
                player.score.kicks += 1;
            },
            AIAction::Catch => {
                stype = "Catch";
                player.score.catchs += 1;
            },
            AIAction::Tackle => {
                stype = "Tackle";
                player.score.tackles += 1;
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
        let opos = player.pos;
        if opos.0 == 99.0 && opos.1 == 99.0 {
            player.pos = npos;
            return;
        }
        let dx = npos.0-opos.0;
        let dy = npos.1-opos.1;
        let d = dx*dx + dy*dy;
        player.score.dist += d.sqrt();
        player.pos = npos;
    }

    /// Return the min and max player score for each of the teams
    fn score_minmax(&self, inc_cardscore: bool) -> ((f32,f32), (f32,f32)) {
        let mut lmax = f32::MIN;
        let mut lmin = f32::MAX;
        for i in 0..self.lplayers.len() {
            let player = &self.lplayers[i];
            if lmax < player.score.score(inc_cardscore) {
                lmax = player.score.score(inc_cardscore);
            }
            if lmin > player.score.score(inc_cardscore) {
                lmin = player.score.score(inc_cardscore);
            }
        }
        let mut rmax = f32::MIN;
        let mut rmin = f32::MAX;
        for i in 0..self.rplayers.len() {
            let player = &self.rplayers[i];
            if rmax < player.score.score(inc_cardscore) {
                rmax = player.score.score(inc_cardscore);
            }
            if rmin > player.score.score(inc_cardscore) {
                rmin = player.score.score(inc_cardscore);
            }
        }
        ((lmin,lmax), (rmin,rmax))
    }

    /// Return the max player distance traversed for each of the teams
    fn dist_max(&self) -> (f32, f32) {
        let mut lmax = f32::MIN;
        for i in 0..self.lplayers.len() {
            let player = &self.lplayers[i];
            if lmax < player.score.dist {
                lmax = player.score.dist;
            }
        }
        let mut rmax = f32::MIN;
        for i in 0..self.rplayers.len() {
            let player = &self.rplayers[i];
            if rmax < player.score.dist {
                rmax = player.score.dist;
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
    /// Contains all game actions, even same type actions which are too near in time.
    pub rawactions: Vec<ActionData>,
    /// Flag to indicate a seek was requested
    handle_deferedseek: bool,
}

impl ActionsInfo {

    pub fn new(acnt: usize, bcnt: usize) -> ActionsInfo {
        ActionsInfo {
            players: Players::new(acnt, bcnt),
            actions: Vec::new(),
            rawactions: Vec::new(),
            handle_deferedseek: false,
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

    fn summary_simple(&self, inc_cardscore: bool) {
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            eprintln!("DBUG:{}:L{:02}:{}", MTAG, player.id, player.score.score(inc_cardscore));
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            eprintln!("DBUG:{}:R{:02}:{}", MTAG, player.id, player.score.score(inc_cardscore));
        }
    }

    fn summary_asciiart(&self, inc_cardscore: bool) {
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            eprint!("DBUG:{}:L{:02}:", MTAG, player.id);
            for _j in 0..player.score.score(inc_cardscore).round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            eprint!("DBUG:{}:R{:02}:", MTAG, player.id);
            for _j in 0..player.score.score(inc_cardscore).round() as usize {
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
    pub fn summary_score_sdl(&self, sx: &mut SdlX, summarytype: char, inc_cardscore: bool) {
        // let (amax, bmax) = (20.0, 20.0);
        let ((mut lmin, mut lmax), (mut rmin, mut rmax)) = self.players.score_minmax(inc_cardscore);
        if summarytype == SUMMARY_RELATIVE_ALL {
            lmax = lmax.max(rmax);
            rmax = lmax;
            lmin = lmin.min(rmin);
            rmin = lmin;
        }
        for i in 0..self.players.lplayers.len() {
            let player = &self.players.lplayers[i];
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let lpscore = player.score.score(inc_cardscore) - lmin;
            sx.nn_fill_rect(0.05, 0.05*(i as f32 + 4.0), 0.4*(lpscore/(lmax-lmin)), 0.04)
        }
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let rpscore = player.score.score(inc_cardscore) - rmin;
            sx.nn_fill_rect(0.55, 0.05*(i as f32 + 4.0), 0.4*(rpscore/(rmax-rmin)), 0.04)
        }
    }

    /// SummaryType if 'T' => Bar relative to max in each team
    /// SummaryType if 'A' => Bar relative to max across both teams
    pub fn summary_dist_sdl(&self, sx: &mut SdlX, summarytype: char) {
        let (mut lmax, mut rmax) = self.players.dist_max();
        if summarytype == SUMMARY_RELATIVE_ALL {
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
            let yh = yh*(player.score.dist/lmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
        }
        let xs = 0.55;
        for i in 0..self.players.rplayers.len() {
            let player = &self.players.rplayers[i];
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.score.dist/rmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
        }
    }

    pub fn summary(&self, inc_cardscore: bool) {
        self.summary_asciiart(inc_cardscore);
        self.summary_simple(inc_cardscore);
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
                    self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, ppscore);
                    self.players.ascore_update(curactd.time, curactd.side, curactd.playerid, cpscore);
                } else {
                    let pscore = score.0 * score.3;
                    self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, pscore);
                    let pscore = score.0 * score.4;
                    self.players.ascore_update(curactd.time, curactd.side, curactd.playerid, pscore);
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
                    self.players.ascore_update(curactd.time, curactd.side, curactd.playerid, pscore);
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
                    self.players.ascore_update(curactd.time, curactd.side, pid, pscore);
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
                        self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, pscore);
                        return HAReturn::ContinueSearch;
                    }
                    pscore *= SCORE_GOALCHAIN_OTHERSIDE_BEYOND_IMMIDIATE_RATIO;
                    self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, pscore);
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
                self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, ppscore);
                self.players.ascore_update(curactd.time, curactd.side, curactd.playerid, cpscore);
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
                self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, ppscore);
                self.players.ascore_update(curactd.time, curactd.side, curactd.playerid, cpscore);
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
                self.players.ascore_update(curactd.time, prevactd.side, prevactd.playerid, ppscore);
                self.players.ascore_update(curactd.time, curactd.side, curactd.playerid, cpscore);
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
        if self.handle_deferedseek {
            self.skip_after_including(curactd.time);
            self.handle_deferedseek = false;
        }
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

impl ActionsInfo {

    /// Skip all action records with a time stamp, which is
    /// greater than or equal to specified timestamp/counter.
    fn skip_after_including(&mut self, timecounter: usize) {
        let sacnt = self.actions.len();
        let sracnt = self.rawactions.len();
        // Skip wrt actions
        for i in (0..self.actions.len()).rev() {
            let checkact = &self.actions[i];
            if checkact.time < timecounter {
                break;
            }
            self.actions.pop();
        }
        // Skip wrt rawactions
        for i in (0..self.rawactions.len()).rev() {
            let checkact = &self.rawactions[i];
            if checkact.time < timecounter {
                break;
            }
            self.rawactions.pop();
        }
        let eacnt = self.actions.len();
        let eracnt = self.rawactions.len();
        eprintln!("DBUG:{}:SkipAfterInc:A:{}->{}:RA:{}->{}", MTAG, sacnt, eacnt, sracnt, eracnt);
    }

    /// There need not be records in ActionsInfo for each time step in the game,
    /// So also it wont know what is the latest/current time step active, so going
    /// back relative to current time step is not directly possible currently.
    /// Also as it stands currently, it is not necessary that all entities in the
    /// program will seek in the same way. So need to use a indirect logic wrt this.
    ///
    /// NOTE: If a playdata source returns multiple actions wrt a time step and
    /// inturn if after a seek (especially backwards), if it returns from somewhere
    /// in the middle of the set of actions associated with the seeked timestamp,
    /// then the handle_deferedseek through skip_after_including may not help fully.
    pub fn seek(&mut self, _seekdelta: isize) {
        self.handle_deferedseek = true;
    }

}

#[derive(Debug, PartialEq)]
pub enum SummaryPlayerType {
    Individual,
    Cumulative,
}

impl ActionsInfo {

    /// Plot time vs ascore wrt specified side+playerid.
    /// It could either be based on individual ascores over time or cumulated ascores over time.
    /// Specify the position of the plot window ((x,y),(w,h))
    ///
    /// If yminmax is not specified, then a standard min max is used and inturn adjusted based on player's score.
    pub fn summary_player(&mut self, sx: &mut SdlX, side: char, playerid: usize, maxtime: usize, yminmax: Option<(f32, f32)>, sptype: &SummaryPlayerType, win: XRect) {
        use crate::sdlx::PlotType;
        let player = self.players.get_player(side, playerid);
        let vts;
        let mut ymin = -2.0;
        let mut ymax = 5.0;
        if yminmax.is_some() {
            let yminmax = yminmax.unwrap();
            ymin = yminmax.0;
            ymax = yminmax.1;
        }
        if *sptype == SummaryPlayerType::Individual {
            vts = &player.score.vtimeascore_indiv;
        } else {
            vts = &player.score.vtimeascore_cumul;
            if yminmax.is_none() {
                if player.score.ascore > ymax {
                    ymax = player.score.ascore;
                } else if ymin > player.score.ascore {
                    ymin = player.score.ascore;
                }
            }
        }
        let stag = format!("{}{:02}", side, playerid);
        //eprintln!("DBUG:{}:SummaryPlayer:{}{:02}:Len[{}]", MTAG, side, playerid, vts.len());
        sx.n_plot_uf(win.0.0, win.0.1, win.1.0, win.1.1, vts, 0.0, maxtime as f32, ymin, ymax, &stag, PlotType::Lines);
    }

    #[allow(dead_code)]
    pub fn summary_players_seperate(&mut self, sx: &mut SdlX, side: char, maxtime: usize, sptype: SummaryPlayerType, win: XRect) {
        let ((wx,wy), (ww,wh)) = win;
        let players = if side == entities::SIDE_L { &self.players.lplayers } else { &self.players.rplayers };
        let ph = wh/players.len() as f32;
        for pi in 0..players.len() {
            let x = wx;
            let y = wy - (pi as f32*ph);
            self.summary_player(sx, side, pi, maxtime, None, &sptype, ((x,y),(ww,ph)));
        }
    }

    pub fn summary_players(&mut self, sx: &mut SdlX, side: char, maxtime: usize, yminmax: Option<(f32, f32)>, sptype: SummaryPlayerType, win: XRect) {
        let players = if side == entities::SIDE_L { &self.players.lplayers } else { &self.players.rplayers };
        for pi in 0..players.len() {
            self.summary_player(sx, side, pi, maxtime, yminmax, &sptype, win);
        }
    }

}