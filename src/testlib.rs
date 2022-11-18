//!
//! Testlib
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx;
use crate::entities::gentity::GEntity;

pub fn test_ncolor() {
    for i in 0..100 {
        let f = i as f32/100.0;
        let color = sdlx::ncolor_gyr(f);
        println!("DBUG:TestNColor:{}:{:?}",f, color);
    }
}

pub fn test_gentity(font: &Font) {
    let mut g1 = GEntity::new("test01", (0.5,0.5), (16,16), Color::WHITE, font);
    g1.set_fcolor(0.25, 1.0);
    g1.set_nxarc(1.2, 0.98, Color::RED);
}
