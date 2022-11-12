//!
//! Random play data
//! HanishKVC, 2022
//!

use rand;

use crate::entities;

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
    amov: Vec<(f32, f32)>,
    bmov: Vec<(f32, f32)>,
    achg: Vec<usize>,
    bchg: Vec<usize>,
    rcnt: usize,
}

impl RandomData {

    pub fn new(spr: f32, acnt: usize, bcnt: usize) -> RandomData {
        let mut apos = Vec::new();
        let mut amov = Vec::new();
        let mut achg = Vec::new();
        for _i in 0..acnt {
            let fx = (rand::random::<u32>() % entities::SCREEN_WIDTH) as f32;
            let fy = (rand::random::<u32>() % entities::SCREEN_HEIGHT) as f32;
            apos.push((fx, fy));
            amov.push((0.0, 0.0));
            achg.push(rand::random::<usize>() % 128);
        }
        let mut bpos = Vec::new();
        let mut bmov = Vec::new();
        let mut bchg = Vec::new();
        for _i in 0..bcnt {
            let fx = (rand::random::<u32>() % 400) as f32;
            let fy = (rand::random::<u32>() % 400) as f32;
            bpos.push((fx, fy));
            bmov.push((0.0, 0.0));
            bchg.push(rand::random::<usize>() % 128);
        }

        RandomData {
            spr: spr,
            fpr: 0.0,
            next: 0.0,
            acnt: acnt,
            bcnt: bcnt,
            apos: apos,
            bpos: bpos,
            amov: amov,
            bmov: bmov,
            rcnt: 0,
            achg: achg,
            bchg: bchg,
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

    #[cfg(feature="inbetween_frames")]
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
            self.bpos[i].0 += dx as f32;
            self.bpos[i].1 += dy as f32;
            pu.bteampositions.push((i as i32, self.bpos[i].0, self.bpos[i].1));
        }
        pu
    }

    #[cfg(not(feature="inbetween_frames"))]
    fn next_record(&mut self) -> PositionsUpdate {
        self.rcnt += 1;
        let mut pu = PositionsUpdate::new();
        for i in 0..self.acnt {
            if self.rcnt % self.achg[i] == 0 {
                let dx = (rand::random::<i32>() % 4) as f32;
                let dy = (rand::random::<i32>() % 4) as f32;
                self.amov[i] = (dx, dy);
            }
            self.apos[i].0 += self.amov[i].0;
            self.apos[i].1 += self.amov[i].1;
            pu.ateampositions.push((i as i32, self.apos[i].0, self.apos[i].1));
        }
        for i in 0..self.bcnt {
            if self.rcnt % self.bchg[i] == 0 {
                let dx = (rand::random::<i32>() % 4) as f32;
                let dy = (rand::random::<i32>() % 4) as f32;
                self.bmov[i] = (dx, dy);
            }
            self.bpos[i].0 += self.bmov[i].0;
            self.bpos[i].1 += self.bmov[i].1;
            pu.bteampositions.push((i as i32, self.bpos[i].0, self.bpos[i].1));
        }
        pu
    }

    fn bdone(&self) -> bool {
        return false;
    }

}
