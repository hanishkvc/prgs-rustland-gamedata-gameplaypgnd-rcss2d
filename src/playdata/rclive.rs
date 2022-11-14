//!
//! Act as a monitor for robocup soccer sim
//! HanishKVC, 2022
//!

use std::net::UdpSocket;

use super::{PlayData, PlayUpdate};

pub struct RCLive {
    skt: UdpSocket,
    srvraddr: String,
}

impl RCLive {

    pub fn new(addr: &str) -> RCLive {
        let skt = UdpSocket::bind("0.0.0.0:6600").unwrap();
        let sinit = "(dispinit version 5)\r\n";
        skt.send_to(sinit.as_bytes(), addr).unwrap();
        eprintln!("DBUG:PPGND:RCLive:New:{:?}", skt);
        RCLive {
            skt: skt,
            srvraddr: addr.to_string(),
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
        let pu = PlayUpdate::new();
        let mut buf = [0u8; 2048];
        let gotr = self.skt.recv_from(&mut buf);
        eprintln!("DBUG:PPGND:RCLive:Got:{:?}", gotr);
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
    }

}
