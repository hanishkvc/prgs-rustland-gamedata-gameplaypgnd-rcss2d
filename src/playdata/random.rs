//!
//! Random play data
//! HanishKVC, 2022
//!

use rand;

use crate::entities;
use crate::sdlx::XSpaces;

use super::PlayData;
use super::PlayUpdate;

use crate::entities::SCREEN_WIDTH;
use crate::entities::SCREEN_HEIGHT;


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
    s2n: XSpaces,
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
            achg.push(1 + (rand::random::<usize>() % 128));
        }
        let mut bpos = Vec::new();
        let mut bmov = Vec::new();
        let mut bchg = Vec::new();
        for _i in 0..bcnt {
            let fx = (rand::random::<u32>() % 400) as f32;
            let fy = (rand::random::<u32>() % 400) as f32;
            bpos.push((fx, fy));
            bmov.push((0.0, 0.0));
            bchg.push((rand::random::<usize>() % 128) + 1);
        }

        let srect = ((-20.0, -20.0), (SCREEN_WIDTH as f32 + 20.0, SCREEN_HEIGHT as f32 + 20.0));
        let nrect = ((0.0,0.0), (1.0,1.0));

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
            s2n: XSpaces::new(srect, nrect)
        }
    }

}

impl RandomData {

    fn fpos_fix(mut pos: (f32, f32)) -> (f32, f32) {
        if pos.0 < 0.0 {
            pos.0 = SCREEN_WIDTH as f32;
        }
        if pos.0 > (SCREEN_WIDTH as f32) {
            pos.0 = 0.0;
        }
        if pos.1 < 0.0 {
            pos.1 = SCREEN_HEIGHT as f32;
        }
        if pos.1 > (SCREEN_HEIGHT as f32) {
            pos.1 = 0.0;
        }
        return pos;
    }

    fn pos_fix(&mut self) {
        for i in 0..self.acnt {
            self.apos[i] = Self::fpos_fix(self.apos[i]);
        }
        for i in 0..self.bcnt {
            self.bpos[i] = Self::fpos_fix(self.bpos[i]);
        }
    }

}

impl PlayData for RandomData {

    fn fps_changed(&mut self, fps: f32) {
        self.fpr = fps*self.spr;
        self.next = 0.0;
    }

    fn seconds_per_record(&self) -> f32 {
        self.spr
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
    fn next_record(&mut self) -> PlayUpdate {
        let mut pu = PlayUpdate::new();
        for i in 0..self.acnt {
            let dx = (rand::random::<i32>() % 8) as f32;
            let dy = (rand::random::<i32>() % 8) as f32;
            self.apos[i].0 += dx;
            self.apos[i].1 += dy;
            let (fx, fy) = self.s2n.d2o((self.apos[i].0, self.apos[i].1));
            pu.ateampositions.push((i as i32, fx, fy));
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
            let (fx, fy) = self.s2n.d2o((self.bpos[i].0, self.bpos[i].1));
            pu.bteampositions.push((i as i32, fx, fy));
        }
        pu
    }

    #[cfg(not(feature="inbetween_frames"))]
    fn next_record(&mut self) -> PlayUpdate {
        self.rcnt += 1;
        let mut pu = PlayUpdate::new();
        for i in 0..self.acnt {
            if self.rcnt % self.achg[i] == 0 {
                let dx = (rand::random::<i32>() % 4) as f32;
                let dy = (rand::random::<i32>() % 4) as f32;
                self.amov[i] = (dx, dy);
            }
            self.apos[i].0 += self.amov[i].0;
            self.apos[i].1 += self.amov[i].1;
            let (fx, fy) = self.s2n.d2o((self.apos[i].0, self.apos[i].1));
            pu.ateamcoded.push((i as i32, fx, fy, 0.0));
        }
        for i in 0..self.bcnt {
            if self.rcnt % self.bchg[i] == 0 {
                let dx = (rand::random::<i32>() % 4) as f32;
                let dy = (rand::random::<i32>() % 4) as f32;
                self.bmov[i] = (dx, dy);
            }
            self.bpos[i].0 += self.bmov[i].0;
            self.bpos[i].1 += self.bmov[i].1;
            let (fx, fy) = self.s2n.d2o((self.bpos[i].0, self.bpos[i].1));
            pu.bteamcoded.push((i as i32, fx, fy, 0.0));
        }
        self.pos_fix();
        pu
    }

    fn seek(&mut self, seekdelta: isize) {
        self.rcnt = (self.rcnt as isize + seekdelta) as usize;
        return;
    }

    fn bdone(&self) -> bool {
        return false;
    }

    fn send_record(&mut self, _buf: &[u8]) {
        todo!()
    }

    fn send_record_coded(&mut self, _code: isize) {
        todo!()
    }

}
