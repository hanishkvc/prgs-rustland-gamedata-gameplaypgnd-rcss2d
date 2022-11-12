//!
//! Random play data
//! HanishKVC, 2022
//!

use rand;

use super::PlayData;
use super::PositionsUpdate;

pub struct RandomData {
    /// seconds per record
    spr: f32,
    /// frames per record
    fpr: f32,
    next: f32,
    acnt: usize,
    bcnt: usize,
}

impl RandomData {

    pub fn new(spr: f32, acnt: usize, bcnt: usize) -> RandomData {
        RandomData {
            spr: spr,
            fpr: 0.0,
            next: 0.0,
            acnt: acnt,
            bcnt: bcnt,
        }
    }

}

impl PlayData for RandomData {

    fn setup(&mut self, fps: f32) {
        self.fpr = fps*self.spr;
        self.next = 0.0;
    }

    fn next_frame_is_record_ready(&mut self) -> bool {
        self.next += 1.0;
        if self.next >= self.fpr {
            self.next = self.next - self.fpr;
            return true;
        }
        return false;
    }

    fn next_record(&mut self) -> PositionsUpdate {
        let mut pu = PositionsUpdate::new();
        for i in 0..self.acnt {
            let dx = (rand::random::<i32>() % 128) as f32;
            let dy = (rand::random::<i32>() % 128) as f32;
            pu.ateampositions.push((i as i32, dx, dy));
        }
        let maxx = 1 + rand::random::<u32>() % 256;
        for i in 0..self.bcnt {
            let maxy = 1 + rand::random::<u32>() % 256;
            let dx = (rand::random::<i32>() % maxx as i32) as f32;
            let dy = (rand::random::<i32>() % maxy as i32) as f32;
            pu.bteampositions.push((i as i32, dx, dy));
        }
        pu
    }

    fn bdone(&self) -> bool {
        return false;
    }

}
