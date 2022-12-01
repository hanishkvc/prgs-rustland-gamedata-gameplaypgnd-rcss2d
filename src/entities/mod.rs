//!
//! The entities in the playground
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::{SdlX, XRect};
use crate::playdata::{PlayUpdate, GameState};
use crate::proc::actions::{ActionsInfo, ActionData, AIAction};

pub const SIDE_L: char = 'l';
pub const SIDE_R: char = 'r';
pub const XPLAYERID_START: usize = 0x1000;
pub const XPLAYERID_UNKNOWN: usize = 0x1001;
pub const XPLAYERID_OOPS_OTHERSIDE_START: usize = 0x8000;

const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

pub const BASE_SCREEN_WIDTH: u32 = 1024;
pub const BASE_SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);

pub const FRAMES_PER_SEC: usize = 24;

pub const PITCH_RECT: XRect = ((0.03,0.04), (0.97,0.96));

pub const MSG_SCORE_POS: (f32,f32)      = (0.01,0.01);
pub const MSG_STIME_POS: (f32,f32)      = (0.50,0.01);
pub const MSG_FPS_POS: (f32,f32)        = (0.80,0.01);
pub const MSG_GAME_POS: (f32,f32)       = (0.01,0.98);
pub const MSG_UNKNOWN_POS: (f32,f32)    = (0.50,0.98);
pub const MSG_TIMED_POS: (f32, f32)     = (0.01,0.08);

const MSG_TIMED_NUMFRAMES: isize = 40;

pub fn screen_color_bg_rel(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: SCREEN_COLOR_BG.r.saturating_add(r),
        g: SCREEN_COLOR_BG.g.saturating_add(g),
        b: SCREEN_COLOR_BG.b.saturating_add(b),
        a: SCREEN_COLOR_BG.a,
    }
}

type _PosType = f32;

pub mod gentity;
pub mod team;
pub mod objects;
use objects::Ball;
use objects::FixedPosMessage;
pub mod simobjs;
use simobjs::VirtBall;


#[derive(Debug)]
/// Manage the entities in the playground.
pub(crate) struct PGEntities<'a> {
    /// The frames per second, wrt movements on the screen
    fps: f32,
    /// The fixed position based messages on the screen
    vfpmsgs: Vec<FixedPosMessage>,
    /// Whether to show the ball or not
    pub showball: bool,
    /// The ball in the playground
    ball: Ball<'a>,
    /// One of the two teams in the playground
    lteam: team::Team<'a>,
    /// The other team in the playground.
    rteam: team::Team<'a>,
    /// The pitch boundry within the screen, in normalised 0.0-1.0 space.
    pitch: XRect,
    /// If extra pitch markers should be shown or not.
    pub showxtrapitchmarkers: bool,
    /// Info from Data
    pub actionsinfo: ActionsInfo,
    /// A virtual interpolated ball
    /// The graphical object representing virtual ball on the screen
    virtballg: Ball<'a>,
    /// The VirtBall object containing the key position datas wrt the ball.
    virtballd: Option<VirtBall>,
    /// Timed Msg
    pub timedmsg: FixedPosMessage,
}

impl<'a> PGEntities<'a> {

