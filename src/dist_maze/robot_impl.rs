use std::net::SocketAddr;

use crate::{
    dist_maze::DistMazeClient,
    traits::{MazeError, Robot, RobotInternal},
};

use super::swarm::Swarm;

/// Implementation of [`crate::traits::Robot`] that queries maze environment (an instance of
/// [`crate::dist_maze::DistMazeServer`]) via tcp sockets using internal
/// [`crate::dist_maze::DistMazeClient`] instance.
#[derive(Debug)]
pub struct DistRobot {
    env: RobotInternal,
    swarm: Swarm,
}

impl DistRobot {
    /// Create a new distributed robot by telling it at what address to find the distributed maze.
    ///
    /// Initializes a DistMazeClient connects it to the DistMazeServer at the address provided.
    pub fn try_build(maze_addr: SocketAddr) -> Result<Self, MazeError> {
        Ok(Self {
            env: RobotInternal::try_from(maze_addr)?,
            swarm: Swarm::new(),
        })
    }

    /// Initialize Swarm connection (UdpSockets)
    pub fn swarm_join(&mut self) {
        todo!()
    }

    /// Shutdown Swarm connection
    pub fn swarm_leave(&mut self) {
        todo!()
    }
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

// TODO: start some basic tests to drive development of underlying Swarm, DistMazeClient, &
// DistMazeServer components. make sure to follow existing tests for TextRobot as much as possible
