use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use socket2::{Domain, Protocol, Socket, Type};

use crate::{
    dist_maze::DistMazeClient,
    traits::{MazeError, Robot, RobotError, RobotErrorType, RobotInternal},
};

use super::swarm::Swarm;

/// Implementation of [`crate::traits::Robot`] that queries maze environment (an instance of
/// [`crate::dist_maze::DistMazeServer`]) via tcp sockets using internal
/// [`crate::dist_maze::DistMazeClient`] instance.
///
/// Optionally joins a UDP-broadcast swarm transport via [`Self::join_swarm`] for inter-robot
/// messaging.
#[derive(Debug)]
pub struct DistRobot {
    env: RobotInternal,
    swarm: Option<Swarm>,
}

impl DistRobot {
    /// Create a new distributed robot by telling it at what address to find the distributed maze.
    ///
    /// Initializes a DistMazeClient and connects it to the DistMazeServer at the address provided.
    /// The server will auto-register this bot on first contact.
    pub fn try_build(maze_addr: SocketAddr) -> Result<Self, MazeError> {
        let maze = DistMazeClient::try_from(maze_addr)?;
        Ok(Self {
            env: RobotInternal::new(maze),
            swarm: None,
        })
    }

    /// Attach a UDP broadcast swarm transport to this robot for the standard one-bot-per-host
    /// deployment.
    ///
    /// Binds a non-blocking UDP socket to `0.0.0.0:port` with `SO_BROADCAST` enabled and sends
    /// every outbound message to the limited-broadcast address `255.255.255.255:port`, which
    /// reaches all peers on the local subnet via the wire-facing interface.
    ///
    /// **Not suitable for multiple robots in one process** — Linux does not deliver limited
    /// broadcasts back to local sockets on the same host. Use [`Self::join_swarm_local`] for
    /// same-host deployments and tests.
    pub fn join_swarm(self, port: u16) -> Result<Self, RobotError> {
        self.join_swarm_inner(port, false)
    }

    /// Attach a UDP broadcast swarm transport configured for multiple robots running on a single
    /// host (e.g. tests, demos).
    ///
    /// Binds a non-blocking UDP socket to `0.0.0.0:port` with `SO_REUSEADDR` + `SO_REUSEPORT` so
    /// multiple in-process robots can share the port, and broadcasts to the loopback-subnet
    /// broadcast address `127.255.255.255:port` so the kernel delivers packets via `lo` to every
    /// peer bound on the same port.
    pub fn join_swarm_local(self, port: u16) -> Result<Self, RobotError> {
        self.join_swarm_inner(port, true)
    }

    fn join_swarm_inner(mut self, port: u16, local: bool) -> Result<Self, RobotError> {
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|e| transport_err("create socket", e))?;
        if local {
            sock.set_reuse_address(true)
                .map_err(|e| transport_err("set SO_REUSEADDR", e))?;
            sock.set_reuse_port(true)
                .map_err(|e| transport_err("set SO_REUSEPORT", e))?;
        }
        sock.set_broadcast(true)
            .map_err(|e| transport_err("set SO_BROADCAST", e))?;
        sock.set_nonblocking(true)
            .map_err(|e| transport_err("set non-blocking", e))?;
        let bind = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
        sock.bind(&SocketAddr::V4(bind).into())
            .map_err(|e| transport_err(&format!("bind {bind}"), e))?;
        let bcast = if local {
            Ipv4Addr::new(127, 255, 255, 255)
        } else {
            Ipv4Addr::BROADCAST
        };
        let target = SocketAddr::V4(SocketAddrV4::new(bcast, port));
        self.swarm = Some(Swarm::new(sock.into(), target));
        Ok(self)
    }

    /// Send a message to all peer robots via the swarm broadcast.
    ///
    /// The message is encoded into bytes via the user-provided `TryInto<[u8; 32]>` impl, then
    /// broadcast to `255.255.255.255:port`. Returns `RobotError::NotJoined` if `join_swarm`
    /// has not been called.
    pub fn try_send<T, const N: usize>(&self, msg: T) -> Result<(), RobotError>
    where
        T: TryInto<[u8; N]>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        let swarm = self
            .swarm
            .as_ref()
            .ok_or_else(|| RobotError::new(RobotErrorType::NotJoined))?;
        let payload: [u8; N] = msg.try_into().map_err(|e| {
            RobotError::new(RobotErrorType::EncodeError(e.to_string())).caused_by(e)
        })?;
        swarm
            .send_raw(&payload)
            .map_err(|e| transport_err("send", e))
    }

    /// Fetch the next unprocessed message from the swarm. Non-blocking: returns `Ok(None)` when
    /// no message is currently waiting. The payload is decoded into `T` via the user-provided
    /// `TryFrom<[u8; 32]>` impl. Messages this robot sent are dropped (the user never sees them).
    pub fn try_recv<T, const N: usize>(&self) -> Result<Option<T>, RobotError>
    where
        T: TryFrom<[u8; N]>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        let swarm = self
            .swarm
            .as_ref()
            .ok_or_else(|| RobotError::new(RobotErrorType::NotJoined))?;
        match swarm.try_recv_raw().map_err(|e| transport_err("recv", e))? {
            None => Ok(None),
            Some(bytes) => T::try_from(bytes).map(Some).map_err(|e| {
                RobotError::new(RobotErrorType::DecodeError(e.to_string())).caused_by(e)
            }),
        }
    }
}

