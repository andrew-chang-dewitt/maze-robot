//! UDP transport for inter-robot swarm communication.
//!
//! Each `Swarm` wraps a caller-configured `UdpSocket` plus a target address. The caller chooses
//! the transport details (broadcast vs. multicast, port, `SO_REUSEPORT`, non-blocking mode); the
//! `Swarm` only handles framing and self-filtering.
//!
//! Wire format: `[8-byte sender nonce][payload bytes]`. Receivers drop any datagram whose nonce
//! matches their own so a sender never sees its own messages.

use std::{
    io,
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct Swarm {
    socket: UdpSocket,
    target: SocketAddr,
    nonce: [u8; 8],
}

impl Swarm {
    pub fn new(socket: UdpSocket, target: SocketAddr) -> Self {
        Self {
            socket,
            target,
            nonce: rand_nonce(),
        }
    }

    /// Send a raw payload to the target address. Prepends the per-instance nonce so receivers
    /// can filter out their own copies of broadcasts/multicasts they emitted.
    pub fn send_raw(&self, payload: &[u8]) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(8 + payload.len());
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(payload);
        self.socket.send_to(&bytes, self.target).map(|_| ())
    }

    /// Non-blocking receive. Returns `Ok(None)` when no datagram is currently ready (requires
    /// the underlying socket to be in non-blocking mode). Drops datagrams whose 8-byte nonce
    /// prefix matches this instance's.
    pub fn try_recv_raw<const N: usize>(&self) -> io::Result<Option<[u8; N]>> {
        // Max UDP payload (65,507 = 65,535 - 20 IP hdr - 8 UDP hdr).
        let mut buf = [0u8; 65507];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((n, _)) => {
                    if n != 8 + N {
                        // datagram must be nonce (8 bytes) + payload (N bytes)
                        continue;
                    }
                    if buf[..8] == self.nonce {
                        // self-broadcast looped back; drop and try the next one
                        continue;
                    }
                    return Ok(Some(
                        buf[8..n].try_into().expect("length already validated"),
                    ));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(e),
            }
        }
    }
}

fn rand_nonce() -> [u8; 8] {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    // ASLR randomizes stack base each run → free entropy source across runs. Within one run,
    // consecutive calls differ by `nanos` (clock advances ns-resolution between calls).
    let stack = &nanos as *const _ as u64;
    (nanos ^ stack).to_le_bytes()
}
