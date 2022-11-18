//!
//! Act as a monitor for robocup soccer sim
//! HanishKVC, 2022
//!

use std::net::UdpSocket;
use std::time;

use tokensk::TStrX;
use loggerk::{ldebug,log_d};

use crate::sdlx::XSpaces;

use super::{PlayData, PlayUpdate};

const OWN_ADDRESS: &str = "0.0.0.0:6600";
const READ_TIMEOUT_MS: u64 = 500;
const STAMINA_BASE: f32 = 8000.0;


/// Help act as a simple monitor client for RoboCup Sim
pub struct RCLive {
    skt: UdpSocket,
    /// The robocup server address to communicate to.
    srvraddr: String,
    /// Help tokenise recieved data.
    tstrx: TStrX,
    /// Help convert from Robocups pitch space to normal space.
    r2n: XSpaces,
    /// Track whether the server addr has been updated to
    /// the one over which server sent data to the monitor.
    /// ie after the initial handshake.
    bsrvraddr_updated: bool,
    /// Time wrt last message seen from server
    stime: String,
    /// Team name and Score
    ateam: String,
    bteam: String,
}

impl RCLive {

    pub fn new(addr: &str) -> RCLive {
        let skt = UdpSocket::bind(OWN_ADDRESS).unwrap();
        skt.set_read_timeout(Some(time::Duration::from_millis(READ_TIMEOUT_MS))).unwrap();
        let sinit = "(dispinit version 5)\r\n";
        skt.send_to(sinit.as_bytes(), addr).unwrap();
        eprintln!("DBUG:PPGND:RCLive:New:{:?}", skt);
        let rrect = ((-55.0, -37.0), (55.0, 37.0));
        let nrect = ((0.0,0.0), (1.0,1.0));
        let mut tstrx = TStrX::new();
        tstrx.flags.string_canbe_asubpart = true;
        tstrx.flags.blocktok_dlimuser_endreqd = false;
        tstrx.delims.bracket = ('{','}');
        tstrx.delims.obracket = Some(('[',']'));
        tstrx.delims.string = '"';
        RCLive {
            skt: skt,
            srvraddr: addr.to_string(),
            tstrx: tstrx,
            r2n: XSpaces::new(rrect, nrect),
            bsrvraddr_updated: false,
            stime: String::new(),
            ateam: String::new(),
            bteam: String::new(),
        }
    }

}

impl RCLive {

    fn handle_time(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_,d) = tok.split_once(':').unwrap();
        pu.msgs.insert("stime".to_string(), d.to_string());
        self.stime = d.to_string();
    }

    fn handle_mode(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_t,d) = tok.split_once(':').unwrap();
        pu.msgs.insert("game".to_string(), format!("{}:{}", self.stime, d));
    }

    fn handle_teams(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_t,d) = tok.split_once(':').unwrap();
        let mut tstr = self.tstrx.from_str(d, true);
        tstr.peel_bracket('[').unwrap();
        let teams = tstr.tokens_vec(',', true, false).unwrap();
        for team in teams {
            if team.trim().len() == 0 {
                continue;
            }
            let mut tstr = self.tstrx.from_str(&team, true);
            tstr.peel_bracket('{').unwrap();
            let toks = tstr.tokens_vec(',', true, false).unwrap();
            let mut side = '?';
            let mut name = String::new();
            let mut score = String::new();
            for tok in toks {
                if tok.starts_with("\"side\"") {
                    let (_,d) = tok.split_once(':').unwrap();
                    side = d.chars().nth(1).unwrap();
                }
                if tok.starts_with("\"name\"") {
                    let (_,d) = tok.split_once(':').unwrap();
                    let mut tstr = self.tstrx.from_str(d, true);
                    if tstr.char_first().unwrap() == '"' {
                        tstr.peel_string('"').unwrap();
                    }
                    name = tstr.to_string();
                }
                if tok.starts_with("\"score\"") {
                    let (_,d) = tok.split_once(':').unwrap();
                    score = d.to_string();
                }
            }
            let ts = format!("{} [{}]", name, score);
            if side == 'l' {
                self.ateam = ts;
            } else if side == 'r' {
                self.bteam = ts;
            }
        }
        pu.msgs.insert("score".to_string(), format!("{} vs {}", self.ateam, self.bteam));
    }

    fn handle_ball(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_b,d) = tok.split_once(':').unwrap();
        let mut tstr = self.tstrx.from_str(d, true);
        tstr.peel_bracket('{').unwrap();
        let toksl2 = tstr.tokens_vec(',', true, true).unwrap();
        let mut fx = 0.0;
        let mut fy = 0.0;
        for tokl2 in toksl2 {
            let (k,v) = tokl2.split_once(':').unwrap();
            if k == "\"x\"" {
                fx = v.parse().unwrap();
            }
            if k == "\"y\"" {
                fy = v.parse().unwrap();
            }
        }
        let (fx,fy) = self.r2n.d2o((fx,fy));
        pu.ball = (fx, fy);
    }

    fn handle_players(&mut self, tok: &str, pu: &mut PlayUpdate) {
        let (_p,d) = tok.split_once(':').unwrap();
        let mut tstr = self.tstrx.from_str(d, true);
        tstr.peel_bracket('[').unwrap();
        let toks = tstr.tokens_vec(',', true, false).unwrap();
        ldebug!(&format!("DBUG:PPGND:RCLive:Got:Toks:Players:{:#?}", toks));
        for tok in toks {
            if tok.trim().len() == 0 {
                continue;
            }
            let mut tstr = self.tstrx.from_str(&tok, true);
            tstr.peel_bracket('{').unwrap();
            let toksl2 = tstr.tokens_vec(',', true, true).unwrap();
            ldebug!(&format!("DBUG:PPGND:RCLive:Got:Toks:Player:{:#?}", toksl2));
            let mut pnum = 0;
            let mut fx = 0.0;
            let mut fy = 0.0;
            let mut side = String::new();
            let mut fstamina = 1.0f32;
            for tokl2 in toksl2 {
                let (k,v) = tokl2.split_once(':').unwrap();
                if k == "\"side\"" {
                    side = v.to_string();
                }
                if k == "\"unum\"" {
                    pnum = v.parse().unwrap();
                }
                if k == "\"x\"" {
                    fx = v.parse().unwrap();
                }
                if k == "\"y\"" {
                    fy = v.parse().unwrap();
                }
                if k == "\"stamina\"" {
                    fstamina = v.parse().unwrap();
                }
            }
            let (fx,fy) = self.r2n.d2o((fx,fy));
            fstamina = (fstamina/STAMINA_BASE).min(1.0);
            if side.chars().nth(1).unwrap() == 'l' {
                pu.ateamcoded.push((pnum-1, fx, fy, fstamina));
            } else {
                pu.bteamcoded.push((pnum-1, fx, fy, fstamina));
            }    
        }

    }

}

