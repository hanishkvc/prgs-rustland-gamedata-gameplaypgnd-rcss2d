//!
//! A team
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::entities;
use crate::entities::gentity::Entity;
use crate::sdlx::SdlX;



#[derive(Debug)]
pub struct Team<'a> {
    name: String,
    color: Color,
    players: Vec<Entity<'a>>,
    pmoves: Vec<(f32,f32)>,
    pchgmovs: Vec<i32>,
}

impl<'a> Team<'a> {

    pub fn new(name: &str, color: Color, nplayers: i32, font: &'a Font) -> Team<'a> {
        let mut team = Team {
            name: name.to_string(),
            color: color,
            players: Vec::new(),
            pmoves: Vec::new(),
            pchgmovs: Vec::new(),
        };
        let bx = (rand::random::<u32>() % entities::SCREEN_WIDTH) as f32;
        for i in 0..nplayers {
            let fx = (rand::random::<u32>() % (entities::SCREEN_WIDTH/4)) as f32;
            let fy = (rand::random::<u32>() % entities::SCREEN_HEIGHT) as f32;
            team.players.push(Entity::new(i.to_string().as_str(), (bx+fx, fy), team.color, font));
            team.pmoves.push((0.0,0.0));
            let mut chgmov = fx.round() as i32;
            if chgmov == 0 {
                chgmov = 1;
            }
            team.pchgmovs.push(chgmov);
        }
        print!("INFO:PGND:Team:Created:{}:{:#?}\n", team.name, team);
        team
    }

    pub fn update_dummy(&mut self, step: usize) {
        for i in 0..self.players.len() {
            let player = &mut self.players[i];
            if step % (self.pchgmovs[i] as usize) == 0 {
                let dx = (rand::random::<i32>() % 4) as f32;
                let dy = (rand::random::<i32>() % 4) as f32;
                self.pmoves[i] = (dx, dy);
            }
            player.pos_set_rel(self.pmoves[i].0, self.pmoves[i].1);
        }
    }

    pub fn update(&mut self, tposs: Vec<(i32, f32, f32)>, babsolute: bool) {
        for ppos in tposs {
            let fx = ppos.1;
            let fy = ppos.2;
            if babsolute {
                self.players[ppos.0 as usize].pos_set_abs(fx, fy);
            } else {
                self.players[ppos.0 as usize].move_to_in_frames((fx, fy), entities::FRAMES_PER_SEC as f32);
            }
        }
    }

    pub fn next_frame(&mut self) {
        for i in 0..self.players.len() {
            self.players[i].next_frame();
        }
    }

    pub fn draw(&self, sx: &mut SdlX) {
        for i in 0..self.players.len() {
            self.players[i].draw(sx);
        }
    }

}