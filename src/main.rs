use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::read_to_string,
};

use anyhow::anyhow;
use clap::Parser;

use maze_robot::Direction;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let maze_text = read_to_string(app.maze_file)?;

    let mut maze_iter = maze_text.chars().enumerate();
    let mut width: usize = 0;
    let mut start: Option<usize> = None;

    loop {
        match maze_iter.next() {
            Some((idx, 'S')) => start = Some(idx),
            Some((idx, '\n')) => {
                if width == 0 {
                    width = idx
                }
            }
            None => break,
            _ => (),
        }
    }

    println!("solving maze w/ following characteristics:");
    println!("width: {width}");
    println!("height: {}", maze_text.lines().count());
    println!("maze:\n{maze_text}");

    if let Some(idx) = start {
        let solution = solve_from(&maze_text.chars().collect(), width, idx)?
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(" -> ");
        println!("Maze solved! Solution: {solution}");

        return Ok(());
    }

    Err(anyhow!("Bad maze, must specify starting position with 'S'"))
}

fn solve_from(maze_chars: &Vec<char>, width: usize, start: usize) -> anyhow::Result<Vec<char>> {
    let mut to_visit = VecDeque::from([start]);
    let mut visited = HashSet::<usize>::new();
    let mut maze_graph = HashMap::<usize, [Option<usize>; 4]>::new();
    let mut parents = HashMap::<usize, usize>::new();

    loop {
        // process from front of queue
        if let Some(pos) = to_visit.pop_front() {
            // mark as visited
            visited.insert(pos);
            println!("processing {:?}", maze_chars.get(pos));

            // then inspect neighbors
            let mut neighbors = [None, None, None, None];
            let mut finish_found: Option<usize> = None;

            for (idx, &direction) in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ]
            .iter()
            .enumerate()
            {
                // update neighbors array for each direction
                neighbors[idx] = peek(&maze_chars, width, direction, pos);
                // println!("updated neighbors array @ {idx} with {:?}", neighbors[idx]);
                // handling valid positions
                if let Some(npos) = neighbors[idx] {
                    // track parent relationship
                    // only if neighbor not the parent of the current node
                    if Some(&npos) != parents.get(&pos) {
                        parents.insert(npos, pos);
                    }
                    // done if neighbor is F!
                    if let Some(&val) = maze_chars.get(npos) {
                        // update graph w/ found adj list for current cell
                        if val == 'F' {
                            finish_found = Some(npos);
                            break;
                        }
                    }

                    // otherwise, push unseen neighbor to queue
                    if !to_visit.contains(&npos) && !visited.contains(&npos) {
                        to_visit.push_back(npos);
                    }
                }
            }

            // update graph w/ found adj list for current cell
            maze_graph.insert(pos, neighbors);
            // println!("done processing {:?}", maze_chars.get(pos));
            // println!("queue: {to_visit:?}");
            // println!("parents map: {parents:?}");
            println!("maze:\n");
            println!("{}", render_maze(maze_chars, &maze_graph, pos));
            if let Some(finish) = finish_found {
                return Ok(trace_solution(parents, finish)
                    .iter()
                    .map(|&p| {
                        maze_chars
                            .get(p)
                            .expect("char will exist in maze")
                            .to_owned()
                    })
                    .collect());
            }
        } else {
            // if no more cells in queue, then we never found the end
            return Err(anyhow!("No solution found!"));
        }
    }
}

fn peek(maze: &Vec<char>, width: usize, direction: Direction, position: usize) -> Option<usize> {
    // println!("checking {direction:?} from {position}");

    let result = match direction {
        Direction::Up => {
            // no up if in top row
            if position < width {
                None
            } else {
                Some(position - width - 1)
            }
        }
        Direction::Right => {
            // no right if in right col
            let res = position + 1;

            if res > maze.len() || Some(&'\n') == maze.get(res) {
                None
            } else {
                Some(res)
            }
        }
        Direction::Down => {
            // no down if in bottom row
            let res = position + width + 1;

            if res > maze.len() { None } else { Some(res) }
        }
        Direction::Left => {
            // no left if in left column
            let res = position - 1;

            if position == 0 || Some(&'\n') == maze.get(res) {
                None
            } else {
                Some(res)
            }
        }
    };

    if let Some(neighbor) = result {
        let cell_value = maze
            .get(result.expect("will have neighbor position at this point"))
            .expect("maze position must exist at this point");

        // println!(
        //     "{direction:#?} from {:?} (@{position}) is {cell_value:?}",
        //     maze.get(position).unwrap()
        // );

        if cell_value == &'+' {
            None
        } else {
            Some(neighbor)
        }
    } else {
        None
    }
}

fn trace_solution(parents: HashMap<usize, usize>, finish: usize) -> Vec<usize> {
    // println!("[trace_solution] called with {parents:?}, {finish}");
    let mut out = Vec::new();
    let mut next = Some(&finish);
    // println!("[trace_solution] starting at {:?}", next);
    let mut i = 0;

    while next.is_some() && i < 10 {
        let curr = next.unwrap();
        out.push(curr.to_owned());
        // println!("[trace_solution] updated out: {:?}", out);
        next = parents.get(curr);
        // println!("[trace_solution] next is: {:?}", next);
        i += 1;
    }

    out.reverse();
    // println!("[trace_solution] returning: {:#?}", out);

    out
}

fn render_maze(
    maze_chars: &Vec<char>,
    maze_graph: &HashMap<usize, [Option<usize>; 4]>,
    position: usize,
) -> String {
    let mut discovered = HashSet::<usize>::new();

    for cell in maze_graph.values().flatten() {
        if let Some(p) = cell {
            discovered.insert(*p);
        }
    }

    maze_chars
        .iter()
        .enumerate()
        .map(|(idx, &val)| {
            if discovered.contains(&idx) {
                if idx == position {
                    "x".to_string()
                } else {
                    "·".to_string()
                }
            } else {
                val.to_string()
            }
        })
        .collect()
}