impl PlayData for RCLive {
    fn seconds_per_record(&self) -> f32 {
        return 0.05;
    }

    fn fps_changed(&mut self, _fps: f32) {
    }

    fn next_frame_is_record_ready(&mut self) -> bool {
        return true;
    }

    fn next_record(&mut self) -> super::PlayUpdate {
        let mut pu = PlayUpdate::new();
        let mut buf = [0u8; 8196];
        let gotr = self.skt.recv_from(&mut buf);
        if gotr.is_err() {
            let err = gotr.unwrap_err();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                eprintln!("WARN:PPGND:RCLive:No data...");
                return pu;
            } else {
                panic!("ERRR:PPGND:RCLive:Unexpected error:{}", err);
            }
        }
        if !self.bsrvraddr_updated {
            self.srvraddr = gotr.as_ref().unwrap().1.to_string();
            self.bsrvraddr_updated = true;
        }
        let sbuf = String::from_utf8_lossy(&buf);
        ldebug!(&format!("DBUG:PPGND:RCLive:Got:{:?}:{}", gotr, &sbuf));
        let mut tstr = self.tstrx.from_str(&sbuf, true);
        tstr.peel_bracket('{').unwrap();
        let toks = tstr.tokens_vec(',', true, true).unwrap();
        ldebug!(&format!("DBUG:PPGND:RCLive:Got:Toks:Full:{:#?}", toks));
        for tok in toks {
            if tok.starts_with("\"type\"") {
                let (_t,d) = tok.split_once(':').unwrap();
                if d != "\"show\"" {
                    eprintln!("DBUG:PPGND:RCLive:UnhandledTypeMsg:{}", sbuf);
                    let ilen = sbuf.trim().len().min(32);
                    pu.msgs.insert("unknown".to_string(), sbuf[0..ilen].to_string());
                    return pu;
                }
                continue;
            }
            if tok.starts_with("\"time\"") {
                self.handle_time(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"mode\"") {
                self.handle_mode(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"teams\"") {
                self.handle_teams(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"ball\"") {
                self.handle_ball(&tok, &mut pu);
                continue;
            }
            if tok.starts_with("\"players\"") {
                self.handle_players(&tok, &mut pu);
                continue;
            }
        }
        ldebug!(&format!("DBUG:PPGND:RCLive:Got:Pu:{:?}", pu));
        pu
    }

    fn seek(&mut self, _seekdelta: isize) {
        return;
    }

    fn bdone(&self) -> bool {
        return false;
    }

    fn send_record(&mut self, buf: &[u8]) {
        self.skt.send_to(buf, &self.srvraddr).unwrap();
        eprintln!("DBUG:PPGND:RCLive:Sent:{:?}:To:{:?}-{:?}", buf, self.skt, self.srvraddr);
    }

    fn send_record_coded(&mut self, code: isize) {
        let msg = match code {
            1 => "(dispstart)\x00".as_bytes(),
            _ => todo!(),
        };
        self.send_record(msg);
    }

}
