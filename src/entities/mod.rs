//!
//! A entity in the playground
//! HanishKVC, 2022
//!

use sdl2::{pixels::Color, render::WindowCanvas, rect::Rect, ttf::Font};
use sdl2::render::{TextureCreator, Texture};
use sdl2::video::WindowContext;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);


pub fn screen_color_bg_rel(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: SCREEN_COLOR_BG.r+r,
        g: SCREEN_COLOR_BG.g+g,
        b: SCREEN_COLOR_BG.b+b,
        a: SCREEN_COLOR_BG.a,
    }
}


pub struct Entity<'a> {
    _id: String,
    pos: (i32, i32),
    color: Color,
    onscreen: bool,
    idtx: Texture<'a>,
}

impl<'a> Entity<'a> {

    pub fn new(id: &str, pos: (i32, i32), color: Color, font: &Font, tc: &'a TextureCreator<WindowContext>) -> Entity<'a> {
        let ts = font.render(id).blended(Color::WHITE).unwrap();
        let tt = ts.as_texture(tc).unwrap();
        Entity {
            _id: id.to_string(),
            pos: pos,
            color: color,
            onscreen: true,
            idtx: tt,
        }
    }

    /// Set absolute position of the entity
    pub fn pos_set_abs(&mut self, ix: i32, iy: i32) {
        self.pos = (ix, iy);
    }

    /// Set relative position of the entity
    pub fn pos_set_rel(&mut self, ix: i32, iy: i32) {
        self.pos = (self.pos.0 + ix, self.pos.1 + iy);

        if self.onscreen {
            if self.pos.0 < 0 {
                self.pos.0 = SCREEN_WIDTH as i32;
            }
            if self.pos.0 > (SCREEN_WIDTH as i32) {
                self.pos.0 = 0;
            }
            if self.pos.1 < 0 {
                self.pos.1 = SCREEN_HEIGHT as i32;
            }
            if self.pos.1 > (SCREEN_HEIGHT as i32) {
                self.pos.1 = 0;
            }
        }
    }

    /// Draw the entity on passed canvas
    pub fn draw(&self, wc: &mut WindowCanvas) {
        wc.set_draw_color(self.color);
        wc.fill_rect(Rect::new(self.pos.0, self.pos.1, ENTITY_WIDTH, ENTITY_HEIGHT)).unwrap();
        //wc.string(self.pos.0 as i16, self.pos.1 as i16, &self.id, Color::RGB(0, 0, 200)).unwrap();
        wc.copy(&self.idtx, None, Some(Rect::new(self.pos.0, self.pos.1, 16, 16))).unwrap();
    }

}
