//!
//! Identify pass quality
//! HanishKVC, 2022
//!
//! TODO:
//! * Track for halftime/etal and avoid providing -ve scoring
//!   due to any of these leading to shifting/switching of side
//!   wrt the kick that will follow.
//! * Allow -ve scoring to goalie, if they allow a goal to occur
//!   ie beyond the failed catch situations.
//! * Account penalties beyond cards during scoring.
//!

use std::collections::HashMap;
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

/// Filtering of player time vs score data
//pub const PLOT_TVS_FILTER: [f32;5] = [0.1,0.2,0.4,0.2,0.1];
pub const PLOT_TVS_FILTER: [f32;0] = [];


#[derive(Debug)]
/// Maintain the scoring related to a player
struct Score {
    /// The overall performance related score
    pscore: f32,
    /// Vector of time,pscoredeltas
    vtimepscore_deltas: Vec<(usize, f32)>,
    /// Vector of time vs pscore cumulative
    vtimepscore_cumul: Vec<(usize,f32)>,
    /// Historic cumulative min pscore
    hist_cumul_minpscore: f32,
    /// Historic cumulative max pscore
    hist_cumul_maxpscore: f32,
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

    fn new(pscore: f32, kicks: usize, tackles: usize, catchs: usize, dist: f32, card: playdata::Card) -> Score {
        Score {
            pscore,
            vtimepscore_deltas: Vec::new(),
            vtimepscore_cumul: Vec::new(),
            hist_cumul_maxpscore: f32::MIN,
            hist_cumul_minpscore: f32::MAX,
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

    fn pscore_update(&mut self, time: usize, pscoredelta: f32) {
        self.pscore += pscoredelta;
        if self.pscore > self.hist_cumul_maxpscore {
            self.hist_cumul_maxpscore = self.pscore;
        }
        if self.pscore < self.hist_cumul_minpscore {
            self.hist_cumul_minpscore = self.pscore;
        }
        self.vtimepscore_deltas.push((time, pscoredelta));
        self.vtimepscore_cumul.push((time, self.pscore));
    }

    fn card_score_value(card: playdata::Card) -> f32 {
        match card {
            playdata::Card::None => 0.0,
            playdata::Card::Yellow => -1.5,
            playdata::Card::Red => -3.0,
        }
    }

    fn card_issued(&mut self, time: usize, card: playdata::Card) {
        self.card = card.clone();
        self.pscore_update(time, Self::card_score_value(card));
    }

    /// Returns the performance score as is, or after substracting latest
    /// card penalty.
    ///
    /// NOTE: It will only avoid the last / latest card penalty. If multiple
    /// card penalties were issued, then the previous ones wont be adjusted
    /// for.
    fn score(&self, inc_cardscore: bool) -> f32 {
        let adjustcardscore;
        if inc_cardscore {
            adjustcardscore = 0.0
        } else {
            adjustcardscore = Self::card_score_value(self.card.clone());
        }
        return self.pscore - adjustcardscore;
    }

}


type Pos = (f32, f32);

#[derive(Debug)]
struct Player {
    id: String,
    score: Score,
    pos: Pos,
}

impl Player {
    fn new(playerid: String, score: Score, pos: Pos) -> Player {
        Player {
            id: playerid,
            score: score,
            pos: pos,
        }
    }
}


#[derive(Debug)]
struct Teams {
    lpids: Vec<String>,
    rpids: Vec<String>,
    lplayers: HashMap<String, Player>,
    rplayers: HashMap<String, Player>,
    lballtime: f32,
    rballtime: f32,
}

impl Teams {

    fn new(lplayers: &Vec<&str>, rplayers: &Vec<&str>) -> Teams {
        let mut teams = Teams {
            lpids: Vec::new(),
            rpids: Vec::new(),
            lplayers: HashMap::new(),
            rplayers: HashMap::new(),
            lballtime: 0.0,
            rballtime: 0.0,
        };
        for pid in lplayers {
            teams.lpids.push(pid.to_string());
            teams.lplayers.insert(pid.to_string(), Player::new(pid.to_string(), Score::default(), (99.0,99.0)));
        }
        for pid in rplayers {
            teams.rpids.push(pid.to_string());
            teams.rplayers.insert(pid.to_string(), Player::new(pid.to_string(), Score::default(), (99.0,99.0)));
        }
        return teams;
    }

    /// Get the specified player
    fn get_player<'a>(&'a self, side: char, playerid: &str) -> &'a Player {
        let player = if side == entities::SIDE_L { self.lplayers.get(playerid) } else { self.rplayers.get(playerid) };
        return player.unwrap();
    }

