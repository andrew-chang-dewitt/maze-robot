use std::fmt::Display;

use maze_robot::{Cell, Direction, Maze, MazeError};

use crate::text_maze::TextCell;

/// A maze encoded by a string, where:
/// - 'S' is starting location
/// - 'F' is ending location
/// - '+' & out of bounds are considered walls
/// - all others are considered open
///
/// Tracks robot location as private state used by the two `Maze` trait methods.
#[derive(Debug)]
pub struct TextMaze {
    chars: Vec<char>,
    loc: usize,
    width: usize,
}

impl TextMaze {
    fn get_posn_in_dir(&self, direction: Direction) -> Option<usize> {
        match direction {
            Direction::North => {
                // no up if in top row
                if self.loc <= self.width {
                    None
                } else {
                    Some(self.loc - self.width - 1)
                }
            }
            Direction::South => {
                // go down one row by adding width & accounting for newline char
                let pos = self.loc + self.width + 1;
                // no down if past end of chars vec
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            Direction::East => {
                // go right one col by incrementing pos
                let pos = self.loc + 1;
                // no right if past end of chars vec
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            Direction::West => {
                // no left if loc already at start
                if self.loc == 0 {
                    None
                } else {
                    // go left one col by decrementing pos
                    Some(self.loc - 1)
                }
            }
        }
    }
}

impl Maze for TextMaze {
    fn look_dir(&self, direction: Direction) -> Cell {
        self.get_posn_in_dir(direction)
            .and_then(|pos| self.chars.get(pos))
            .map(|chr| TextCell::from(chr).into())
            .unwrap_or(Cell::Wall)
    }

    fn move_dir(&mut self, direction: Direction) -> Result<(), MazeError> {
        self.loc = self
            .get_posn_in_dir(direction)
            .and_then(|pos| {
                let cell = self.chars.get(pos).map(|chr| TextCell::from(chr).into())?;
                match cell {
                    Cell::Wall => None,
                    _ => Some(pos),
                }
            })
            .ok_or(MazeError::MoveError(direction, self.to_string()))?;

        Ok(())
    }
}

impl TryFrom<&str> for TextMaze {
    type Error = MazeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (chars, maybe_loc, maybe_width) =
            value
                .chars()
                .enumerate()
                .try_fold((vec![], None, None), |mut acc, (idx, chr)| {
                    match chr {
                        'S' => acc.1 = Some(idx),
                        '\n' => match acc.2 {
                            Some(width) => {
                                if ((idx + 1) % (width + 1)) != 0 {
                                    return Err(MazeError::CreationError(String::from(
                                        "TextMaze must have all lines with equal lengths.",
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

        let loc = maybe_loc.ok_or(MazeError::CreationError(String::from(
            "TextMaze must specify start location w/ 'S'",
        )))?;
        let width = match maybe_width {
            Some(w) => Ok(w),
            None if !chars.iter().all(|c| c == &'\n') => Ok(chars.len()),
            _ => Err(MazeError::CreationError(String::from(
                "TextMaze cannot have empty lines",
            ))),
        }?;

        Ok(TextMaze { chars, loc, width })
    }
}

impl Display for TextMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let marked: Vec<String> = self
            .chars
            .iter()
            .enumerate()
            .map(|(idx, chr)| {
                if idx == self.loc {
                    String::from("X")
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

    pub const WALL_MAZE: &str = r#"+++
+S+
+++"#;
    pub const TOPL_MAZE: &str = r#"S+
++"#;
    pub const TOPR_MAZE: &str = r#"+S
++"#;
    pub const BOTL_MAZE: &str = r#"++
S+"#;
    pub const BOTR_MAZE: &str = r#"++
+S"#;

    #[rstest]
    #[case::up(("  \nS ", Direction::North), "X \nS ")]
    #[case::right(("S \n  ", Direction::East), "SX\n  ")]
    #[case::down((" S\n  ", Direction::South), " S\n X")]
    #[case::left(("  \n S", Direction::West), "  \nXS")]
    fn test_move_open(#[case] (state, direction): (&str, Direction), #[case] exp: String) {
        let mut maze = TextMaze::try_from(state).expect("maze to create successfully");
        maze.move_dir(direction)
            .expect("state to update succesfully");
        let act = maze.to_string();

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_move_invalid(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
        #[values(WALL_MAZE, TOPL_MAZE, TOPR_MAZE, BOTL_MAZE, BOTR_MAZE)] state: &str,
    ) {
        let mut maze = TextMaze::try_from(state).expect("maze to create successfully");

        match maze.move_dir(direction) {
            Ok(_) => panic!(
                "should have returned error when trying to move {direction:?} in maze:\n{state}\ninstead, got new state:\n{}",
                maze.to_string()
            ),

            Err(MazeError::MoveError(_, _)) => (),
            Err(e) => panic!("expected UpdateError, got {e:?}"),
        }
    }
}
