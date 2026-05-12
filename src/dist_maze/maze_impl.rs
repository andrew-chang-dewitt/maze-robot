use std::{
    collections::HashMap,
    fmt::Display,
    io,
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
    fn look_dir(&self, direction: crate::Direction) -> Result<Cell, MazeError> {
        todo!()
    }

    fn move_dir(&mut self, direction: crate::Direction) -> Result<(), crate::traits::MazeError> {
        todo!()
    }
}

impl Display for DistMazeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
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

    /// Register a bot so the server can route incoming requests to the correct [`MultiMaze`] slot.
    ///
    /// Associates the connecting peer address `addr` with bot `id`. Subsequent messages received
    /// from `addr` are dispatched to the underlying [`MultiMaze`] using `id` as the bot identifier.
    /// Must be called before [`Self::start`] for each bot that will connect.
    pub fn register_bot(&mut self, addr: SocketAddr, id: usize) {
        self.bots.insert(addr, id);
    }

    /// Begin listening for incoming requests from remote [`DistMazeClient`] instances.
    ///
    /// Decodes each message as a [`ServerOp`] and dispatches to the underlying [`MultiMaze`]:
    /// - `look_dir` response: single byte encoding the returned [`Cell`]
    ///   (Finish=`0x00`, Occupied=`0x01`, Open=`0x02`, Wall=`0x03`)
    /// - `move_dir` response: `0xFF` on success
    ///
    /// Uses the bots map (populated via [`Self::register_bot`]) to resolve the peer address of
    /// each incoming connection to a bot id for [`MultiMaze`] dispatch. Connections from
    /// unregistered addresses receive an error response and do not interrupt the loop.
    ///
    /// Blocks until the underlying listener returns an accept error.
    pub fn start(&mut self) -> Result<(), MazeError> {
        let maze = &mut self.maze;
        let bots = &self.bots;

        self.server
            .start(
                move |addr: SocketAddr, msg: ServerOpMsg| -> Result<&'static [u8], io::Error> {
                    let bot_id = bots.get(&addr).copied().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("no bot registered for {addr}"),
                        )
                    })?;

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

    fn try_from(value: (M, SocketAddr)) -> Result<Self, Self::Error> {
        let (maze, addr) = value;
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
    use std::fmt::Display;
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpStream};
    use std::thread;
    use socket2::{Domain, Protocol, Socket, Type};

    // --- MockMaze ---

    #[derive(Debug)]
    struct MockMaze {
        look_cell: Cell,
        look_ok: bool,
        move_ok: bool,
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
    }

    // --- Test helpers ---

    fn ok_maze() -> MockMaze {
        MockMaze { look_cell: Cell::Open, look_ok: true, move_ok: true }
    }

    fn make_server(maze: MockMaze) -> (DistMazeServer<MockMaze>, SocketAddr) {
        let server = DistMazeServer::try_from((
            maze,
            "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        ))
        .unwrap();
        let addr = server.local_addr().unwrap();
        (server, addr)
    }

    // Binds a client socket to an ephemeral local address, registers it with the server as
    // bot `id`, and returns the socket ready to connect.
    fn bind_client(server: &mut DistMazeServer<MockMaze>, bot_id: usize) -> (Socket, SocketAddr) {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
        socket.bind(&"127.0.0.1:0".parse::<SocketAddr>().unwrap().into()).unwrap();
        let client_addr = socket.local_addr().unwrap().as_socket().unwrap();
        server.register_bot(client_addr, bot_id);
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

    // Connect without pre-binding (unregistered client).
    fn send_op_unregistered(server_addr: SocketAddr, op: u8) -> Vec<u8> {
        let mut stream = TcpStream::connect(server_addr).unwrap();
        stream.write_all(&[op]).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        buf
    }

    const LOOK_NORTH: u8 = 0; // ServerOp::Look(North)
    const MOVE_NORTH: u8 = 4; // ServerOp::Move(North)

    // --- DistMazeServer::start tests ---

    #[rstest]
    #[case(Cell::Finish, &[0x00u8])]
    #[case(Cell::Occupied, &[0x01u8])]
    #[case(Cell::Open, &[0x02u8])]
    #[case(Cell::Wall, &[0x03u8])]
    fn start_look_encodes_cell_variant(#[case] cell: Cell, #[case] expected: &[u8]) {
        // maze always returns `cell` regardless of direction; move_ok is irrelevant here
        let maze = MockMaze { look_cell: cell, look_ok: true, move_ok: true };
        let (mut server, server_addr) = make_server(maze);
        // bind before start so the local address is known and can be pre-registered
        let (client, _) = bind_client(&mut server, 0);
        thread::spawn(move || server.start().ok());
        // direction doesn't affect cell encoding; LOOK_NORTH keeps the case focused on the cell byte
        assert_eq!(send_op(client, server_addr, LOOK_NORTH), expected);
    }

    #[test]
    fn start_move_success_returns_success_byte() {
        let (mut server, server_addr) = make_server(ok_maze());
        let (client, _) = bind_client(&mut server, 0);
        thread::spawn(move || server.start().ok());
        // successful move_dir is encoded as a single 0xFF sentinel byte
        assert_eq!(send_op(client, server_addr, MOVE_NORTH), b"\xff");
    }

    #[test]
    fn start_unknown_bot_returns_error() {
        let (mut server, server_addr) = make_server(ok_maze());
        // no register_bot call — every peer address will be unknown to the server
        thread::spawn(move || server.start().ok());
        // handler returns Err for the unregistered address; TcpServer writes "Error" to the stream
        assert_eq!(send_op_unregistered(server_addr, LOOK_NORTH), b"Error");
    }

    #[test]
    fn start_maze_error_returns_error() {
        // move_ok: false makes the maze reject every move, exercising the maze-error path
        let maze = MockMaze { look_cell: Cell::Open, look_ok: true, move_ok: false };
        let (mut server, server_addr) = make_server(maze);
        let (client, _) = bind_client(&mut server, 0);
        thread::spawn(move || server.start().ok());
        // handler propagates the MazeError as Err; TcpServer writes "Error" to the stream
        assert_eq!(send_op(client, server_addr, MOVE_NORTH), b"Error");
    }

    #[test]
    fn start_survives_unknown_bot_and_handles_next() {
        let (mut server, server_addr) = make_server(ok_maze());
        let (client, _) = bind_client(&mut server, 0);
        thread::spawn(move || server.start().ok());
        // an unregistered connection must return an error without killing the loop
        assert_eq!(send_op_unregistered(server_addr, LOOK_NORTH), b"Error");
        // the registered client connects after the failure; server must still be accepting
        assert_eq!(send_op(client, server_addr, LOOK_NORTH), &[0x02u8]); // Cell::Open
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
