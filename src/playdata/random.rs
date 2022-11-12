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
    apos: Vec<(f32,f32)>,
    bpos: Vec<(f32,f32)>,
}

impl RandomData {

    pub fn new(spr: f32, acnt: usize, bcnt: usize) -> RandomData {
        let mut apos = Vec::new();
        for _i in 0..acnt {
            apos.push((0.0,0.0));
        }
        let mut bpos = Vec::new();
        for _i in 0..bcnt {
            bpos.push((0.0,0.0));
        }

        RandomData {
            spr: spr,
            fpr: 0.0,
            next: 0.0,
            acnt: acnt,
            bcnt: bcnt,
            apos: apos,
            bpos: bpos,
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
            let dx = (rand::random::<i32>() % 8) as f32;
            let dy = (rand::random::<i32>() % 8) as f32;
            self.apos[i].0 += dx;
            self.apos[i].1 += dy;
            pu.ateampositions.push((i as i32, self.apos[i].0, self.apos[i].1));
        }
        let mut dx;
        let mut dy;
        for i in 0..self.bcnt {
            if cfg!(feature="inbetween_frames") {
                dx = 1 + rand::random::<i32>() % 16;
                dy = 1 + rand::random::<i32>() % 16;
            } else {
                dx = 1 + rand::random::<i32>() % 2;
                dy = 1 + rand::random::<i32>() % 2;
            }
            self.bpos[i].0 += (dx as f32);
            self.bpos[i].1 += (dy as f32);
            pu.bteampositions.push((i as i32, self.bpos[i].0, self.bpos[i].1));
        }
        pu
    }

    fn bdone(&self) -> bool {
        return false;
    }

}
