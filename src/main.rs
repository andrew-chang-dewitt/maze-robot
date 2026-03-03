use std::fs::read_to_string;

use clap::Parser;

mod solution;
mod text_maze;
use crate::solution::{render_solution, solve};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text = read_to_string(app.maze_file)?;
    let solution = solve(maze_text.as_str())?;

    println!("Solution:\n{}", render_solution(solution));

    Ok(())
}
