//!
//! Simulated objects
//! HanishKVC, 2022
//!

struct SimBall {
    vdata: Vec<String>,
    ltime: usize,
    lpos: (f32, f32),
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
            ltime: 0,
            lpos: (0.0,0.0),
            mov: (0.0,0.0),
        }
    }

    pub fn next_record(&mut self, ctime: usize) -> (f32, f32) {
        //if ctime < self.
        (0.0, 0.0)
    }

}