use std::fs::read_to_string;

use clap::Parser;
use maze_robot::solve;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text = read_to_string(app.maze_file)?;

    println!("Solution: {:#?}", solve(maze_text.as_str())?);

    Ok(())
}
