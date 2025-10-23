use std::fs::read_to_string;

use anyhow::Context;
use clap::Parser;

use maze_robot::Robot;

mod dfs;

use dfs::{find_solution, render_solution};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let robot = Robot::try_from(read_to_string(app.maze_file).unwrap().as_str()).unwrap();

    let solution = find_solution(robot)
        .context("Error encountered while finding solution")
        .and_then(|(graph, stack)| render_solution(graph, stack))
        .context("Error encountered while rendering solution")?;

    println!("Maze solved!\n\n{solution}");

    Ok(())
}
