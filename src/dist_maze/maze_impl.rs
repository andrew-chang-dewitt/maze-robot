use std::{
    fmt::Display,
    io,
    net::{SocketAddr, TcpStream},
};

use crate::traits::{Maze, MazeError, MazeErrorType};

/// An implementor of [`crate::traits::Maze`] that accesses & updates state in remote maze node
/// (defined in [`DistMazeServer`]) over tcp sockets.
#[derive(Debug)]
pub struct DistMazeClient {
    socket: TcpStream,
}

impl Maze for DistMazeClient {
    fn look_dir(&self, direction: crate::Direction) -> crate::Cell {
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

/// A wrapper of [`crate::traits::Maze`] that exposes limited state information & update
/// capabilities to remote instances of [`DistMazeClient`] via tcp sockets.
#[derive(Debug)]
pub struct DistMazeServer<M: Maze> {
    maze: M,
}

impl<M: Maze> DistMazeServer<M> {}
