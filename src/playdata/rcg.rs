//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use tokensk::TStr;

use crate::playdata::PositionsUpdate;
use crate::playdata::PlayData;
use crate::sdlx::XSpaces;


pub struct Rcg {
    _fname: String,
    _file: File,
    lines: Vec<String>,
    iline: isize,
    pub bdone: bool,
    framesper_record: f32,
    framesafter_lastrecord: f32,
    r2d: XSpaces,
}

impl Rcg {

    pub fn new(fname: &str) -> Rcg {
        let mut file = File::open(fname).unwrap();
        let mut sdata = String::new();
        let _gotr = file.read_to_string(&mut sdata).unwrap();
        let vdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vline = Vec::new();
        for line in vdata {
            vline.push(line.to_string());
        }
        let rrect = ((-56.0, -37.0), (56.0, 37.0));
        let drect = ((0.0,0.0), (1.0,1.0));
        Rcg {
            _fname: fname.to_string(),
            _file: file,
            lines: vline,
            iline: -1,
            bdone: false,
            framesper_record: 1.0,
            framesafter_lastrecord: 0.0,
            r2d: XSpaces::new(rrect, drect)
        }
    }

}

impl PlayData for Rcg {

    fn setup(&mut self, _fps: f32) {
        self.framesper_record = 1.0;
        self.framesafter_lastrecord = 0.0;
    }

    fn next_frame_is_record_ready(&mut self) -> bool {
        self.framesafter_lastrecord += 1.0;
        if self.framesafter_lastrecord >= self.framesper_record {
            return true;
        }
        return false;
    }

    fn next_record(&mut self) -> PositionsUpdate {
        let bcontinue = true;
        let mut pu = PositionsUpdate::new();
        while bcontinue {
            self.iline += 1;
            if self.iline >= self.lines.len() as isize {
                print!("WARN:PGND:Rcg:No more data\n");
                self.bdone = true;
                break;
            }
            let mut tstr = TStr::from_str(&self.lines[self.iline as usize], true);
            if tstr.len() == 0 {
                continue;
            }
            if tstr.char_first().unwrap() == '#' {
                continue;
            }
            if tstr.the_str().starts_with("ULG") {
                continue;
            }
            tstr.peel_bracket('(').unwrap();
            let toks = tstr.tokens_vec(' ', true, true).unwrap();
            if toks[0].starts_with("show") {
                for tok in toks {
                    if !tok.starts_with("((l") && !tok.starts_with("((r") && !tok.starts_with("((b") {
                        continue;
                    }
                    let mut tstr = TStr::from_str(&tok, true);
                    tstr.peel_bracket('(').unwrap();
                    let vdata = tstr.tokens_vec(' ', true, true).unwrap();
                    if vdata[0].starts_with("(b") {
                        let fx: f32 = vdata[1].parse().unwrap();
                        let fy: f32 = vdata[2].parse().unwrap();
                        let fx = self.r2d.d2ox(fx);
                        let fy = self.r2d.d2oy(fy);
                        pu.ball = (fx, fy);
                        continue;
                    }
                    let mut tstr = TStr::from_str(&vdata[0], true);
                    tstr.peel_bracket('(').unwrap();
                    let (steam, splayer) = tstr.split_once(' ').unwrap();
                    let iplayer: i32 = splayer.parse().unwrap();
                    let fx: f32 = vdata[3].parse().unwrap();
                    let fy: f32 = vdata[4].parse().unwrap();
                    let fx = self.r2d.d2ox(fx);
                    let fy = self.r2d.d2oy(fy);
                    if steam == "l" {
                        pu.ateampositions.push((iplayer-1, fx, fy));
                    } else {
                        pu.bteampositions.push((iplayer-1, fx, fy));
                    }
                }
                break;
            } else {
                print!("DBUG:PGND:Rcg:Skipping:{:?}\n", toks);
            }
        }
        return pu;
    }

    fn bdone(&self) -> bool {
        return self.bdone;
    }

}
