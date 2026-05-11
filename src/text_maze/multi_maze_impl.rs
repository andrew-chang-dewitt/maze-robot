use std::{collections::HashSet, fmt::Display};

use crate::{
    Cell, Direction,
    text_maze::TextCell,
    traits::{MazeError, MazeErrorType, MultiMaze},
};

fn bot_char(id: usize) -> char {
    match id {
        0..=9 => (b'0' + id as u8) as char,
        10..=35 => (b'A' + (id - 10) as u8) as char,
        36..=61 => (b'a' + (id - 36) as u8) as char,
        _ => panic!("bot id {id} exceeds max of 61"),
    }
}

#[derive(Debug)]
pub struct MultiTextMaze {
    chars: Vec<char>,
    locs: Vec<usize>,
    occupied: HashSet<usize>,
    width: usize,
}

impl MultiTextMaze {
    fn get_posn_in_dir(&self, loc: usize, direction: Direction) -> Option<usize> {
        match direction {
            Direction::North => {
                if loc <= self.width {
                    None
                } else {
                    Some(loc - self.width - 1)
                }
            }
            Direction::South => {
                let pos = loc + self.width + 1;
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            Direction::East => {
                let pos = loc + 1;
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            Direction::West => {
                if loc == 0 {
                    None
                } else {
                    Some(loc - 1)
                }
            }
        }
    }
}

impl MultiMaze for MultiTextMaze {
    fn look_dir(&self, id: usize, direction: Direction) -> Result<Cell, MazeError> {
        if id >= self.locs.len() {
            return Err(MazeError::new(MazeErrorType::UnknownRobot(id)));
        }
        let loc = self.locs[id];
        let Some(pos) = self.get_posn_in_dir(loc, direction) else {
            return Ok(Cell::Wall);
        };
        // bot positions are runtime state not encoded in chars — check occupied set before chars
        // no self-exclusion needed: peek target is always a different cell than current loc
        if self.occupied.contains(&pos) {
            return Ok(Cell::Occupied);
        }
        Ok(self
            .chars
            .get(pos)
            .map(|chr| TextCell::from(chr).into())
            .unwrap_or(Cell::Wall))
    }

    fn move_dir(&mut self, id: usize, direction: Direction) -> Result<(), MazeError> {
        if id >= self.locs.len() {
            return Err(MazeError::new(MazeErrorType::UnknownRobot(id)));
        }
        let loc = self.locs[id];
        let new_loc = self
            .get_posn_in_dir(loc, direction)
            .and_then(|pos| {
                let cell: Cell = self.chars.get(pos).map(|chr| TextCell::from(chr).into())?;
                if matches!(cell, Cell::Wall) || self.occupied.contains(&pos) {
                    return None;
                }
                Some(pos)
            })
            .ok_or_else(|| MazeError::new(MazeErrorType::MoveError(direction, self.to_string())))?;

        // only vacate old cell if no other bot remains there (shared-start case)
        let old_loc = self.locs[id];
        if self.locs.iter().enumerate().all(|(i, &l)| i == id || l != old_loc) {
            self.occupied.remove(&old_loc);
        }
        self.occupied.insert(new_loc);
        self.locs[id] = new_loc;

        Ok(())
    }
}

impl TryFrom<(&str, usize)> for MultiTextMaze {
    type Error = MazeError;

    fn try_from((value, n_bots): (&str, usize)) -> Result<Self, Self::Error> {
        if n_bots == 0 {
            return Err(MazeError::new(MazeErrorType::CreationError(String::from(
                "n_bots must be >= 1",
            ))));
        }

        let (chars, starts, maybe_width) =
            value
                .chars()
                .enumerate()
                .try_fold((vec![], vec![], None), |mut acc, (idx, chr)| {
                    match chr {
                        'S' => acc.1.push(idx),
                        '\n' => match acc.2 {
                            Some(width) => {
                                if ((idx + 1) % (width + 1)) != 0 {
                                    return Err(MazeError::new(MazeErrorType::CreationError(
                                        String::from(
                                            "MultiTextMaze must have all lines with equal lengths.",
                                        ),
                                    )));
                                }
                            }
                            None => acc.2 = Some(idx),
                        },
                        _ => (),
                    };
                    acc.0.push(chr);
                    Ok(acc)
                })?;

        if starts.is_empty() {
            return Err(MazeError::new(MazeErrorType::CreationError(String::from(
                "MultiTextMaze must specify start location w/ 'S'",
            ))));
        }

        let width = match maybe_width {
            Some(w) => Ok(w),
            None if !chars.iter().all(|c| c == &'\n') => Ok(chars.len()),
            _ => Err(MazeError::new(MazeErrorType::CreationError(String::from(
                "MultiTextMaze cannot have empty lines",
            )))),
        }?;

        let locs: Vec<usize> = (0..n_bots).map(|i| starts[i % starts.len()]).collect();
        let occupied: HashSet<usize> = locs.iter().cloned().collect();

        Ok(MultiTextMaze {
            chars,
            locs,
            occupied,
            width,
        })
    }
}

impl Display for MultiTextMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // build lowest-id bot at each occupied position for rendering
        let mut loc_to_bot: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for (id, &loc) in self.locs.iter().enumerate() {
            loc_to_bot.entry(loc).or_insert(id);
        }

        let marked: Vec<String> = self
            .chars
            .iter()
            .enumerate()
            .map(|(idx, chr)| {
                if let Some(&id) = loc_to_bot.get(&idx) {
                    bot_char(id).to_string()
                } else {
                    chr.to_string()
                }
            })
            .collect();

        write!(f, "{}", marked.join(""))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const WALL_MAZE: &str = "+++\n+S+\n+++";
    const TOPL_MAZE: &str = "S+\n++";
    const TOPR_MAZE: &str = "+S\n++";
    const BOTL_MAZE: &str = "++\nS+";
    const BOTR_MAZE: &str = "++\n+S";

    fn make_maze(s: &str, n: usize) -> MultiTextMaze {
        MultiTextMaze::try_from((s, n)).expect("maze to create successfully")
    }

    // --- single-bot parity ---

    #[rstest]
    #[case::up(("  \nS ", Direction::North, "0 \nS "), 1)]
    #[case::right(("S \n  ", Direction::East, "S0\n  "), 1)]
    #[case::down((" S\n  ", Direction::South, " S\n 0"), 1)]
    #[case::left(("  \n S", Direction::West, "  \n0S"), 1)]
    fn test_move_open(
        #[case] (state, direction, exp): (&str, Direction, &str),
        #[case] _n: usize,
    ) {
        let mut maze = make_maze(state, 1);
        maze.move_dir(0, direction)
            .expect("state to update successfully");
        assert_eq!(maze.to_string(), exp);
    }

    #[rstest]
    fn test_move_invalid_single(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
        #[values(WALL_MAZE, TOPL_MAZE, TOPR_MAZE, BOTL_MAZE, BOTR_MAZE)] state: &str,
    ) {
        let mut maze = make_maze(state, 1);
        let err = maze
            .move_dir(0, direction)
            .expect_err("should return error");

        match err.get_type() {
            MazeErrorType::MoveError(_, _) => (),
            _ => panic!("expected MoveError, got {err:?}"),
        }
    }

    // --- multi-bot tests ---

    #[test]
    fn test_peek_occupied() {
        // "SS": bot 0 at idx 0, bot 1 at idx 1
        let maze = make_maze("SS", 2);
        let cell = maze.look_dir(0, Direction::East).expect("look_dir ok");
        assert_eq!(cell, Cell::Occupied);
    }

    #[test]
    fn test_move_blocked_by_bot() {
        // "SS": bot 1 is East of bot 0
        let mut maze = make_maze("SS", 2);
        let err = maze
            .move_dir(0, Direction::East)
            .expect_err("should be blocked by bot 1");

        match err.get_type() {
            MazeErrorType::MoveError(_, _) => (),
            _ => panic!("expected MoveError, got {err:?}"),
        }
    }

    #[test]
    fn test_two_bots_independent() {
        // " SS ": starts at idx 1 & 2; n=2 → locs=[1,2]
        let mut maze = make_maze(" SS ", 2);
        maze.move_dir(0, Direction::West).expect("bot 0 moves West");
        maze.move_dir(1, Direction::East).expect("bot 1 moves East");
        assert_eq!(maze.locs[0], 0);
        assert_eq!(maze.locs[1], 3);
    }

    #[test]
    fn test_unknown_robot_id_look() {
        let maze = make_maze("S", 1);
        let err = maze
            .look_dir(99, Direction::North)
            .expect_err("unknown id should error");

        match err.get_type() {
            MazeErrorType::UnknownRobot(99) => (),
            _ => panic!("expected UnknownRobot(99), got {err:?}"),
        }
    }

    #[test]
    fn test_unknown_robot_id_move() {
        let mut maze = make_maze("S", 1);
        let err = maze
            .move_dir(99, Direction::North)
            .expect_err("unknown id should error");

        match err.get_type() {
            MazeErrorType::UnknownRobot(99) => (),
            _ => panic!("expected UnknownRobot(99), got {err:?}"),
        }
    }

    #[test]
    fn test_render_two_bots() {
        // "S S": starts=[0,2], n=2 → locs=[0,2]
        let maze = make_maze("S S", 2);
        assert_eq!(maze.to_string(), "0 1");
    }

    #[test]
    fn test_round_robin_starts() {
        // "S S": starts=[0,2], n=3 → locs=[0,2,0]
        let maze = make_maze("S S", 3);
        assert_eq!(maze.locs, vec![0, 2, 0]);
    }

    #[test]
    fn test_shared_start_render() {
        // "S" n=2: both at idx 0 → render '0' (lowest id wins)
        let maze = make_maze("S", 2);
        assert_eq!(maze.to_string(), "0");
    }

    #[test]
    fn test_shared_start_collision() {
        // "S ": both bots start at idx 0
        let mut maze = make_maze("S ", 2);
        // bot 0 moves East to idx 1
        maze.move_dir(0, Direction::East).expect("bot 0 moves East");
        // bot 1 tries East → bot 0 is there
        let err = maze
            .move_dir(1, Direction::East)
            .expect_err("blocked by bot 0");
        match err.get_type() {
            MazeErrorType::MoveError(_, _) => (),
            _ => panic!("expected MoveError, got {err:?}"),
        }
        // bot 0 tries West → bot 1 still at idx 0
        let err = maze
            .move_dir(0, Direction::West)
            .expect_err("blocked by bot 1");
        match err.get_type() {
            MazeErrorType::MoveError(_, _) => (),
            _ => panic!("expected MoveError, got {err:?}"),
        }
    }
}
