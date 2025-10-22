use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::read_to_string,
};

use anyhow::{Context, anyhow};
use clap::Parser;

use maze_robot::{Cell, Direction, Maze, Robot};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct App {
    maze_file: String,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let robot = Robot::try_from(read_to_string(app.maze_file).unwrap().as_str()).unwrap();

    let solution = find_solution_bfs(robot)
        .context("Error encountered while finding solution")
        .and_then(|(start, finish, graph, path)| render_path_bfs(start, finish, graph, path))
        .context("Error encountered while rendering solution")?;

    println!("Maze solved!\n\n{solution}");

    Ok(())
}

type ParentMap = HashMap<usize, usize>;
type CellGraph = HashMap<usize, [Option<usize>; 4]>;

const DIR_ARR: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

fn find_solution_bfs<M: Maze>(
    mut robot: Robot<M>,
) -> anyhow::Result<(usize, usize, CellGraph, ParentMap)> {
    // generate indicies for observed nodes
    let mut nxt_idx = 1;
    // track seen nodes that still need processed
    // initialized w/ starting position in queue (id 0)
    let mut to_visit = VecDeque::from([0]);
    // track visited nodes to avoid cycles
    let mut visited = HashSet::<usize>::new();
    // build graph of maze cells & neighbors as we go
    let mut graph: CellGraph = HashMap::new();
    // & track all traveled edges as parent-child relationships
    let mut path: ParentMap = HashMap::new();
    // finally, track current location
    let mut location = 0;

    // process from front of queue until empty
    while let Some(cell) = to_visit.pop_front() {
        println!("processing cell {cell} from {location} with\ngraph {graph:#?},\npath {path:#?}");
        // navigate robot to next cell
        location = navigate_bfs(&mut robot, location, cell, &graph, &path)?;
        // println!("moved robot to cell {location}");
        // mark as visitied
        visited.insert(cell);
        // println!("marked {location} as visited {visited:?}");
        // mark finish as not yet found
        let mut finish: Option<usize> = None;

        // get info to prefill neighbors array with parent cell
        let parent_direction = match location {
            0 => None,
            _ => path.get(&location),
        }
        .and_then(|pdx| graph.get(&pdx).map(|n| (pdx, n)))
        .and_then(|(pdx, n)| find_direction_to_neighbor(n, location).map(|dir| (pdx, dir)))
        .map(|(pdx, dir_p_to_l)| (pdx, dir_p_to_l.reverse()));

        let init_neighbors = match parent_direction {
            Some((&pdx, Direction::Up)) => [Some(pdx), None, None, None],
            Some((&pdx, Direction::Right)) => [None, Some(pdx), None, None],
            Some((&pdx, Direction::Left)) => [None, None, Some(pdx), None],
            Some((&pdx, Direction::Down)) => [None, None, None, Some(pdx)],
            None => [None, None, None, None],
        };

        // add cell & neighbors list to graph
        let neighbors: [Option<usize>; 4] = DIR_ARR.iter().enumerate().fold(
            init_neighbors,
            |mut acc, (idx, direction)| match robot.peek(*direction) {
                Cell::Open => {
                    // assign neighbor cell an index
                    let neighbor = nxt_idx;
                    // increment cell index any time the next is used
                    nxt_idx += 1;
                    // update neighbors list to include cell
                    acc[idx] = Some(neighbor);
                    // if not already visited
                    if !visited.contains(&neighbor) {
                        // add parent-child relationship for neighbor & location to path
                        path.insert(neighbor, location);
                        // push neighbor to queue
                        to_visit.push_back(neighbor);
                    }
                    // finally, return updated neighbors list
                    acc
                }
                Cell::Finish => {
                    // assign neighbor cell an index
                    let neighbor = nxt_idx;
                    // increment cell index any time the next is used
                    nxt_idx += 1;
                    // mark finish as found!
                    finish = Some(neighbor);
                    // update neighbors list to include finish
                    acc[idx] = Some(neighbor);
                    // add parent-child relationship for neighbor & location to path
                    path.insert(neighbor, location);
                    acc
                }
                Cell::Wall => acc,
            },
        );
        graph.insert(cell, neighbors);
    }

    Err(anyhow!("No solution found 😢"))
}

fn navigate_bfs<M: Maze>(
    robot: &mut Robot<M>,
    location: usize,
    target: usize,
    graph: &CellGraph,
    path: &ParentMap,
) -> anyhow::Result<usize> {
    let mut loc = location;

    while loc != target {
        // get list adj list for current location
        let neighbors = graph.get(&loc).ok_or(anyhow!(
            "Somehow unable to get neighbors list for {loc} in {graph:#?}"
        ))?;
        // find next location to go to
        let next: usize = if neighbors.contains(&Some(target)) {
            // if target in adj list, we should go to it
            target
        } else {
            // otherwise, we should go to parent
            *path.get(&loc).ok_or(anyhow!(
                "Somehow unable to locate parent of {loc} in {path:#?} for {graph:#?}"
            ))?
        };
        // find direction to travel
        let direction: Direction = find_direction_to_neighbor(neighbors, next).ok_or(anyhow!(
            "Failed to find direction from {location} to {next} in\n{graph:#?}\ngiven path\n{path:#?}"
        ))?;
        // finally, move robot to parent
        robot.go(direction).with_context(|| {
            format!("Failed to move robot from {location} to {next} in {graph:#?}")
        })?;
        loc = next;
    }

    Ok(loc)
}

fn find_direction_to_neighbor(neighbors: &[Option<usize>; 4], target: usize) -> Option<Direction> {
    println!("looking for {target} in {neighbors:?}");
    neighbors
        .iter()
        .enumerate()
        .fold(None, |dir: Option<Direction>, (idx, neighbor)| {
            dir.or_else(|| {
                neighbor.and_then(|v| {
                    if v == target {
                        Some(DIR_ARR[idx])
                    } else {
                        None
                    }
                })
            })
        })
}

fn render_path_bfs(
    from: usize,
    to: usize,
    graph: CellGraph,
    path: ParentMap,
) -> anyhow::Result<String> {
    todo!()
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
