//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use tokensk::TStr;

pub struct Rcg {
    fname: String,
    file: File,
    lines: Vec<String>,
    iline: isize,
    pub bdone: bool,
}

impl Rcg {

    pub fn new(fname: &str) -> Rcg {
        let mut file = File::open(fname).unwrap();
        let mut sdata = String::new();
        let gotr = file.read_to_string(&mut sdata).unwrap();
        let vdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vline = Vec::new();
        for line in vdata {
            vline.push(line.to_string());
        }
        Rcg {
            fname: fname.to_string(),
            file: file,
            lines: vline,
            iline: -1,
            bdone: false,
        }
    }

    pub fn next_record(&mut self) -> Vec<String> {
        let bcontinue = true;
        let mut vtoks = Vec::new();
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
                vtoks = toks;
                break;
            } else {
                print!("DBUG:PGND:Rcg:Skipping:{:?}\n", toks);
            }
        }
        return vtoks;
    }

}
