//!
//! Process robocup soccer simulator rcg server to monitor log file
//! HanishKVC, 2022
//!

use std::{fs::File, io::Read};
use tokensk::TStr;

struct Rcg {
    fname: String,
    file: File,
    lines: Vec<String>,
    inext: usize,
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
            inext: 0,
        }
    }

    pub fn next_record(&self) -> Vec<String> {
        let bcontinue = true;
        let mut vtoks = Vec::new();
        while bcontinue {
            let mut tstr = TStr::from_str(&self.lines[self.inext], true);
            if tstr.len() == 0 {
                continue;
            }
            if tstr.char_first().unwrap() == '#' {
                continue;
            }
            vtoks = tstr.tokens_vec(' ', true, true).unwrap();
            if vtoks[0].starts_with("show") {
                break;
            }
        }
        return vtoks;
    }

}