fn transport_err(stage: &str, err: std::io::Error) -> RobotError {
    RobotError::new(RobotErrorType::TransportError(format!("{stage}: {err}"))).caused_by(err)
}

impl Robot for DistRobot {
    fn get_internal(&self) -> &RobotInternal {
        &self.env
    }
}

impl TryFrom<SocketAddr> for RobotInternal {
    type Error = MazeError;

    fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
        let maze = DistMazeClient::try_from(value)?;

        Ok(RobotInternal::new(maze))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Direction};
    use rstest::rstest;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener};
    use std::thread;

    // One-shot mock DistMazeServer: accepts one connection, consumes the op byte, writes
    // `response`, then closes. Decouples DistRobot tests from DistMazeServer implementation.
    fn one_shot_mock(response: &'static [u8]) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1];
            stream.read_exact(&mut buf).ok(); // consume the op byte sent by DistMazeClient
            stream.write_all(response).ok();
        });
        addr
    }

    fn make_robot(server_addr: SocketAddr) -> DistRobot {
        DistRobot::try_build(server_addr).expect("robot connects to server successfully")
    }

    // --- peek tests ---

    #[rstest]
    fn test_peek_wall(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) {
        // each direction gets its own fresh mock and robot connection; server always returns Wall
        let robot = make_robot(one_shot_mock(b"\x03\x00\x00\x00\x00"));
        assert_eq!(robot.peek(direction).unwrap(), Cell::Wall);
    }

    #[rstest]
    fn test_peek_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) {
        let robot = make_robot(one_shot_mock(b"\x02\x00\x00\x00\x00"));
        assert_eq!(robot.peek(direction).unwrap(), Cell::Open);
    }

    #[test]
    fn test_peek_finish() {
        let robot = make_robot(one_shot_mock(b"\x00\x00\x00\x00\x00"));
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Finish);
    }

    #[test]
    fn test_peek_occupied() {
        // Cell::Occupied has no TextMaze analogue — unique to multi-robot distributed mazes
        let robot = make_robot(one_shot_mock(b"\x01\x01\x00\x00\x00"));
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Occupied(1));
    }

    #[test]
    fn test_peek_returns_err_on_server_error() {
        // 0xFF is the move-success sentinel, not a valid cell byte; must propagate as Err
        let robot = make_robot(one_shot_mock(b"\xff\x00\x00\x00\x00"));
        assert!(robot.peek(Direction::North).is_err());
    }

    // --- go tests ---

    #[rstest]
    fn test_go_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) -> Result<(), MazeError> {
        // server returns the move-success sentinel 0xFF; go must return Ok for every direction
        let robot = make_robot(one_shot_mock(b"\xff\x00\x00\x00\x00"));
        robot.go(direction)
    }

    #[test]
    fn test_go_returns_err_on_move_failure() {
        // 'E' (0x45, first byte of "Error") is not 0xFF; client returns Err, robot propagates it
        let robot = make_robot(one_shot_mock(b"E\x00\x00\x00\x00"));
        assert!(robot.go(Direction::North).is_err());
    }

    // --- swarm tests ---
    mod swarm {
        use super::*;
        use std::error::Error;
        use std::fmt::Display;
        use std::sync::atomic::{AtomicU16, Ordering};
        use std::time::{Duration, Instant};

        // Each test claims a unique port so concurrent runs and stale packets from prior tests
        // can't bleed across. Starts well above the ephemeral range used by `one_shot_mock`.
        static NEXT_PORT: AtomicU16 = AtomicU16::new(49500);
        fn next_port() -> u16 {
            NEXT_PORT.fetch_add(1, Ordering::SeqCst)
        }

        // Build a robot with both the (dummy) maze TCP server and the swarm transport ready. The
        // maze mock just parks the connection — swarm tests never exercise peek/go.
        fn make_swarm_robot(port: u16) -> DistRobot {
            DistRobot::try_build(one_shot_mock(b"\x02"))
                .expect("robot connects to maze mock")
                .join_swarm_local(port)
                .expect("robot joins swarm")
        }

        // Simple String<->[u8; 32] codec via TryFrom/TryInto, used by every test.
        #[derive(Debug, Clone, PartialEq)]
        struct Msg(String);

        #[derive(Debug)]
        struct MsgError(String);

        impl Display for MsgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl Error for MsgError {}

        impl TryFrom<[u8; 32]> for Msg {
            type Error = MsgError;

            fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
                let trimmed: Vec<u8> = bytes.iter().copied().take_while(|&b| b != 0).collect();
                String::from_utf8(trimmed)
                    .map(Msg)
                    .map_err(|e| MsgError(format!("Error parsing bytes {bytes:?}:\n{e}")))
            }
        }

        impl TryFrom<Msg> for [u8; 32] {
            type Error = MsgError;

            fn try_from(msg: Msg) -> Result<Self, Self::Error> {
                let mut as_vec = Vec::from(msg.0.as_bytes());

                // pad vec w/ null bytes to get length to 32
                while as_vec.len() < 32 {
                    as_vec.push(b"\0"[0]);
                }

                as_vec.try_into().map_err(|e| {
                    MsgError(format!(
                        "Failed to encode {msg:?} to [u8; 32]; vec {e:?} too long"
                    ))
                })
            }
        }

        // Pull every available Msg off `r` for at most `total`, returning the strings received.
        fn drain(robot: &DistRobot, total: Duration) -> Vec<String> {
            let deadline = Instant::now() + total;
            let mut out = Vec::new();

            while Instant::now() < deadline {
                match robot.try_recv::<Msg, 32>() {
                    Ok(Some(m)) => out.push(m.0),
                    Ok(None) => thread::sleep(Duration::from_millis(5)),
                    Err(e) => panic!("try_recv failed: {e}"),
                }
            }

            out
        }

        #[test]
        fn two_robots_exchange_messages() {
            // Two robots on the same port should each receive the other's broadcast — and never
            // their own.
            let port = next_port();
            let a = make_swarm_robot(port);
            let b = make_swarm_robot(port);

            a.try_send(Msg("from-a".into())).expect("a sends");
            b.try_send(Msg("from-b".into())).expect("b sends");

            let ra = drain(&a, Duration::from_millis(200));
            let rb = drain(&b, Duration::from_millis(200));

            assert!(
                ra.contains(&"from-b".to_string()),
                "a missing b's msg: {ra:?}"
            );
            assert!(
                rb.contains(&"from-a".to_string()),
                "b missing a's msg: {rb:?}"
            );
            assert!(!ra.contains(&"from-a".to_string()), "a saw own msg: {ra:?}");
            assert!(!rb.contains(&"from-b".to_string()), "b saw own msg: {rb:?}");
        }

        #[test]
        fn three_robots_each_receive_others_messages() {
            // All three on the same port; each one's broadcast must reach the other two and not
            // itself.
            let port = next_port();
            let a = make_swarm_robot(port);
            let b = make_swarm_robot(port);
            let c = make_swarm_robot(port);

            a.try_send(Msg("from-a".into())).unwrap();
            b.try_send(Msg("from-b".into())).unwrap();
            c.try_send(Msg("from-c".into())).unwrap();

            let ra = drain(&a, Duration::from_millis(200));
            let rb = drain(&b, Duration::from_millis(200));
            let rc = drain(&c, Duration::from_millis(200));

            for (name, recv, own) in [
                ("a", &ra, "from-a"),
                ("b", &rb, "from-b"),
                ("c", &rc, "from-c"),
            ] {
                assert!(
                    !recv.contains(&own.to_string()),
                    "{name} saw own msg ({own}): {recv:?}"
                );
                for other in ["from-a", "from-b", "from-c"].iter().filter(|m| **m != own) {
                    assert!(
                        recv.contains(&other.to_string()),
                        "{name} missing {other}: {recv:?}"
                    );
                }
            }
        }

        #[test]
        fn try_recv_returns_none_when_no_messages_waiting() {
            // Fresh robot with no senders: try_recv must yield Ok(None), not block or error.
            let port = next_port();
            let a = make_swarm_robot(port);
            assert!(matches!(a.try_recv::<Msg, 32>(), Ok(None)));
        }

        #[test]
        fn send_without_join_swarm_returns_not_joined() {
            // join_swarm was never called; send must surface NotJoined.
            let r = make_robot(one_shot_mock(b"\x02"));
            let err = r.try_send(Msg("x".into())).expect_err("send must fail");
            assert!(matches!(err.get_type(), RobotErrorType::NotJoined));
        }

        #[test]
        fn try_recv_without_join_swarm_returns_not_joined() {
            // join_swarm was never called; try_recv must surface NotJoined.
            let r = make_robot(one_shot_mock(b"\x02"));
            let err = r.try_recv::<Msg, 32>().expect_err("try_recv must fail");
            assert!(matches!(err.get_type(), RobotErrorType::NotJoined));
        }
    }
}
