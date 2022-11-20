//!
//! Random play data
//! HanishKVC, 2022
//!

use rand;

use crate::entities;
use crate::sdlx::XSpaces;

use super::PlayData;
use super::PlayUpdate;
use super::VPlayerData;
use super::PlayerData;


use crate::entities::SCREEN_WIDTH;
use crate::entities::SCREEN_HEIGHT;

const FRAMES_NORMAL_SPR_MULT: f32 = 4.0;
const FRAMES_INBTW_SPR_MULT: f32 = FRAMES_NORMAL_SPR_MULT*50.0;

struct Team {
    cnt: usize,
    pos: Vec<(f32,f32)>,
    mov: Vec<(f32, f32)>,
    chg: Vec<usize>,
}

impl Team {

    fn new(cnt: usize) -> Team {
        let mut pos = Vec::new();
        let mut mov = Vec::new();
        let mut chg = Vec::new();
        for _i in 0..cnt {
            let fx = (rand::random::<u32>() % entities::SCREEN_WIDTH) as f32;
            let fy = (rand::random::<u32>() % entities::SCREEN_HEIGHT) as f32;
            pos.push((fx, fy));
            mov.push((0.0, 0.0));
            chg.push(1 + (rand::random::<usize>() % 128)*(FRAMES_NORMAL_SPR_MULT as usize));
        }
        Team {
            cnt: cnt,
            pos: pos,
            mov: mov,
            chg: chg,
        }
    }

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
        for i in 0..self.cnt {
            self.pos[i] = Self::fpos_fix(self.pos[i]);
        }
    }


}

pub struct RandomData {
    /// seconds per record
    base_spr: f32,
    /// frames per record
    fpr: f32,
    next: f32,
    ateam: Team,
    bteam: Team,
    rcnt: usize,
    s2n: XSpaces,
}

impl RandomData {

    ///
    /// base_spr: the smallest fraction of a second, at which the logic works internally wrt movemens
    pub fn new(base_spr: f32, acnt: usize, bcnt: usize) -> RandomData {
        let srect = ((-20.0, -20.0), (SCREEN_WIDTH as f32 + 20.0, SCREEN_HEIGHT as f32 + 20.0));
        let nrect = ((0.0,0.0), (1.0,1.0));
        let ateam = Team::new(acnt);
        let bteam = Team::new(bcnt);
        RandomData {
            base_spr,
            fpr: 0.0,
            next: 0.0,
            rcnt: 0,
            s2n: XSpaces::new(srect, nrect),
            ateam: ateam,
            bteam: bteam,
        }
    }

}

impl RandomData {

    fn pos_fix(&mut self) {
        self.ateam.pos_fix();
        self.bteam.pos_fix();
    }

}

impl PlayData for RandomData {

    fn fps_changed(&mut self, fps: f32) {
        self.fpr = fps*self.seconds_per_record();
        self.next = 0.0;
    }

    #[cfg(feature="inbetween_frames")]
    fn seconds_per_record(&self) -> f32 {
        self.base_spr*FRAMES_INBTW_SPR_MULT
    }

    #[cfg(not(feature="inbetween_frames"))]
    fn seconds_per_record(&self) -> f32 {
        self.base_spr*FRAMES_NORMAL_SPR_MULT
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
            let pd = VPlayerData::new();
            pd.push(PlayerData::Pos(fx, fy));
            let fstamina = ((self.rcnt%3000) as f32)/3000.0;
            pd.push(PlayerData::Stamina(fstamina));
            pu.ateamcoded.push((i as i32, pd));
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
            let pd = VPlayerData::new();
            pd.push(PlayerData::Pos(fx, fy));
            let fstamina = ((self.rcnt%3000) as f32)/3000.0;
            pd.push(PlayerData::Stamina(fstamina));
            pu.bteamcoded.push((i as i32, pd));
        }
        pu
    }

    #[cfg(not(feature="inbetween_frames"))]
    fn next_record(&mut self) -> PlayUpdate {
        self.rcnt += 1;
        let mut pu = PlayUpdate::new();
        // Messages
        pu.msgs.insert("stime".to_string(), self.rcnt.to_string());
        let gphase = (self.rcnt%3000)/1000;
        let sgphase = match gphase {
            0 => "Phase 1",
            1 => "Phase 2",
            2 => "Phase 3",
            _ => todo!(),
        };
        pu.msgs.insert("game".to_string(), sgphase.to_string());
        // Player datas
        for i in 0..self.acnt {
            if self.rcnt % self.achg[i] == 0 {
                let dx = (rand::random::<i32>() % 4) as f32;
                let dy = (rand::random::<i32>() % 4) as f32;
                self.amov[i] = (dx, dy);
            }
            self.apos[i].0 += self.amov[i].0;
            self.apos[i].1 += self.amov[i].1;
            let (fx, fy) = self.s2n.d2o((self.apos[i].0, self.apos[i].1));
            let mut pd = VPlayerData::new();
            pd.push(PlayerData::Pos(fx, fy));
            let fstamina = 1.0-(((self.rcnt%3000) as f32)/3000.0);
            pd.push(PlayerData::Stamina(fstamina));
            pu.ateamcoded.push((i as i32, pd));
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
            let mut pd = VPlayerData::new();
            pd.push(PlayerData::Pos(fx, fy));
            let fstamina = 1.0-(((self.rcnt%3000) as f32)/3000.0);
            pd.push(PlayerData::Stamina(fstamina));
            pu.bteamcoded.push((i as i32, pd));
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

    fn send_record_coded(&mut self, code: isize) {
        eprintln!("WARN:PPGND:PlayDataRandom:ignoring request for send record coded [{}]", code);
    }

}
