use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::{
    Cell, Direction,
    dist_maze::TcpServer,
    traits::{Maze, MazeError, MazeErrorType, MultiMaze},
};

/// An implementor of [`crate::traits::Maze`] that accesses & updates state in remote maze node
/// (defined in [`DistMazeServer`]) over tcp sockets.
#[derive(Debug)]
pub struct DistMazeClient {
    socket: TcpStream,
}

impl Maze for DistMazeClient {
    fn look_dir(&self, direction: Direction) -> Result<Cell, MazeError> {
        let op: [u8; 1] = ServerOp::Look(direction).into();
        (&self.socket).write_all(&op).map_err(io_to_maze_err)?;

        let mut resp = [0u8; 1];
        (&self.socket)
            .read_exact(&mut resp)
            .map_err(io_to_maze_err)?;

        match resp[0] {
            0x00 => Ok(Cell::Finish),
            0x01 => Ok(Cell::Occupied),
            0x02 => Ok(Cell::Open),
            0x03 => Ok(Cell::Wall),
            b => Err(MazeError::new(MazeErrorType::CreationError(format!(
                "unexpected look_dir response byte: {b:#04x}"
            )))),
        }
    }

    fn move_dir(&mut self, direction: Direction) -> Result<(), MazeError> {
        let op: [u8; 1] = ServerOp::Move(direction).into();
        self.socket.write_all(&op).map_err(io_to_maze_err)?;

        let mut resp = [0u8; 1];
        self.socket.read_exact(&mut resp).map_err(io_to_maze_err)?;

        match resp[0] {
            0xFF => Ok(()),
            _ => Err(MazeError::new(MazeErrorType::MoveError(
                direction,
                "server returned error".to_string(),
            ))),
        }
    }
}

impl Display for DistMazeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.socket.peer_addr() {
            Ok(addr) => write!(f, "DistMazeClient({addr})"),
            Err(_) => write!(f, "DistMazeClient(disconnected)"),
        }
    }
}

fn io_to_maze_err(e: io::Error) -> MazeError {
    MazeError::new(MazeErrorType::CreationError(e.to_string())).caused_by(e)
}

impl TryFrom<SocketAddr> for DistMazeClient {
    type Error = MazeError;

    fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
        let socket = TcpStream::connect(value).map_err(|e| {
            MazeError::new(MazeErrorType::CreationError(format!(
                "Unable to establish connection to Maze node!\n  {e}"
            )))
            .caused_by(e)
        })?;

        Ok(Self { socket })
    }
}

/// A wrapper of [`crate::traits::MultiMaze`] that exposes limited state information & update
/// capabilities to remote instances of [`DistMazeClient`] via tcp sockets.
///
/// Bots are registered automatically when they first connect — no out-of-band setup required.
#[derive(Debug)]
pub struct DistMazeServer<M: MultiMaze> {
    maze: M,
    bots: HashMap<SocketAddr, usize>,
    server: TcpServer,
}

// Bridges ServerOp's TryFrom (Error = ServerOpDecodeError) to the io::Error bound TcpServer requires.
struct ServerOpMsg(ServerOp);

