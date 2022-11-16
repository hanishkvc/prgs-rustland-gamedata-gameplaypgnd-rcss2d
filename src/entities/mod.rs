//!
//! The entities in the playground
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::{SdlX, XRect};
use crate::playdata::PlayUpdate;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;
const ENTITY_RADIUS: i16 = 8;

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);

pub const FRAMES_PER_SEC: usize = 24;

pub const PITCH_RECT: XRect = ((0.03,0.04), (0.97,0.96));

pub const MSG_SCORE_POS: (f32,f32) = (0.01,0.01);
pub const MSG_STIME_POS: (f32,f32) = (0.90,0.01);
pub const MSG_GAME_POS: (f32,f32) = (0.01,0.98);
pub const MSG_UNKNOWN_POS: (f32,f32) = (0.50,0.98);

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
    ateam: team::Team<'a>,
    /// The other team in the playground.
    bteam: team::Team<'a>,
    /// The pitch boundry within the screen, in normalised 0.0-1.0 space.
    pitch: XRect,
    /// If extra pitch markers should be shown or not.
    pub showxtrapitchmarkers: bool,
}

impl<'a> PGEntities<'a> {

    /// Create a playground instance with
    /// * pitch: the dimensions of the pitch with in the screen,
    ///   in normalised 0.0-1.0 space.
    /// * [a/b]nplayers: the number of players on both sides.
    /// * font: the font used for creating the cached text image datas if any
    ///
    /// The following fixed position messages are supported on the screen
    /// * score: Give the current score, if any.
    /// * stime: Provide any time related info wrt the game.
    /// * game: show any game related messages.
    pub fn new(pitch: XRect, anplayers: i32, bnplayers: i32, font: &'a Font) -> PGEntities<'a> {
        let mut vfpmsgs = Vec::new();
        let scoremsg = FixedPosMessage::new("score", MSG_SCORE_POS, false, -1);
        vfpmsgs.push(scoremsg);
        let stimemsg = FixedPosMessage::new("stime", MSG_STIME_POS, false, -1);
        vfpmsgs.push(stimemsg);
        let gamemsg = FixedPosMessage::new("game", MSG_GAME_POS, false, -1);
        vfpmsgs.push(gamemsg);
        let unknownmsg = FixedPosMessage::new("unknown", MSG_UNKNOWN_POS, false, -1);
        vfpmsgs.push(unknownmsg);
        PGEntities {
            fps: FRAMES_PER_SEC as f32,
            vfpmsgs: vfpmsgs,
            ball: Ball::new(font),
            showball: true,
            ateam: team::Team::new("ateam", Color::RED, anplayers, font),
            bteam: team::Team::new("bteam", Color::BLUE, bnplayers, font),
            pitch: pitch,
            showxtrapitchmarkers: true,
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
    pub fn update(&mut self, pu: PlayUpdate, babsolute: bool, inframes: f32) {
        for fpmsg in &mut self.vfpmsgs {
            fpmsg.update(&pu.msgs);
        }
        self.ball.update(pu.ball, babsolute, inframes);
        self.ateam.update(pu.ateamfcoded, babsolute, inframes);
        self.bteam.update(pu.bteamfcoded, babsolute, inframes);
    }

    /// If using interpolated updating of object positions,
    /// request them to generate their next interpolated position.
    pub fn next_frame(&mut self) {
        self.ball.next_frame();
        self.ateam.next_frame();
        self.bteam.next_frame();
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
        self.ateam.draw(sx);
        self.bteam.draw(sx);
        if self.showball {
            self.ball.draw(sx);
        }
    }

}
