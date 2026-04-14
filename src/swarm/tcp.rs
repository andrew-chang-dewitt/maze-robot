//! TCP-based long-lived peer communication for a robot swarm.
//!
//! Each bot in the swarm maintains one persistent [`TcpStream`] per peer.
//! Messages are length-prefixed binary frames so they compose cleanly over a
//! stream without needing an external serialisation library.
//!
//! # Wire format
//!
//! Every message starts with a single **type byte**:
//!
//! | type byte | variant        | payload                                      |
//! |-----------|----------------|----------------------------------------------|
//! | `0x00`    | `StateUpdate`  | `x: i64 LE`, `y: i64 LE`, `cell: u8`        |
//! | `0x01`    | `Solution`     | `n: u64 LE`, then `n × (x: i64 LE, y: i64 LE)` |
//! | `0x02`    | `Disconnect`   | *(no payload)*                               |

use std::{
    io::{self, Read, Write},
    net::{SocketAddr, Shutdown, TcpListener, TcpStream},
};

use crate::controller::Cell;

// ── Type-tag constants ────────────────────────────────────────────────────────

const TAG_STATE_UPDATE: u8 = 0x00;
const TAG_SOLUTION: u8 = 0x01;
const TAG_DISCONNECT: u8 = 0x02;

// ── Message type ─────────────────────────────────────────────────────────────

/// A message exchanged between swarm peers.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// A newly-discovered cell at grid coordinates `(x, y)`.
    StateUpdate(i64, i64, Cell),

    /// The maze solution has been found; the payload is the winning path as
    /// an ordered list of `(x, y)` grid coordinates.
    Solution(Vec<(i64, i64)>),

    /// Signals all peers that this bot is about to disconnect.
    Disconnect,
}

// ── Listening / connecting ────────────────────────────────────────────────────

/// Bind a [`TcpListener`] on `addr` so that peer bots can connect to us.
///
/// The caller is responsible for calling [`TcpListener::accept`] in whatever
/// loop or thread is appropriate for their architecture.
pub fn listen(addr: SocketAddr) -> io::Result<TcpListener> {
    TcpListener::bind(addr)
}

/// Connect to every address in `peers`, returning one [`TcpStream`] per peer.
///
/// Connections are attempted sequentially.  If any connection fails the
/// entire [`io::Result`] is `Err`; already-opened streams are dropped and
/// their OS connections closed.
pub fn connect(peers: &[SocketAddr]) -> io::Result<Vec<TcpStream>> {
    peers.iter().map(|addr| TcpStream::connect(addr)).collect()
}

// ── Broadcast helpers ─────────────────────────────────────────────────────────

/// Send a newly-discovered cell at `(x, y)` to every connected peer.
///
/// Iterates over `peers` sequentially; the first write error is returned and
/// remaining peers are skipped.
pub fn broadcast_state(peers: &mut [TcpStream], x: i64, y: i64, cell: Cell) -> io::Result<()> {
    broadcast(peers, &Message::StateUpdate(x, y, cell))
}

/// Notify every connected peer that this bot has found the solution and share
/// the winning `path` as an ordered list of `(x, y)` coordinates.
///
/// Iterates over `peers` sequentially; the first write error is returned and
/// remaining peers are skipped.
pub fn broadcast_solution(peers: &mut [TcpStream], path: &[(i64, i64)]) -> io::Result<()> {
    broadcast(peers, &Message::Solution(path.to_vec()))
}

/// Tell every connected peer that it is time to disconnect.
///
/// Iterates over `peers` sequentially; the first write error is returned and
/// remaining peers are skipped.
pub fn broadcast_disconnect(peers: &mut [TcpStream]) -> io::Result<()> {
    broadcast(peers, &Message::Disconnect)
}

