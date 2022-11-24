//!
//! Simulated objects
//! HanishKVC, 2022
//!

/// A interpolated ball
struct SimBall {
    vdata: Vec<String>,
    vin: usize,
    ltime: usize,
    lpos: (f32, f32),
    cpos: (f32, f32),
    mov: (f32, f32),
}

impl SimBall {

    pub fn new(fname: &str) -> SimBall {
        let sdata = String::from_utf8(std::fs::read(fname).unwrap()).unwrap();
        let tdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vdata = Vec::new();
        for data in tdata {
            vdata.push(data.to_string());
        }
        SimBall {
            vdata: vdata,
            vin: 0,
            ltime: 0,
            lpos: (0.0, 0.0),
            cpos: (0.0, 0.0),
            mov: (0.0, 0.0),
        }
    }

    pub fn next_record(&mut self, ctime: usize) -> (f32, f32) {
        while ctime > self.ltime {
            let sdata = &self.vdata[self.vin];
            self.vin += 1;
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