    /// Create a playground instance with
    /// * pitch: the dimensions of the pitch with in the screen,
    ///   in normalised 0.0-1.0 space.
    /// * [l/r]nplayers: the number of players on both sides.
    /// * font: the font used for creating the cached text image datas if any
    ///
    /// The following fixed position messages are supported on the screen
    /// * score: Give the current score, if any.
    /// * stime: Provide any time related info wrt the game.
    /// * game: show any game related messages.
    pub fn new(pitch: XRect, lnplayers: i32, rnplayers: i32, fps: f32, font: &'a Font) -> PGEntities<'a> {
        let mut vfpmsgs = Vec::new();
        let scoremsg = FixedPosMessage::new("score", MSG_SCORE_POS, false, -1);
        vfpmsgs.push(scoremsg);
        let mut stimemsg = FixedPosMessage::new("stime", MSG_STIME_POS, false, -1);
        stimemsg.config(Some(true), None);
        vfpmsgs.push(stimemsg);
        let gamemsg = FixedPosMessage::new("game", MSG_GAME_POS, false, -1);
        vfpmsgs.push(gamemsg);
        let unknownmsg = FixedPosMessage::new("unknown", MSG_UNKNOWN_POS, false, -1);
        vfpmsgs.push(unknownmsg);
        let mut timedmsg = FixedPosMessage::new("timedmsg", MSG_TIMED_POS, true, MSG_TIMED_NUMFRAMES);
        timedmsg.update_direct("");
        PGEntities {
            fps: fps,
            vfpmsgs: vfpmsgs,
            ball: Ball::new(font),
            showball: true,
            virtballg: Ball::new(font),
            virtballd: None,
            lteam: team::Team::new("lteam", Color::RED, lnplayers, font),
            rteam: team::Team::new("rteam", Color::BLUE, rnplayers, font),
            pitch: pitch,
            showxtrapitchmarkers: true,
            actionsinfo: ActionsInfo::new(lnplayers as usize, rnplayers as usize),
            timedmsg: timedmsg,
        }
    }

    pub fn fps(&self) -> f32 {
        return self.fps;
    }

    /// Allow one to increase or decrease the fps, relative to the current fps.
    pub fn fps_adjust(&mut self, ratio: f32) -> f32 {
        self.fps *= ratio;
        return self.fps;
    }

    /// Allow contents of the playground to be updated, based on got play data.
    ///
    /// pu: contains the play data with info for objects on the playground.
    ///
    /// babsolute (for objects that move)
    /// * if true, sets the object to the given position immidiately,
    /// * if false, it uses multi-frame interpolated updating of the object
    ///   to the given position.
    ///   * inframes - specifies as to in how many frames the object should
    ///     be moved to the new location being specified.
    ///
    /// Handle Game State info (Currently Goal)
    ///
    pub fn update(&mut self, pu: PlayUpdate, babsolute: bool, inframes: f32) {
        for fpmsg in &mut self.vfpmsgs {
            fpmsg.update(&pu.msgs);
        }
        if self.virtballd.is_some() {
            let virtball = self.virtballd.as_mut().unwrap();
            let bpos = virtball.next_record(pu.timecounter);
            self.virtballg.update(bpos, babsolute, inframes);
        }
        self.ball.update(pu.ball, babsolute, inframes);
        self.lteam.update(pu.timecounter, pu.lteamcoded, babsolute, inframes, &mut self.actionsinfo);
        self.rteam.update(pu.timecounter, pu.rteamcoded, babsolute, inframes, &mut self.actionsinfo);
        match pu.state {
            GameState::Goal(side)=> {
                self.actionsinfo.handle_action(ActionData::new(pu.timecounter, side, XPLAYERID_UNKNOWN, pu.ball, AIAction::Goal))
            },
            _ => {

            }
        }
    }

    /// If using interpolated updating of object positions,
    /// request them to generate their next interpolated position.
    pub fn next_frame(&mut self) {
        if self.virtballd.is_some() {
            self.virtballg.next_frame();
        }
        self.ball.next_frame();
        self.lteam.next_frame();
        self.rteam.next_frame();
    }

    /// Draw the pitch on the screen, along with the boundries and any markers.
    fn draw_pitch(&self, sx: &mut SdlX) {
        //let inbtwcolor = Color::RGB(230, 230, 230);
        let inbtwcolor = Color::WHITE;
        let ((nx1,ny1),(nx2,ny2)) = self.pitch;
        sx.nn_thick_line(nx1, ny1, nx2, ny1, 0.002, Color::WHITE);
        sx.nn_thick_line(nx1, ny1, nx1, ny2, 0.004, Color::WHITE);
        sx.nn_thick_line(nx1, ny2, nx2, ny2, 0.002, Color::WHITE);
        sx.nn_thick_line(nx2, ny1, nx2, ny2, 0.004, Color::WHITE);
        sx.nn_line(0.50, 0.02, 0.50, 0.98, inbtwcolor); // Center vertical
        let leftmidx = nx1-0.01;
        sx.nn_line(leftmidx, 0.40, leftmidx, 0.60, inbtwcolor); // Left mid
        let rightmidx = nx2+0.01;
        sx.nn_line(rightmidx, 0.40, rightmidx, 0.60, inbtwcolor); // Right mid
        // Additional markers
        if self.showxtrapitchmarkers {
            sx.nn_line(0.25, 0.48, 0.25, 0.52, inbtwcolor); // Horizontal left quarter
            sx.nn_line(0.75, 0.48, 0.75, 0.52, inbtwcolor); // Horizontal right quarter
            sx.nn_line(0.48, 0.25, 0.52, 0.25, inbtwcolor); // Vertical top quarter
            sx.nn_line(0.48, 0.75, 0.52, 0.75, inbtwcolor); // Vertical bottom quarter
            sx.nn_line(0.49, 0.50, 0.51, 0.50, inbtwcolor); // Center horizontal
        }
    }

    /// Draw all the objects in the playground.
    pub fn draw(&mut self, sx: &mut SdlX) {
        self.draw_pitch(sx);
        for fpmsg in &mut self.vfpmsgs {
            fpmsg.draw(sx);
        }
        self.lteam.draw(sx);
        self.rteam.draw(sx);
        if self.showball {
            self.ball.draw(sx);
        }
        if self.virtballd.is_some() {
            self.virtballg.draw(sx);
        }
        self.timedmsg.draw(sx);
    }

}

impl<'a> PGEntities<'a> {

