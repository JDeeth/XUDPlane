use std::{net::UdpSocket, time::Duration};

#[macro_use]
extern crate structure;

struct XpUdp {
    xp_udp_addr: String,
    socket: UdpSocket,
}

impl XpUdp {
    fn new(xp_host_addr: &str, client_port: u16) -> Self {
        Self {
            xp_udp_addr: format!("{xp_host_addr}:49000"),
            socket: UdpSocket::bind(format!("0.0.0.0:{client_port}"))
                .expect("could not bind socket"),
        }
    }

    fn send(&self, packed_msg: &[u8]) {
        self.socket
            .send_to(&packed_msg, self.xp_udp_addr.as_str())
            .unwrap();
    }

    fn command_once(&self, name: &str) {
        self.send(&pack_cmnd(name))
    }

    fn set_dataref(&self, dataref: &str, value: f32) {
        self.send(&pack_dref(dataref, value))
    }

    fn subscribe_to_dataref(&self, dataref: &str, freq_per_sec: i32, reference: i32) {
        self.send(&pack_rref(dataref, freq_per_sec, reference))
    }
}

fn pack_cmnd(command_name: &str) -> Vec<u8> {
    structure!("<4sx500s")
        .pack(b"CMND", command_name.as_bytes())
        .unwrap()
}

fn pack_dref(dataref: &str, value: f32) -> Vec<u8> {
    structure!("<4sxf500s")
        .pack(b"DREF", value, dataref.as_bytes())
        .unwrap()
}

fn pack_rref(dataref: &str, per_sec: i32, index: i32) -> Vec<u8> {
    structure!("<4sxii400s")
        .pack(b"RREF", per_sec, index, dataref.as_bytes())
        .unwrap()
}

fn main() -> std::io::Result<()> {
    // This is just a a scratchpad to test against a running sim
    let xp = XpUdp::new("192.168.178.36", 49015);
    let simtimepaused_idx = 12;
    xp.socket.set_nonblocking(false)?;
    xp.socket.set_read_timeout(Some(Duration::new(2, 0)))?;
    xp.set_dataref("sim/cockpit2/radios/actuators/transponder_code", 1024f32);
    xp.command_once("sim/operation/pause_off");
    xp.subscribe_to_dataref("sim/time/paused", 3, simtimepaused_idx);
    let mut msg_buf = [0u8; 256];
    for i in 0.. {
        match xp.socket.recv_from(&mut msg_buf) {
            Ok(_) => println!("{i}: {:x?}", msg_buf.to_vec()),
            _ => (),
        }
        if i == 12 {
            xp.command_once("sim/operation/pause_on");
        }
        if i == 20 {
            xp.subscribe_to_dataref("sim/time/paused", 0, simtimepaused_idx);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmnd_pack() {
        let packed = &pack_cmnd("hello/world");
        let expected_data = b"CMND\0hello/world\0";
        assert!(packed.starts_with(expected_data));
        assert_eq!(packed.len(), 505);
    }

    #[test]
    fn test_dref_pack() {
        let packed = pack_dref("hello/world", 1f32);
        let expected_data = b"DREF\0\0\0\x80\x3fhello/world\0";
        assert!(packed.starts_with(expected_data));
        assert_eq!(packed.len(), 509);
    }

    #[test]
    fn test_rref_pack() {
        let packed = pack_rref("hello/world", 7, 42);
        let expected_data = b"RREF\0\x07\0\0\0\x2a\0\0\0hello/world\0";
        assert!(packed.starts_with(expected_data));
        assert_eq!(packed.len(), 413);
    }
}
