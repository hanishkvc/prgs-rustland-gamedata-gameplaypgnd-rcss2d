//!
//! Simulated objects
//! HanishKVC, 2022
//!

#[derive(Debug)]
/// A interpolated ball
pub struct VirtBall {
    vdata: Vec<String>,
    vin: usize,
    ltime: usize,
    lpos: (f32, f32),
    cpos: (f32, f32),
    mov: (f32, f32),
    lastgentime: usize,
}

impl VirtBall {

    pub fn new(fname: &str) -> VirtBall {
        let sdata = String::from_utf8(std::fs::read(fname).unwrap()).unwrap();
        let tdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vdata = Vec::new();
        for data in tdata {
            vdata.push(data.to_string());
        }
        VirtBall {
            vdata: vdata,
            vin: 0,
            ltime: 0,
            lpos: (0.0, 0.0),
            cpos: (0.0, 0.0),
            mov: (0.0, 0.0),
            lastgentime: 0,
        }
    }

    ///
    /// Calculate the interpolated position wrt each requested time.
    /// If the last time is repeated again, the same position is sent.
    /// If there is no more data, keep the ball moving in the direction
    /// it already is.
    ///
    /// It uses the current position and position of the ball wrt the
    /// immidiate next action that will be there in the game future, to
    /// help interpoloate the ball positions. This calculation is repeated/
    /// done when ever the ball (or rather playback) has just gone past a
    /// known game action time, wrt the next segment.
    ///
    pub fn next_record(&mut self, ctime: usize) -> (f32, f32) {
        if ctime == self.lastgentime {
            return self.cpos;
        }
        self.lastgentime = ctime;
        while ctime > self.ltime {
            if self.vin >= self.vdata.len() {
                break;
            }
            let sdata = &self.vdata[self.vin];
            self.vin += 1;
            if sdata.trim().len() == 0 {
                break;
            }
            let sdata = sdata.split(',').collect::<Vec<&str>>();
            self.ltime = sdata[0].parse().unwrap();
            let fx = sdata[1].parse().unwrap();
            let fy = sdata[2].parse().unwrap();
            self.lpos = (fx, fy);
            let dt = self.ltime as isize - ctime as isize;
            if dt < 0 {
                continue;
            } else if dt == 0 {
                self.cpos = self.lpos;
                return self.lpos;
            }
            let dx = (self.lpos.0 - self.cpos.0)/(dt as f32 +1.0);
            let dy = (self.lpos.1 - self.cpos.1)/(dt as f32 +1.0);
            self.mov = (dx,dy);
        }
        self.cpos = (self.cpos.0 + self.mov.0, self.cpos.1 + self.mov.1);
        self.cpos
    }

}
