//! maze node for a swarm of bots to explore simultaneously.
//! exposes capability for bots to inspect neighboring cells & move to them via tcp sockets & by
//! internally maintaining state of each bot's current location (but doesn't expose anything for
//! tracking current location for bots to access).
use std::{
    fs::read_to_string,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use clap::Parser;

use maze_robot::{dist_maze::DistMazeServer, text_maze::MultiTextMaze};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    /// Source file for maze to serve.
    maze_file: String,
    /// TCP port to listen on.
    port: u16,
}

fn main() -> anyhow::Result<()> {
    let App { maze_file, port } = App::parse();
    let maze = read_to_string(maze_file)?;
    let server = make_server(maze, format!("0.0.0.0:{port}").as_str())?;

    // start() spawns the worker thread and returns a one-shot shutdown closure.
    let shutdown = server.start()?;

    // Wire shutdown into Ctrl-C. The closure is FnOnce, so park it in Mutex<Option<_>> and
    // take() on the first signal. Use a channel to deliver the shutdown's result back to main.
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let slot: Arc<
        Mutex<Option<Box<dyn FnOnce() -> Result<(), maze_robot::traits::MazeError> + Send>>>,
    > = Arc::new(Mutex::new(Some(Box::new(shutdown))));
    let slot_for_handler = Arc::clone(&slot);
    ctrlc::set_handler(move || {
        if let Some(stop) = slot_for_handler.lock().unwrap().take() {
            let _ = result_tx.send(stop());
        }
    })
    .map_err(|e| anyhow!("failed to install Ctrl-C handler: {e}"))?;

    // Block main until the handler runs shutdown and reports its result.
    match result_rx.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(anyhow!("shutdown channel closed before signal: {e}")),
    }
}

fn make_server(text: String, addr: &str) -> anyhow::Result<DistMazeServer<MultiTextMaze>> {
    let maze = MultiTextMaze::try_from(text)?;
    let socket = addr
        .parse::<SocketAddr>()
        .map_err(|e| anyhow!("invalid address {addr}: {e}"))?;
    let server = DistMazeServer::try_from((maze, socket))?;
    Ok(server)
}

#[cfg(test)]
mod tests {
    use maze_robot::{Cell, Direction, dist_maze::DistRobot, traits::Robot};
    use std::{net::SocketAddr, thread, time::Duration};

    use super::*;

    //  +++++
    //  S + F
    //  +   +
    //  +++++
    // S at char-index 6 (including the \n separators).
    // Only open path from start: E→idx7, then S→idx13, E→14, E→15, N→9; peek E at idx9 → Finish.
    const MAZE: &str = "+++++\nS + F\n+   +\n+++++";
    const ADDR: &str = "127.0.0.1:4000";

    #[test]
    fn test_solution() {
        let server = make_server(String::from(MAZE), ADDR).expect("server builds");
        let shutdown = server.start().expect("server starts");

        // FIXME: robot should do a try-connect loop to avoid needing this
        thread::sleep(Duration::from_millis(100));

        let robot = DistRobot::try_build(ADDR.parse::<SocketAddr>().expect("valid address"))
            .expect("robot connects");

        assert_eq!(
            robot.peek(Direction::North).unwrap(),
            Cell::Wall,
            "north from start should be wall"
        );
        robot.go(Direction::East).expect("go east");
        robot.go(Direction::South).expect("go south");
        robot.go(Direction::East).expect("go east");
        robot.go(Direction::East).expect("go east");
        robot.go(Direction::North).expect("go north");
        assert_eq!(robot.peek(Direction::East).unwrap(), Cell::Finish);

        // Close the client connection so the server's per-stream read loop exits; then signal
        // shutdown so the server's outer accept loop exits; then join the worker thread.
        drop(robot);
        shutdown().expect("server shuts down cleanly");
    }
}
