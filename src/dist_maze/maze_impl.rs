use std::{
    fmt::Display,
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
    server: TcpServer,
}

impl<M: MultiMaze> DistMazeServer<M> {}

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

        Ok(Self { maze, server })
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
        assert_eq!(ServerOp::try_from(bytes), expected);
    }
}
