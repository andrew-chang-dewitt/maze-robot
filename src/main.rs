use std::fs::read_to_string;

use anyhow::Context;
use clap::Parser;

use maze_robot::Robot;

// mod solution;
mod graph;

// use solution::{find_solution, render_solution};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text =
        read_to_string(app.maze_file).context("Failed to load maze from file {app.maze_file}")?;
    let (_robot, _start) = Robot::try_new(maze_text.as_str()).unwrap();

    todo!()
    // let solution = find_solution(robot, start)
    //     .context("Error encountered while finding solution")
    //     .map(|solution| render_solution(solution))?;

    // println!("Maze solved!\n\n{solution}");

    // Ok(())
}