/// Cleanly shut down every stream in `peers`.
///
/// Both the send and receive halves are shut down on each stream.  The first
/// shutdown error is returned; remaining streams are still shut down on a
/// best-effort basis before the error propagates.
pub fn disconnect(peers: Vec<TcpStream>) -> io::Result<()> {
    let mut first_err: Option<io::Error> = None;

    for stream in peers {
        if let Err(e) = stream.shutdown(Shutdown::Both) {
            // Record the first error but keep trying the remaining streams.
            if first_err.is_none() {
                first_err = Some(e);
            }
        }
    }

    match first_err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

// ── Low-level message I/O ─────────────────────────────────────────────────────

/// Serialise and write `msg` to `stream`, then flush.
///
/// See the [module-level documentation](self) for the wire format.
pub fn write_message(stream: &mut TcpStream, msg: &Message) -> io::Result<()> {
    match msg {
        Message::StateUpdate(x, y, cell) => {
            stream.write_all(&[TAG_STATE_UPDATE])?;
            stream.write_all(&x.to_le_bytes())?;
            stream.write_all(&y.to_le_bytes())?;
            stream.write_all(&[cell_to_byte(cell)])?;
        }
        Message::Solution(path) => {
            stream.write_all(&[TAG_SOLUTION])?;
            // Write the count so the receiver knows how many pairs follow.
            stream.write_all(&(path.len() as u64).to_le_bytes())?;
            for (x, y) in path {
                stream.write_all(&x.to_le_bytes())?;
                stream.write_all(&y.to_le_bytes())?;
            }
        }
        Message::Disconnect => {
            stream.write_all(&[TAG_DISCONNECT])?;
        }
    }

    // Flush to ensure the bytes leave the send buffer immediately.
    stream.flush()
}

/// Read and deserialise one message from `stream`.
///
/// This call blocks until a complete message has been received.
/// See the [module-level documentation](self) for the wire format.
pub fn read_message(stream: &mut TcpStream) -> io::Result<Message> {
    let mut tag = [0u8; 1];
    stream.read_exact(&mut tag)?;

    match tag[0] {
        TAG_STATE_UPDATE => {
            let x = read_i64(stream)?;
            let y = read_i64(stream)?;
            let mut cell_byte = [0u8; 1];
            stream.read_exact(&mut cell_byte)?;
            let cell = byte_to_cell(cell_byte[0])?;
            Ok(Message::StateUpdate(x, y, cell))
        }
        TAG_SOLUTION => {
            let raw_count = read_u64(stream)?;
            // Guard against a malformed count that would cause excessive allocation.
            // A maze path longer than 1 000 000 steps is unrealistic.
            const MAX_PATH_LEN: u64 = 1_000_000;
            if raw_count > MAX_PATH_LEN {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("solution path count {raw_count} exceeds maximum {MAX_PATH_LEN}"),
                ));
            }
            let count = raw_count as usize;
            let mut path = Vec::with_capacity(count);
            for _ in 0..count {
                let x = read_i64(stream)?;
                let y = read_i64(stream)?;
                path.push((x, y));
            }
            Ok(Message::Solution(path))
        }
        TAG_DISCONNECT => Ok(Message::Disconnect),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown message type byte: {other:#04x}"),
        )),
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Send `msg` to every stream in `peers`, returning on the first error.
fn broadcast(peers: &mut [TcpStream], msg: &Message) -> io::Result<()> {
    for stream in peers.iter_mut() {
        write_message(stream, msg)?;
    }
    Ok(())
}

fn read_i64(stream: &mut TcpStream) -> io::Result<i64> {
    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf)?;
    Ok(i64::from_le_bytes(buf))
}

fn read_u64(stream: &mut TcpStream) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

/// Map a [`Cell`] to its wire byte.
fn cell_to_byte(cell: &Cell) -> u8 {
    match cell {
        Cell::Open => 0,
        Cell::Finish => 1,
        Cell::Wall => 2,
    }
}