    pub fn adjust_members(&mut self, virtball_fname: &str) {
        if virtball_fname.len() > 0 {
            self.virtballd = Some(VirtBall::new(virtball_fname));
            self.virtballg.set_color(Color::BLACK);
        }
        self.lteam.adjust_players(0x0e); //9
        self.rteam.adjust_players(0x0e); //3
    }

    pub fn toggle_bshowstamina(&mut self) {
        let lshow = self.lteam.toggle_bshowstamina();
        let rshow = self.rteam.toggle_bshowstamina();
        if lshow && rshow {
            self.timedmsg.update_direct("Stamina:Show");
        } else {
            self.timedmsg.update_direct("Stamina:Hide");
        }
    }

    pub fn toggle_bshowactions(&mut self) {
        let lshow = self.lteam.toggle_bshowactions();
        let rshow = self.rteam.toggle_bshowactions();
        if lshow && rshow {
            self.timedmsg.update_direct("Actions:Show");
        } else {
            self.timedmsg.update_direct("Actions:Hide");
        }
    }

    pub fn toggle_bshowcards(&mut self) {
        let lshow = self.lteam.toggle_bshowcards();
        let rshow = self.rteam.toggle_bshowcards();
        if lshow && rshow {
            self.timedmsg.update_direct("Cards:Show");
        } else {
            self.timedmsg.update_direct("Cards:Hide");
        }
    }

    pub fn seek(&mut self, seekdelta: isize) {
        if self.virtballd.is_some() {
            self.virtballd.as_mut().unwrap().seek(seekdelta);
        }
        self.actionsinfo.seek(seekdelta);
    }

    pub fn save_virtball_csv(&mut self) {
        let mut sdata = String::new();
        let actions = &self.actionsinfo.rawactions;
        for i in 0..actions.len() {
            let action = &actions[i];
            sdata.push_str(&format!("{},{},{}\n", action.time, action.pos.0, action.pos.1));
        }
        std::fs::write("/tmp/virtball.csv", sdata).unwrap();
    }

}
