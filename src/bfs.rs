type ParentMap = HashMap<usize, usize>;

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
        println!("moved robot to cell {location}");
        // mark as visitied
        visited.insert(cell);
        println!("marked {location} as visited {visited:?}");
        // mark finish as not yet found
        let mut finish: Option<usize> = None;

        // get info to prefill neighbors array with parent cell
        let parent_direction = match location {
            0 => None,
            _ => path.get(&location),
        }
        .and_then(|pdx| {
            graph.get(&pdx).map(|n| {
                println!("found neighbors {n:?} for parent {pdx} of {location}");
                (pdx, n)
            })
        })
        .and_then(|(pdx, n)| {
            find_direction_to_neighbor(n, location).map(|dir| {
                println!("{location} is {dir:?} from {pdx}");
                (pdx, dir)
            })
        })
        .map(|(pdx, dir_p_to_l)| {
            println!("{pdx} is {:?} from {location}", dir_p_to_l.reverse());
            (pdx, dir_p_to_l.reverse())
        });

        let init_neighbors = match parent_direction {
            Some((&pdx, Direction::Up)) => [Some(pdx), None, None, None],
            Some((&pdx, Direction::Right)) => [None, Some(pdx), None, None],
            Some((&pdx, Direction::Down)) => [None, None, Some(pdx), None],
            Some((&pdx, Direction::Left)) => [None, None, None, Some(pdx)],
            None => [None, None, None, None],
        };

        println!("parsing neighbors, starting w/ list as {init_neighbors:?}");

        // add cell & neighbors list to graph
        let neighbors: [Option<usize>; 4] = DIR_ARR.iter().enumerate().fold(
            init_neighbors,
            |mut acc, (idx, direction)| {
                if let None = acc[idx] {
                    match robot.peek(*direction) {
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
                                println!("assigned {location} as parent of {neighbor} in {path:?}");
                                // push neighbor to queue
                                to_visit.push_back(neighbor);
                                println!(
                                    "found neighbor {neighbor} to the {direction:?} of {location}"
                                );
                            }
                            // finally, return updated neighbors list
                            acc
                        }
                        Cell::Finish => {
                            // assign neighbor cell an index
                            let neighbor = nxt_idx;
                            // increment cell index any time the next is used
                            nxt_idx += 1;
                            println!("found solution @ {neighbor}!");
                            // mark finish as found!
                            finish = Some(neighbor);
                            // update neighbors list to include finish
                            acc[idx] = Some(neighbor);
                            // add parent-child relationship for neighbor & location to path
                            path.insert(neighbor, location);
                            acc
                        }
                        Cell::Wall => acc,
                    }
                } else {
                    acc
                }
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
        println!("Checking if {target} is in {neighbors:?}");
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