/// Map a wire byte back to a [`Cell`], or return an `InvalidData` error.
fn byte_to_cell(b: u8) -> io::Result<Cell> {
    match b {
        0 => Ok(Cell::Open),
        1 => Ok(Cell::Finish),
        2 => Ok(Cell::Wall),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown cell byte: {other:#04x}"),
        )),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    // Helper: spin up a loopback listener, connect a client, accept a server
    // stream, and return both ends so tests can write/read without noise.
    fn loopback_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).unwrap();
        let (server, _) = listener.accept().unwrap();
        (client, server)
    }

    // Round-trip each message variant through write_message / read_message.

    #[test]
    fn roundtrip_state_update() {
        let (mut writer, mut reader) = loopback_pair();

        let msg = Message::StateUpdate(-7, 42, Cell::Open);
        write_message(&mut writer, &msg).unwrap();
        let got = read_message(&mut reader).unwrap();

        assert_eq!(got, msg);
    }

    #[test]
    fn roundtrip_solution() {
        let (mut writer, mut reader) = loopback_pair();

        let path = vec![(0, 0), (1, 0), (2, 0), (2, -1)];
        let msg = Message::Solution(path);
        write_message(&mut writer, &msg).unwrap();
        let got = read_message(&mut reader).unwrap();

        assert_eq!(got, msg);
    }

    #[test]
    fn roundtrip_solution_empty_path() {
        let (mut writer, mut reader) = loopback_pair();

        let msg = Message::Solution(vec![]);
        write_message(&mut writer, &msg).unwrap();
        let got = read_message(&mut reader).unwrap();

        assert_eq!(got, msg);
    }

    #[test]
    fn roundtrip_disconnect() {
        let (mut writer, mut reader) = loopback_pair();

        let msg = Message::Disconnect;
        write_message(&mut writer, &msg).unwrap();
        let got = read_message(&mut reader).unwrap();

        assert_eq!(got, msg);
    }

    #[test]
    fn multiple_messages_on_same_stream() {
        let (mut writer, mut reader) = loopback_pair();

        // Write several messages back-to-back; they must be read in order.
        let msgs = vec![
            Message::StateUpdate(0, 0, Cell::Wall),
            Message::StateUpdate(1, 0, Cell::Finish),
            Message::Solution(vec![(0, 0), (1, 0)]),
            Message::Disconnect,
        ];

        for m in &msgs {
            write_message(&mut writer, m).unwrap();
        }

        for expected in &msgs {
            let got = read_message(&mut reader).unwrap();
            assert_eq!(&got, expected);
        }
    }

    #[test]
    fn unknown_tag_returns_error() {
        let (mut writer, mut reader) = loopback_pair();

        // Write a raw unknown tag byte.
        writer.write_all(&[0xFF]).unwrap();
        writer.flush().unwrap();

        let err = read_message(&mut reader).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn connect_and_listen_roundtrip() {
        // Verify the public listen() / connect() entry-points compose correctly.
        let listener = listen("127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        let mut clients = connect(&[addr]).unwrap();
        assert_eq!(clients.len(), 1);

        let (mut server_stream, _) = listener.accept().unwrap();

        // Client sends a state update; server reads it back.
        let msg = Message::StateUpdate(3, -5, Cell::Finish);
        write_message(&mut clients[0], &msg).unwrap();
        let got = read_message(&mut server_stream).unwrap();
        assert_eq!(got, msg);
    }

    #[test]
    fn broadcast_state_reaches_all_peers() {
        // Set up two listener / stream pairs to simulate two peers.
        let l1 = TcpListener::bind("127.0.0.1:0").unwrap();
        let l2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let addrs = [l1.local_addr().unwrap(), l2.local_addr().unwrap()];

        let mut peers = connect(&addrs).unwrap();
        let (mut s1, _) = l1.accept().unwrap();
        let (mut s2, _) = l2.accept().unwrap();

        broadcast_state(&mut peers, 1, 2, Cell::Open).unwrap();

        let m1 = read_message(&mut s1).unwrap();
        let m2 = read_message(&mut s2).unwrap();

        assert_eq!(m1, Message::StateUpdate(1, 2, Cell::Open));
        assert_eq!(m2, Message::StateUpdate(1, 2, Cell::Open));
    }
}
