//! maze node for a swarm of bots to explore simultaneously.
//! exposes capability for bots to inspect neighboring cells & move to them via tcp sockets & by
//! internally maintaining state of each bot's current location (but doesn't expose anything for
//! tracking current location for bots to access).
use std::{
    fs::read_to_string,
    net::{SocketAddr, ToSocketAddrs},
};

use anyhow::anyhow;
use clap::Parser;

use maze_robot::{dist_maze::DistMazeServer, text_maze::MultiTextMaze};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text = read_to_string(app.maze_file)?;
    let maze_underlying = MultiTextMaze::try_from(maze_text)?;
    let socket = "0.0.0.0:0".parse::<SocketAddr>().expect("valid address");
    let mut maze = DistMazeServer::try_from((maze_underlying, socket))?;

    maze.start().map_err(|e| e.into())
}
