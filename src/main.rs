use std::fs::read_to_string;

use anyhow::Context;
use clap::Parser;

use maze_robot::{CardinalDirection, Robot};

mod solution;

use solution::{find_solution, render_solution};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text =
        read_to_string(app.maze_file).context("Failed to load maze from file {app.maze_file}")?;
    // TODO: how to detect starting direction?
    // for now, just assume all mazes start from the left edge,
    // facing to the right edge
    let starting_direction = CardinalDirection::East;
    let robot = Robot::try_from((maze_text.as_str(), starting_direction)).unwrap();

    let solution = find_solution(robot)
        .context("Error encountered while finding solution")
        .map(|solution| render_solution(solution))?;

    println!("Maze solved!\n\n{solution}");

    Ok(())
}