    /// Get a mutable reference to specified player
    fn get_player_mut<'a>(&'a mut self, side: char, playerid: &str) -> &'a mut Player {
        let player = if side == entities::SIDE_L { self.lplayers.get_mut(playerid) } else { self.rplayers.get_mut(playerid) };
        return player.unwrap();
    }

    /// Help update the score of a specific player, based on card issued if any
    fn card_issued(&mut self, time: usize, side: char, playerid: &str, card: playdata::Card) {
        if playerid.starts_with(entities::XPLAYERID_START) {
            ldebug!(&format!("WARN:{}:Players:Card:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        } else {
            eprintln!("DBUG:{}:Players:Card:{}{:02}:{}", MTAG, side, playerid, card);
        }
        let player = self.get_player_mut(side, playerid);
        player.score.card_issued(time, card);
    }

    /// Help update the performance related score of a specific player
    /// Could be used for actions based scoring or so...
    fn pscore_update(&mut self, time: usize, side: char, playerid: &str, pscoredelta: f32) {
        if playerid.starts_with(entities::XPLAYERID_START) {
            ldebug!(&format!("WARN:{}:Players:Score:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        } else {
            eprintln!("DBUG:{}:Players:Score:{}{:02}:{}", MTAG, side, playerid, pscoredelta);
        }
        let player = self.get_player_mut(side, playerid);
        player.score.pscore_update(time, pscoredelta);
    }

    /// Help update the count wrt specified action of a specific player
    fn count_increment(&mut self, side: char, playerid: &str, atype: AIAction) {
        if playerid.starts_with(entities::XPLAYERID_START) {
            ldebug!(&format!("WARN:{}:Players:CountInc:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        }
        let player = self.get_player_mut(side, playerid);
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

    fn dist_update_from_pos(&mut self, side: char, playerid: &str, npos: Pos) {
        if playerid.starts_with(entities::XPLAYERID_START) {
            ldebug!(&format!("WARN:{}:Players:DistUpdateFromPos:SpecialPlayerId:{}{:02}:Ignoring...", MTAG, side, playerid));
            return;
        }
        let player = self.get_player_mut(side, playerid);
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

    /// Return the min and max player score for each of the teams, based
    /// on the historic values seen wrt player performance scoring.
    ///
    /// NOTE: This always includes the actions based scoring as well as
    /// card based scoring.
    fn score_hist_cumul_minmax(&self) -> ((f32,f32), (f32,f32)) {
        let mut lmax = f32::MIN;
        let mut lmin = f32::MAX;
        for player in &self.lplayers {
            if lmax < player.1.score.hist_cumul_maxpscore {
                lmax = player.1.score.hist_cumul_maxpscore;
            }
            if lmin > player.1.score.hist_cumul_minpscore {
                lmin = player.1.score.hist_cumul_minpscore;
            }
        }
        let mut rmax = f32::MIN;
        let mut rmin = f32::MAX;
        for player in &self.rplayers {
            if rmax < player.1.score.hist_cumul_maxpscore {
                rmax = player.1.score.hist_cumul_maxpscore;
            }
            if rmin > player.1.score.hist_cumul_minpscore {
                rmin = player.1.score.hist_cumul_minpscore;
            }
        }
        ((lmin,lmax), (rmin,rmax))
    }

    /// Return the min and max player score for each of the teams, based on
    /// the current perf scoring wrt each player.
    fn score_minmax(&self, inc_cardscore: bool) -> ((f32,f32), (f32,f32)) {
        let mut lmax = f32::MIN;
        let mut lmin = f32::MAX;
        for player in &self.lplayers {
            if lmax < player.1.score.score(inc_cardscore) {
                lmax = player.1.score.score(inc_cardscore);
            }
            if lmin > player.1.score.score(inc_cardscore) {
                lmin = player.1.score.score(inc_cardscore);
            }
        }
        let mut rmax = f32::MIN;
        let mut rmin = f32::MAX;
        for player in &self.rplayers {
            if rmax < player.1.score.score(inc_cardscore) {
                rmax = player.1.score.score(inc_cardscore);
            }
            if rmin > player.1.score.score(inc_cardscore) {
                rmin = player.1.score.score(inc_cardscore);
            }
        }
        ((lmin,lmax), (rmin,rmax))
    }

    /// Return the max player distance traversed for each of the teams
    fn dist_max(&self) -> (f32, f32) {
        let mut lmax = f32::MIN;
        for player in &self.lplayers {
            if lmax < player.1.score.dist {
                lmax = player.1.score.dist;
            }
        }
        let mut rmax = f32::MIN;
        for player in &self.rplayers {
            if rmax < player.1.score.dist {
                rmax = player.1.score.dist;
            }
        }
        (lmax, rmax)
    }

    /// If ball actions remain on the same side, give ball time to the side
    fn balltime_update(&mut self, prevactd: &ActionData, curactd: &ActionData) {
        if curactd.side == prevactd.side {
            let dtime = (curactd.time - prevactd.time) as f32;
            if curactd.side == entities::SIDE_L {
                self.lballtime += dtime;
            } else {
                self.rballtime += dtime;
            }
        }
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
    playerid: String,
    pub pos: (f32, f32),
    action: AIAction,
}

impl ActionData {

    pub fn new(time: usize, side: char, playerid: String, pos: (f32,f32), action: AIAction) -> ActionData {
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
    teams: Teams,
    /// Contains significant game actions
    actions: Vec<ActionData>,
    /// Contains all game actions, even same type actions which are too near in time.
    pub rawactions: Vec<ActionData>,
    /// Flag to indicate a seek was requested
    handle_deferedseek: bool,
}

impl ActionsInfo {

    pub fn new(lplayers: &Vec<&str>, rplayers: &Vec<&str>) -> ActionsInfo {
        ActionsInfo {
            teams: Teams::new(lplayers, rplayers),
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

    fn summary_score_simple(&self, inc_cardscore: bool) {
        for pid in &self.teams.lpids {
            let player = self.teams.lplayers.get(pid).unwrap();
            eprintln!("DBUG:{}:L{:02}:{}", MTAG, player.id, player.score.score(inc_cardscore));
        }
        for pid in &self.teams.rpids {
            let player = self.teams.rplayers.get(pid).unwrap();
            eprintln!("DBUG:{}:R{:02}:{}", MTAG, player.id, player.score.score(inc_cardscore));
        }
    }

    fn summary_score_asciiart(&self, inc_cardscore: bool) {
        for pid in &self.teams.lpids {
            let player = self.teams.lplayers.get(pid).unwrap();
            eprint!("DBUG:{}:L{:02}:", MTAG, player.id);
            for _j in 0..player.score.score(inc_cardscore).round() as usize {
                eprint!("#");
            }
            eprintln!();
        }
        for pid in &self.teams.rpids {
            let player = self.teams.rplayers.get(pid).unwrap();
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
        let ((mut lmin, mut lmax), (mut rmin, mut rmax)) = self.teams.score_minmax(inc_cardscore);
        if summarytype == SUMMARY_RELATIVE_ALL {
            lmax = lmax.max(rmax);
            rmax = lmax;
            lmin = lmin.min(rmin);
            rmin = lmin;
        }
        let mut i = 0;
        for pid in &self.teams.lpids {
            let player = self.teams.lplayers.get(pid).unwrap();
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let lpscore = player.score.score(inc_cardscore) - lmin;
            sx.nn_fill_rect(0.05, 0.05*(i as f32 + 4.0), 0.4*(lpscore/(lmax-lmin)), 0.04);
            i += 1;
        }
        i = 0;
        for pid in &self.teams.rpids {
            let player = self.teams.rplayers.get(pid).unwrap();
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let rpscore = player.score.score(inc_cardscore) - rmin;
            sx.nn_fill_rect(0.55, 0.05*(i as f32 + 4.0), 0.4*(rpscore/(rmax-rmin)), 0.04);
            i += 1;
        }
    }

    /// SummaryType if 'T' => Bar relative to max in each team
    /// SummaryType if 'A' => Bar relative to max across both teams
    pub fn summary_dist_sdl(&self, sx: &mut SdlX, summarytype: char) {
        let (mut lmax, mut rmax) = self.teams.dist_max();
        if summarytype == SUMMARY_RELATIVE_ALL {
            lmax = lmax.max(rmax);
            rmax = lmax;
        }
        let xs = 0.05;
        let xw = 0.4;
        let xu = xw/self.teams.lplayers.len() as f32;
        let yb = 0.8;
        let yh = 0.1;
        let mut i = 0;
        for pid in &self.teams.lpids {
            let player = self.teams.lplayers.get(pid).unwrap();
            sx.wc.set_draw_color(Color::RGBA(200, 0, 0, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.score.dist/lmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
            i += 1;
        }
        let xs = 0.55;
        i = 0;
        for pid in &self.teams.rpids {
            let player = self.teams.rplayers.get(pid).unwrap();
            sx.wc.set_draw_color(Color::RGBA(0, 0, 200, 40));
            sx.wc.set_blend_mode(BlendMode::Blend);
            let x = xs + (i as f32 * xu);
            let yh = yh*(player.score.dist/rmax);
            sx.nn_fill_rect(x, yb, xu*0.9, yh);
            i += 1;
        }
    }

    pub fn summary(&self, inc_cardscore: bool) {
        self.summary_score_asciiart(inc_cardscore);
        self.summary_score_simple(inc_cardscore);
    }

}


/// Helps tell what should one do after the handle helper returns
pub enum HAReturn {
    /// Continue searching/checking the history further back, cas not yet done
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
                    self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, ppscore);
                    self.teams.pscore_update(curactd.time, curactd.side, &curactd.playerid, cpscore);
                } else {
                    let pscore = score.0 * score.3;
                    self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, pscore);
                    let pscore = score.0 * score.4;
                    self.teams.pscore_update(curactd.time, curactd.side, &curactd.playerid, pscore);
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
                    self.teams.pscore_update(curactd.time, curactd.side, &curactd.playerid, pscore);
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
                    if curactd.time == prevactd.time {
                        // In case playdata source doesnt filter out duplicate entries wrt a given goal,
                        // Skip duplicate goal action data
                        ldebug!(&format!("DBUG:{}:HandleGoal:Goal{}->Goal{} shouldnt occur, ignoring...", MTAG, prevactd, curactd));
                        return HAReturn::Done(false);
                    }
                    panic!("DBUG:{}:HandleGoal:Goal{}->Goal{} shouldnt occur", MTAG, prevactd, curactd);
                }
                return HAReturn::Done(true);
            },
            AIAction::Kick | AIAction::Tackle | AIAction::Catch => {
                if lookbackcnt <= 1 {
                    if curactd.playerid.starts_with(entities::XPLAYERID_START) {
                        // Fill the player responsible for the goal bcas
                        // One doesnt know whether a kick will become a goal or not
                        // at the time of the kick, in general.
                        curactd.playerid = prevactd.playerid.clone();
                        eprintln!("INFO:{}:HandleGoal:{}:Player updated; PrevAction {}", MTAG, curactd, prevactd);
                    } else {
                        eprintln!("WARN:{}:HandleGoal:{}:Player already set; PrevAction {}", MTAG, curactd, prevactd);
                    }
                }
                if prevactd.side == curactd.side {
                    // A successful goal
                    // Nearest player scores more compared to farther players, wrt the chain of actions leading to the goal
                    let pscore = score.0 * score.1 * (1.0/lookbackcnt as f32);
                    let pid = if lookbackcnt <= 1 { &curactd.playerid } else { &prevactd.playerid };
                    self.teams.pscore_update(curactd.time, curactd.side, pid, pscore);
                    if (curactd.time - prevactd.time) > GOAL_CHAIN_TIME {
                        return HAReturn::Done(true);
                    }
                    return HAReturn::ContinueSearch;
                } else {
                    let mut pscore = score.0 * score.3 * (1.0/lookbackcnt as f32);
                    if lookbackcnt <= 1 {
                        // a self goal !?!
                        curactd.playerid = format!("{}-{}", entities::XPLAYERID_OOPS_OTHERSIDE_START, curactd.playerid);
                        eprintln!("WARN:{}:HandleGoal:{}:Player updated - SelfGoal; PrevAction {}", MTAG, curactd, prevactd);
                        if prevactd.action == AIAction::Catch {
                            pscore *= SCORE_GOALIE_MISSED_CATCH_PENALTY_RATIO;
                        }
                        self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, pscore);
                        return HAReturn::ContinueSearch;
                    }
                    pscore *= SCORE_GOALCHAIN_OTHERSIDE_BEYOND_IMMIDIATE_RATIO;
                    self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, pscore);
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
                self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, ppscore);
                self.teams.pscore_update(curactd.time, curactd.side, &curactd.playerid, cpscore);
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
                self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, ppscore);
                self.teams.pscore_update(curactd.time, curactd.side, &curactd.playerid, cpscore);
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
                self.teams.pscore_update(curactd.time, prevactd.side, &prevactd.playerid, ppscore);
                self.teams.pscore_update(curactd.time, curactd.side, &curactd.playerid, cpscore);
                return HAReturn::Done(true);
            },
            AIAction::Catch | AIAction::Goal => {
                // Shouldnt occur (there should be a kick after these), however if it occurs, ignore for now
                eprintln!("WARN:{}:HandleCatch:Catch/Goal{}->Catch{} shouldnt occur, ignoring...", MTAG, prevactd, curactd);
                return HAReturn::Done(false);
            },
        }
    }

    fn balltime_update(&mut self, prevactd: &ActionData, curactd: &ActionData, lookbackcnt: usize) {
        if lookbackcnt != 1 {
            return;
        }
        self.teams.balltime_update(&prevactd, &curactd);
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
                    self.balltime_update(&prevactd, &curactd, lookbackcnt);
                    if let HAReturn::Done(save) = self.handle_kick(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Tackle => {
                    self.balltime_update(&prevactd, &curactd, lookbackcnt);
                    if let HAReturn::Done(save) = self.handle_tackle(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Catch => {
                    self.balltime_update(&prevactd, &curactd, lookbackcnt);
                    if let HAReturn::Done(save) = self.handle_catch(&mut curactd, &prevactd) {
                        bupdate_actions = save;
                        break;
                    }
                },
                AIAction::Goal => {
                    self.balltime_update(&prevactd, &curactd, lookbackcnt);
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
            self.teams.dist_update_from_pos(curactd.side, &curactd.playerid, curactd.pos);
        }
        if bupdate_actions {
            self.actions.push(curactd.clone());
        }
        if bupdate_rawactions {
            self.teams.count_increment(curactd.side, &curactd.playerid, curactd.action.clone());
            ldebug!(&format!("DBUG:{}:RawActions:{}", MTAG, curactd));
            self.rawactions.push(curactd);
        }
    }

}

impl ActionsInfo {

    pub fn handle_card(&mut self, time: usize, side: char, playerid: &str, card: playdata::Card) {
        self.teams.card_issued(time, side, playerid, card);
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
    ScoreDeltas,
    ScoreCumulative,
}

impl ActionsInfo {

    /// Plot time vs pscore wrt specified side+playerid.
    /// It could either be based on individual pscores deltas/changes over time or cumulated pscores over time.
    /// Specify the position of the plot window ((x,y),(w,h))
    ///
    /// If yminmax is not specified, then a standard min max is used and inturn adjusted based on player's score.
    pub fn summary_tvs_player(&mut self, sx: &mut SdlX, side: char, playerid: &str, maxtime: usize, yminmax: Option<(f32, f32)>, sptype: &SummaryPlayerType, win: XRect) {
        use crate::sdlx::PlotType;
        let player = self.teams.get_player(side, playerid);
        let vts;
        let mut ymin = -2.0;
        let mut ymax = 5.0;
        if yminmax.is_some() {
            let yminmax = yminmax.unwrap();
            ymin = yminmax.0;
            ymax = yminmax.1;
        }
        if *sptype == SummaryPlayerType::ScoreDeltas {
            vts = &player.score.vtimepscore_deltas;
        } else {
            vts = &player.score.vtimepscore_cumul;
            if yminmax.is_none() {
                if player.score.pscore > ymax {
                    ymax = player.score.pscore;
                } else if ymin > player.score.pscore {
                    ymin = player.score.pscore;
                }
            }
        }
        let stag = format!("{}{:02}", side.to_uppercase(), playerid);
        //eprintln!("DBUG:{}:SummaryPlayer:{}{:02}:Len[{}]", MTAG, side, playerid, vts.len());
        let weights =if PLOT_TVS_FILTER.len() > 0 {
            Some(PLOT_TVS_FILTER.to_vec())
        } else {
            None
        };
        sx.n_plot_uf(win.0.0, win.0.1, win.1.0, win.1.1, vts, 0.0, maxtime as f32, ymin, ymax, weights, &stag, PlotType::Lines);
    }

    #[allow(dead_code)]
    pub fn summary_tvs_givenside_independent(&mut self, sx: &mut SdlX, side: char, maxtime: usize, sptype: &SummaryPlayerType, win: XRect) {
        let ((wx,wy), (ww,wh)) = win;
        let pids = if side == entities::SIDE_L { self.teams.lpids.clone() } else { self.teams.rpids.clone() };
        let ph = wh/pids.len() as f32;
        for pi in 0..pids.len() {
            let pid = &pids[pi];
            let x = wx;
            let y = wy - (pi as f32*ph);
            self.summary_tvs_player(sx, side, pid, maxtime, None, &sptype, ((x,y),(ww,ph)));
        }
    }

    /// Display time vs score wrt players of the specified side
    pub fn summary_tvs_givenside_shared(&mut self, sx: &mut SdlX, side: char, maxtime: usize, yminmax: Option<(f32, f32)>, sptype: &SummaryPlayerType, win: XRect) {
        let pids = if side == entities::SIDE_L { self.teams.lpids.clone() } else { self.teams.rpids.clone() };
        for pi in 0..pids.len() {
            let pid = &pids[pi];
            self.summary_tvs_player(sx, side, pid, maxtime, yminmax, &sptype, win);
        }
    }

    /// Display time vs score wrt players of both side
    ///
    /// TODO: Currently the minmax is calculated/got wrt the historic cumulative data set and not historic deltas data set
    /// So if in future, we want to plot wrt score deltas rather than cumulative score, then one needs to get the minmax
    /// wrt score deltas.
    pub fn summary_tvs(&mut self, sx: &mut SdlX, maxtime: usize, sptype: &SummaryPlayerType, win: XRect, summarytype: char) {
        let ((mut lmin,mut lmax), (mut rmin,mut rmax)) = self.teams.score_hist_cumul_minmax();
        //eprintln!("DBUG:{}:SummaryTVS:MinMax:{},{}:{},{}", MTAG, lmin, lmax, rmin, rmax);
        if summarytype == SUMMARY_RELATIVE_ALL {
            lmin = lmin.min(rmin);
            lmax = lmax.max(rmax);
            rmin = lmin;
            rmax = lmax;
        }
        let winxy = win.0;
        let winwh = win.1;
        let winwby2 = winwh.0/2.0;
        let lwin = (winxy,(winwby2,winwh.1));
        self.summary_tvs_givenside_shared(sx, entities::SIDE_L, maxtime, Some((lmin,lmax)), sptype, lwin);
        let rwin = ((winxy.0+winwby2, winxy.1),(winwby2, winwh.1));
        self.summary_tvs_givenside_shared(sx, entities::SIDE_R, maxtime, Some((rmin,rmax)), sptype, rwin);
    }

}

impl ActionsInfo {

    pub fn show_ballpossession(&self, sx: &mut SdlX) {
        let lposs = ((self.teams.lballtime/(self.teams.lballtime + self.teams.rballtime))*100.0).round();
        let rposs = 100.0-lposs;
        let lpos = entities::MSG_LBALLPOSS_POS;
        let rpos = entities::MSG_RBALLPOSS_POS;
        sx.n_string(lpos.0, lpos.1, &lposs.to_string(), Color::WHITE);
        sx.n_string(rpos.0, rpos.1, &rposs.to_string(), Color::WHITE);
    }

}
