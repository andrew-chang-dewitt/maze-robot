use std::net::SocketAddr;

use crate::{
    dist_maze::DistMazeClient,
    traits::{MazeError, MazeErrorType, Robot, RobotInternal},
};

use super::swarm::Swarm;

/// Implementation of [`crate::traits::Robot`] that queries maze environment (an instance of
/// [`crate::dist_maze::DistMazeServer`]) via tcp sockets using internal
/// [`crate::dist_maze::DistMazeClient`] instance.
#[derive(Debug)]
pub struct DistRobot {
    env: RobotInternal,
    local_addr: SocketAddr,
    // swarm: Swarm,
}

impl DistRobot {
    /// Create a new distributed robot by telling it at what address to find the distributed maze.
    ///
    /// Initializes a DistMazeClient connects it to the DistMazeServer at the address provided.
    pub fn try_build(maze_addr: SocketAddr) -> Result<Self, MazeError> {
        let maze = DistMazeClient::try_from(maze_addr)?;
        let local_addr = maze.local_addr().map_err(|e| {
            MazeError::new(MazeErrorType::CreationError(e.to_string())).caused_by(e)
        })?;
        Ok(Self {
            env: RobotInternal::new(maze),
            local_addr,
            // swarm: Swarm::new(),
        })
    }

    /// Returns the local socket address of the underlying connection to the maze server.
    ///
    /// Use this to register the robot with a [`crate::dist_maze::DistMazeServer`] via
    /// [`crate::dist_maze::DistMazeServer::register_bot`] before starting the server.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    // /// Initialize Swarm connection (UdpSockets)
    // pub fn swarm_join(&mut self) {
    //     todo!()
    // }

    // /// Shutdown Swarm connection
    // pub fn swarm_leave(&mut self) {
    //     todo!()
    // }
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
        let robot = make_robot(one_shot_mock(b"\x03"));
        assert_eq!(robot.peek(direction).unwrap(), Cell::Wall);
    }

    #[rstest]
    fn test_peek_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) {
        let robot = make_robot(one_shot_mock(b"\x02"));
        assert_eq!(robot.peek(direction).unwrap(), Cell::Open);
    }

    #[test]
    fn test_peek_finish() {
        let robot = make_robot(one_shot_mock(b"\x00"));
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Finish);
    }

    #[test]
    fn test_peek_occupied() {
        // Cell::Occupied has no TextMaze analogue — unique to multi-robot distributed mazes
        let robot = make_robot(one_shot_mock(b"\x01"));
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Occupied);
    }

    #[test]
    fn test_peek_returns_err_on_server_error() {
        // 0xFF is the move-success sentinel, not a valid cell byte; must propagate as Err
        let robot = make_robot(one_shot_mock(b"\xff"));
        assert!(robot.peek(Direction::North).is_err());
    }

    // --- go tests ---

    #[rstest]
    fn test_go_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) -> Result<(), MazeError> {
        // server returns the move-success sentinel 0xFF; go must return Ok for every direction
        let robot = make_robot(one_shot_mock(b"\xff"));
        robot.go(direction)
    }

    #[test]
    fn test_go_returns_err_on_move_failure() {
        // 'E' (0x45, first byte of "Error") is not 0xFF; client returns Err, robot propagates it
        let robot = make_robot(one_shot_mock(b"E"));
        assert!(robot.go(Direction::North).is_err());
    }
}