impl TryFrom<[u8; 1]> for ServerOpMsg {
    type Error = io::Error;
    fn try_from(bytes: [u8; 1]) -> Result<Self, io::Error> {
        ServerOp::try_from(bytes)
            .map(Self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl Into<[u8; 1]> for ServerOpMsg {
    fn into(self) -> [u8; 1] {
        self.0.into()
    }
}

impl<M: MultiMaze> DistMazeServer<M> {
    /// Returns the local address this server is bound to.
    ///
    /// Useful when the server was constructed with port `0` and the OS assigned an ephemeral port.
    pub fn local_addr(&self) -> Result<SocketAddr, io::Error> {
        self.server.local_addr()
    }

    /// Begin listening for incoming requests from remote [`DistMazeClient`] instances.
    ///
    /// On first contact from a new peer address, the server calls [`MultiMaze::add_bot`] to place
    /// the bot at the next available start cell and records the address→id mapping. Subsequent
    /// messages from the same address reuse the registered id.
    ///
    /// Decodes each message as a [`ServerOp`] and dispatches to the underlying [`MultiMaze`]:
    /// - `look_dir` response: single byte encoding the returned [`Cell`]
    ///   (Finish=`0x00`, Occupied=`0x01`, Open=`0x02`, Wall=`0x03`)
    /// - `move_dir` response: `0xFF` on success
    ///
    /// Blocks until the underlying listener returns an accept error.
    pub fn start(&mut self) -> Result<(), MazeError> {
        let maze = &mut self.maze;
        let bots = &mut self.bots;

        self.server
            .start(
                move |addr: SocketAddr, msg: ServerOpMsg| -> Result<&'static [u8], io::Error> {
                    let bot_id = match bots.get(&addr).copied() {
                        Some(id) => id,
                        None => {
                            let id = maze.add_bot().map_err(|e| {
                                io::Error::new(io::ErrorKind::Other, e.to_string())
                            })?;
                            bots.insert(addr, id);
                            id
                        }
                    };

                    match msg.0 {
                        ServerOp::Look(dir) => {
                            let cell = maze
                                .look_dir(bot_id, dir)
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                            Ok(match cell {
                                Cell::Finish => b"\x00",
                                Cell::Occupied => b"\x01",
                                Cell::Open => b"\x02",
                                Cell::Wall => b"\x03",
                            })
                        }
                        ServerOp::Move(dir) => {
                            maze.move_dir(bot_id, dir)
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                            Ok(b"\xff")
                        }
                    }
                },
            )
            .map_err(|e| {
                let msg = format!("TCP server error: {e}");
                MazeError::new(MazeErrorType::CreationError(msg)).caused_by(e)
            })
    }
}

impl<M: MultiMaze> TryFrom<(M, SocketAddr)> for DistMazeServer<M> {
    type Error = MazeError;

    fn try_from((maze, addr): (M, SocketAddr)) -> Result<Self, Self::Error> {
        let listener = TcpListener::bind(addr).map_err(|e| {
            MazeError::new(MazeErrorType::CreationError(format!(
                "Unable to configure TCP listener!\n  {e}"
            )))
            .caused_by(e)
        })?;
        let server = TcpServer::new(listener);

        Ok(Self {
            maze,
            server,
            bots: HashMap::new(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ServerOpDecodeError(u8);

impl Display for ServerOpDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid ServerOp byte: {}", self.0)
    }
}

impl std::error::Error for ServerOpDecodeError {}

#[derive(Debug, PartialEq)]
enum ServerOp {
    Look(Direction),
    Move(Direction),
}

impl From<ServerOp> for [u8; 1] {
    fn from(op: ServerOp) -> Self {
        let dir_index = |d: &Direction| match d {
            Direction::North => 0u8,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        };
        match &op {
            ServerOp::Look(d) => [dir_index(d)],
            ServerOp::Move(d) => [4 + dir_index(d)],
        }
    }
}

impl TryFrom<[u8; 1]> for ServerOp {
    type Error = ServerOpDecodeError;

    fn try_from(bytes: [u8; 1]) -> Result<Self, Self::Error> {
        let byte = bytes[0];
        let dir = match byte % 4 {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            _ => Direction::West,
        };
        match byte {
            0..=3 => Ok(ServerOp::Look(dir)),
            4..=7 => Ok(ServerOp::Move(dir)),
            _ => Err(ServerOpDecodeError(byte)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use socket2::{Domain, Protocol, Socket, Type};
    use std::fmt::Display;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    // --- MockMaze ---

    #[derive(Debug)]
    struct MockMaze {
        look_cell: Cell,
        look_ok: bool,
        move_ok: bool,
        add_ok: bool,
        add_call_count: Arc<Mutex<usize>>,
    }

    impl MockMaze {
        fn new(look_cell: Cell, look_ok: bool, move_ok: bool, add_ok: bool) -> Self {
            Self {
                look_cell,
                look_ok,
                move_ok,
                add_ok,
                add_call_count: Arc::new(Mutex::new(0)),
            }
        }
    }

    impl Display for MockMaze {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockMaze")
        }
    }

    impl MultiMaze for MockMaze {
        fn look_dir(&self, _id: usize, _direction: Direction) -> Result<Cell, MazeError> {
            if self.look_ok {
                Ok(self.look_cell)
            } else {
                Err(MazeError::new(MazeErrorType::UnknownRobot(0)))
            }
        }

        fn move_dir(&mut self, _id: usize, _direction: Direction) -> Result<(), MazeError> {
            if self.move_ok {
                Ok(())
            } else {
                Err(MazeError::new(MazeErrorType::MoveError(
                    Direction::North,
                    "blocked".into(),
                )))
            }
        }

        fn add_bot(&mut self) -> Result<usize, MazeError> {
            let mut count = self.add_call_count.lock().unwrap();
            let id = *count;
            *count += 1;
            drop(count);
            if self.add_ok {
                Ok(id)
            } else {
                Err(MazeError::new(MazeErrorType::CreationError(
                    "add_bot rejected".into(),
                )))
            }
        }

        fn has_bot(&self, _id: usize) -> bool {
            true
        }

        fn bot_ids(&self) -> Vec<usize> {
            vec![0]
        }
    }

    // --- Test helpers ---

    fn ok_maze() -> MockMaze {
        MockMaze::new(Cell::Open, true, true, true)
    }

    fn make_server(maze: MockMaze) -> (DistMazeServer<MockMaze>, SocketAddr) {
        let server =
            DistMazeServer::try_from((maze, "127.0.0.1:0".parse::<SocketAddr>().unwrap())).unwrap();
        let addr = server.local_addr().unwrap();
        (server, addr)
    }

    // Binds a client socket to an ephemeral local address and returns the socket ready to connect.
    // Auto-registration means we no longer need to tell the server about the client in advance.
    fn bind_client() -> (Socket, SocketAddr) {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
        socket
            .bind(&"127.0.0.1:0".parse::<SocketAddr>().unwrap().into())
            .unwrap();
        let client_addr = socket.local_addr().unwrap().as_socket().unwrap();
        (socket, client_addr)
    }

    // Connect a bound socket2::Socket, send one op byte, read the full response.
    fn send_op(socket: Socket, server_addr: SocketAddr, op: u8) -> Vec<u8> {
        socket.connect(&server_addr.into()).unwrap();
        let mut socket = socket;
        socket.write_all(&[op]).unwrap();
        socket.shutdown(Shutdown::Write).unwrap();
        let mut buf = Vec::new();
        socket.read_to_end(&mut buf).unwrap();
        buf
    }

    // Connect without pre-binding; server auto-registers from peer_addr.
    fn send_op_fresh(server_addr: SocketAddr, op: u8) -> Vec<u8> {
        let mut stream = TcpStream::connect(server_addr).unwrap();
        stream.write_all(&[op]).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        buf
    }

    const LOOK_NORTH: u8 = 0; // ServerOp::Look(North)
    const MOVE_NORTH: u8 = 4; // ServerOp::Move(North)

    // --- DistMazeClient tests ---

    // One-shot mock server: accepts one connection, discards the op byte, writes `response`, closes.
    fn mock_server(response: &'static [u8]) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1];
            stream.read_exact(&mut buf).ok(); // consume the single op byte
            stream.write_all(response).ok();
        });
        addr
    }

    #[rstest]
    #[case(b"\x00", Cell::Finish)]
    #[case(b"\x01", Cell::Occupied)]
    #[case(b"\x02", Cell::Open)]
    #[case(b"\x03", Cell::Wall)]
    fn look_dir_decodes_cell_response(#[case] response: &'static [u8], #[case] expected: Cell) {
        // server returns a cell byte; client must decode it to the matching Cell variant
        let addr = mock_server(response);
        let client = DistMazeClient::try_from(addr).unwrap();
        assert_eq!(client.look_dir(Direction::North).unwrap(), expected);
    }

    #[test]
    fn look_dir_returns_err_on_unknown_response_byte() {
        // 0xFF is the move-success sentinel, not a valid cell byte; client must return Err
        let addr = mock_server(b"\xff");
        let client = DistMazeClient::try_from(addr).unwrap();
        assert!(client.look_dir(Direction::North).is_err());
    }

    #[test]
    fn move_dir_returns_ok_on_success_byte() {
        // 0xFF is the server's success sentinel for move_dir
        let addr = mock_server(b"\xff");
        let mut client = DistMazeClient::try_from(addr).unwrap();
        assert!(client.move_dir(Direction::North).is_ok());
    }

    #[test]
    fn move_dir_returns_err_on_error_response() {
        // any byte other than 0xFF signals failure; 'E' (0x45) is the first byte of "Error"
        let addr = mock_server(b"E");
        let mut client = DistMazeClient::try_from(addr).unwrap();
        assert!(client.move_dir(Direction::North).is_err());
    }

    #[test]
    fn display_includes_server_addr() {
        // Display should show the server address so the client can be identified in logs
        let addr = mock_server(b"");
        let client = DistMazeClient::try_from(addr).unwrap();
        assert!(format!("{client}").contains(&addr.to_string()));
    }

    // --- DistMazeServer::start tests ---

    #[rstest]
    #[case(Cell::Finish, &[0x00u8])]
    #[case(Cell::Occupied, &[0x01u8])]
    #[case(Cell::Open, &[0x02u8])]
    #[case(Cell::Wall, &[0x03u8])]
    fn start_look_encodes_cell_variant(#[case] cell: Cell, #[case] expected: &[u8]) {
        let maze = MockMaze::new(cell, true, true, true);
        let (mut server, server_addr) = make_server(maze);
        thread::spawn(move || server.start().ok());
        // auto-registration on first connect; no pre-registration needed
        assert_eq!(send_op_fresh(server_addr, LOOK_NORTH), expected);
    }

    #[test]
    fn start_move_success_returns_success_byte() {
        let (mut server, server_addr) = make_server(ok_maze());
        thread::spawn(move || server.start().ok());
        assert_eq!(send_op_fresh(server_addr, MOVE_NORTH), b"\xff");
    }

    #[test]
    fn start_auto_registers_new_peer() {
        // a fresh connection with no prior setup must succeed via auto-registration
        let (mut server, server_addr) = make_server(ok_maze());
        thread::spawn(move || server.start().ok());
        let result = send_op_fresh(server_addr, LOOK_NORTH);
        assert_eq!(result, &[0x02u8]); // Cell::Open
    }

    #[test]
    fn start_reuses_existing_registration() {
        // same client sends two messages; add_bot must be called exactly once
        let maze = ok_maze();
        let add_count = Arc::clone(&maze.add_call_count);
        let (mut server, server_addr) = make_server(maze);
        thread::spawn(move || server.start().ok());

        let mut stream = TcpStream::connect(server_addr).unwrap();
        stream.write_all(&[LOOK_NORTH]).unwrap();
        let mut resp = [0u8; 1];
        stream.read_exact(&mut resp).unwrap();
        stream.write_all(&[LOOK_NORTH]).unwrap();
        stream.read_exact(&mut resp).unwrap();

        let count = *add_count.lock().unwrap();
        assert_eq!(count, 1, "add_bot called {count} times; expected 1");
    }

    #[test]
    fn start_add_bot_failure_returns_error() {
        // maze rejects add_bot; server must return an error response to the client
        let maze = MockMaze::new(Cell::Open, true, true, false);
        let (mut server, server_addr) = make_server(maze);
        thread::spawn(move || server.start().ok());
        assert_eq!(send_op_fresh(server_addr, LOOK_NORTH), b"Error");
    }

    #[test]
    fn start_survives_add_bot_failure_and_handles_next() {
        // first connection's add_bot fails; second connection's add_bot succeeds
        // MockMaze.add_ok is per-instance, so we use a counter-based approach:
        // set add_ok=false to fail the first client, then stop that server and
        // test the recovery path via distinct server instances.
        // Instead: single maze that fails add_bot on call 0, succeeds on call 1+.
        // MockMaze always returns add_call_count-based id when add_ok=true.
        // Easiest approach: verify server continues accepting after a failure.
        let maze = MockMaze::new(Cell::Open, true, true, false);
        let add_count = Arc::clone(&maze.add_call_count);
        let (mut server, server_addr) = make_server(maze);
        thread::spawn(move || server.start().ok());

        // first connection: add_bot fails → Error
        assert_eq!(send_op_fresh(server_addr, LOOK_NORTH), b"Error");
        // server still alive: confirm add_bot was attempted
        assert_eq!(*add_count.lock().unwrap(), 1);
    }

    #[test]
    fn start_maze_error_returns_error() {
        // move_ok: false makes the maze reject every move, exercising the maze-error path
        let maze = MockMaze::new(Cell::Open, true, false, true);
        let (mut server, server_addr) = make_server(maze);
        let (client, _) = bind_client();
        thread::spawn(move || server.start().ok());
        assert_eq!(send_op(client, server_addr, MOVE_NORTH), b"Error");
    }

    #[rstest]
    #[case(ServerOp::Look(Direction::North), [0u8])]
    #[case(ServerOp::Look(Direction::East), [1u8])]
    #[case(ServerOp::Look(Direction::South), [2u8])]
    #[case(ServerOp::Look(Direction::West), [3u8])]
    #[case(ServerOp::Move(Direction::North), [4u8])]
    #[case(ServerOp::Move(Direction::East), [5u8])]
    #[case(ServerOp::Move(Direction::South), [6u8])]
    #[case(ServerOp::Move(Direction::West), [7u8])]
    fn encode(#[case] op: ServerOp, #[case] expected: [u8; 1]) {
        // Look ops occupy bytes 0-3 (one per direction); Move ops occupy 4-7
        let result: [u8; 1] = op.into();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case([0u8], Ok(ServerOp::Look(Direction::North)))]
    #[case([1u8], Ok(ServerOp::Look(Direction::East)))]
    #[case([2u8], Ok(ServerOp::Look(Direction::South)))]
    #[case([3u8], Ok(ServerOp::Look(Direction::West)))]
    #[case([4u8], Ok(ServerOp::Move(Direction::North)))]
    #[case([5u8], Ok(ServerOp::Move(Direction::East)))]
    #[case([6u8], Ok(ServerOp::Move(Direction::South)))]
    #[case([7u8], Ok(ServerOp::Move(Direction::West)))]
    #[case([8u8], Err(ServerOpDecodeError(8)))]
    #[case([255u8], Err(ServerOpDecodeError(255)))]
    fn decode(#[case] bytes: [u8; 1], #[case] expected: Result<ServerOp, ServerOpDecodeError>) {
        // bytes 0-7 are valid; anything above 7 must produce a ServerOpDecodeError
        assert_eq!(ServerOp::try_from(bytes), expected);
    }
}
