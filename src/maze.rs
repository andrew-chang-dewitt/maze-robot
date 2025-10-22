use std::{error::Error, fmt::Display};

use crate::{Cell, Direction};

pub trait Maze: Display {
    fn look_dir(&self, direction: Direction) -> Cell;
    fn update(&mut self, direction: Direction) -> Result<(), MazeError>;
}

/// maze encoded as str where:
/// - 'S' is starting location
/// - 'F' is ending location
/// - '+' & out of bounds are considered walls
/// - all others are considered open
pub struct TextMaze {
    chars: Vec<char>,
    loc: usize,
    width: usize,
}

impl TextMaze {
    fn get_posn_in_dir(&self, direction: Direction) -> Option<usize> {
        match direction {
            Direction::Up => {
                // no up if in top row
                if self.loc <= self.width {
                    None
                } else {
                    Some(self.loc - self.width - 1)
                }
            }
            Direction::Down => {
                // go down one row by adding width & accounting for newline char
                let pos = self.loc + self.width + 1;
                // no down if past end of chars vec
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            Direction::Right => {
                // go right one col by incrementing pos
                let pos = self.loc + 1;
                // no right if past end of chars vec
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            Direction::Left => {
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
        // println!("looking {direction:?} from {}", self.loc);
        self.get_posn_in_dir(direction)
            .and_then(|pos| self.chars.get(pos))
            .map(|chr| Cell::from(chr))
            .unwrap_or(Cell::Wall)
    }

    fn update(&mut self, direction: Direction) -> Result<(), MazeError> {
        self.loc = self
            .get_posn_in_dir(direction)
            .and_then(|pos| {
                let cell = self.chars.get(pos).map(|chr| Cell::from(chr))?;
                match cell {
                    Cell::Wall => None,
                    _ => Some(pos),
                }
            })
            .ok_or(MazeError::UpdateError(direction))?;

        Ok(())
    }
}

impl TryFrom<&str> for TextMaze {
    type Error = MazeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // println!("creating maze from {value}");
        let (chars, maybe_loc, maybe_width) =
            value
                .chars()
                .enumerate()
                .try_fold((vec![], None, None), |mut acc, (idx, chr)| {
                    // println!("checking char {chr:?} @ {idx}");
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

/// Map text characters to Cell values, where
/// - 'S' is starting location
/// - 'F' is ending location
/// - '+' is a wall
/// - all others are considered open
impl From<&char> for Cell {
    fn from(value: &char) -> Self {
        match value {
            'F' => Self::Finish,
            '+' | '\n' => Self::Wall,
            _ => Self::Open,
        }
    }
}

#[derive(Debug)]
pub enum MazeError {
    CreationError(String),
    UpdateError(Direction),
}

impl Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::CreationError(msg) => format!("CreationError: {msg}"),
            Self::UpdateError(direction) => {
                format!("UpdateError: unable to go {direction} from current location")
            }
        };

        write!(f, "MazeError::{out}")
    }
}

impl Error for MazeError {}

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
    #[case::up(("  \nS ", Direction::Up), "X \nS ")]
    #[case::right(("S \n  ", Direction::Right), "SX\n  ")]
    #[case::down((" S\n  ", Direction::Down), " S\n X")]
    #[case::left(("  \n S", Direction::Left), "  \nXS")]
    fn test_move_open(#[case] (state, direction): (&str, Direction), #[case] exp: String) {
        let mut maze = TextMaze::try_from(state).expect("maze to create successfully");
        maze.update(direction).expect("state to update succesfully");
        let act = maze.to_string();

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_move_invalid(
        #[values(Direction::Up, Direction::Right, Direction::Down, Direction::Left)]
        direction: Direction,
        #[values(WALL_MAZE, TOPL_MAZE, TOPR_MAZE, BOTL_MAZE, BOTR_MAZE)] state: &str,
    ) {
        let mut maze = TextMaze::try_from(state).expect("maze to create successfully");

        match maze.update(direction) {
            Ok(_) => panic!(
                "should have returned error when trying to move {direction:?} in maze:\n{state}\ninstead, got new state:\n{}",
                maze.to_string()
            ),

            Err(MazeError::UpdateError(_)) => (),
            Err(e) => panic!("expected UpdateError, got {e:?}"),
        }
    }
}
