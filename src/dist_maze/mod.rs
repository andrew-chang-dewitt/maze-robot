mod maze_impl;
mod robot_impl;
mod swarm;
mod tcp_server;

pub use maze_impl::{DistMazeClient, DistMazeServer};
pub use robot_impl::DistRobot;
pub use tcp_server::TcpServer;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{Cell, Direction};
    use crate::text_maze::MultiTextMaze;
    use crate::traits::Robot;
    use std::net::SocketAddr;

    //  +++++
    //  S + F
    //  +   +
    //  +++++
    // S at char-index 6 (including the \n separators).
    // Only open path from start: E→idx7, then S→idx13, E→14, E→15, N→9; peek E at idx9 → Finish.
    const MAZE: &str = "+++++\nS + F\n+   +\n+++++";

    // Creates a fresh MultiTextMaze server and a connected DistRobot.
    // Server is spawned first so it can accept; bot auto-registers on first connect.
    fn setup() -> DistRobot {
        let maze = MultiTextMaze::try_from(MAZE).expect("maze created");
        let server = DistMazeServer::try_from((
            maze,
            "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        ))
        .expect("server created");
        let server_addr = server.local_addr().unwrap();
        // start() spawns its own thread and returns a shutdown closure; dropping the closure
        // detaches the worker thread, which is fine here — the test process exit reaps it.
        let _shutdown = server.start().expect("server starts");
        DistRobot::try_build(server_addr).expect("robot connects")
    }

    #[test]
    fn peek_north_from_start_is_wall() {
        // bot starts at S; the row above is all '+'
        let robot = setup();
        assert_eq!(robot.peek(Direction::North).unwrap(), Cell::Wall);
    }

    #[test]
    fn peek_east_from_start_is_open() {
        // cell east of S is a space — the only open direction from start
        let robot = setup();
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Open);
    }

    #[test]
    fn go_east_then_peek_east_is_wall() {
        // after moving east from S, the next cell east is '+'; server must reflect updated position
        let robot = setup();
        robot.go(Direction::East).expect("go east succeeds");
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Wall);
    }

    #[test]
    fn traverse_path_peek_east_is_finish() {
        // walk the only open path through the maze; peeking east from idx 9 must yield Finish
        let robot = setup();
        robot.go(Direction::East).expect("go east");
        robot.go(Direction::South).expect("go south");
        robot.go(Direction::East).expect("go east");
        robot.go(Direction::East).expect("go east");
        robot.go(Direction::North).expect("go north");
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Finish);
    }
}
