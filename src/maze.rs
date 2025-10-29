use std::{error::Error, fmt::Display};

use crate::CardinalDirection;

#[derive(Debug, Eq, PartialEq)]
pub enum Cell {
    Finish,
    Open,
    Wall,
}

pub trait Maze<Loc> {
    fn look_dir(&self, from: &Loc, direction: CardinalDirection) -> (Cell, Option<Loc>);
    fn update(&self, from: &Loc, direction: CardinalDirection) -> Result<Loc, MazeError>;
    fn render(&self, loc: &Loc) -> String;
}

/// maze encoded as str where:
/// - 'S' is starting location
/// - 'F' is ending location
/// - '+' & out of bounds are considered walls
/// - all others are considered open
pub struct TextMaze {
    chars: Vec<char>,
    width: usize,
    pub start: usize,
}

impl TextMaze {
    fn get_posn_in_dir(&self, from: &usize, direction: CardinalDirection) -> Option<usize> {
        match direction {
            CardinalDirection::North => {
                // no up if in top row
                if from <= &self.width {
                    None
                } else {
                    Some(from - &self.width - 1)
                }
            }
            CardinalDirection::South => {
                // go down one row by adding width & accounting for newline char
                let pos = from + &self.width + 1;
                // no down if past end of chars vec
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            CardinalDirection::East => {
                // go right one col by incrementing pos
                let pos = from + 1;
                // no right if past end of chars vec
                if pos >= self.chars.len() {
                    None
                } else {
                    Some(pos)
                }
            }
            CardinalDirection::West => {
                // no left if loc already at start
                if from == &0 {
                    None
                } else {
                    // go left one col by decrementing pos
                    Some(from - 1)
                }
            }
        }
    }
}

impl Maze<usize> for TextMaze {
    fn look_dir(&self, from: &usize, direction: CardinalDirection) -> (Cell, Option<usize>) {
        // println!("looking {direction:?}: {self}");
        self.get_posn_in_dir(from, direction)
            .and_then(|pos| self.chars.get(pos).map(|chr| (Cell::from(chr), Some(pos))))
            .unwrap_or((Cell::Wall, None))
    }

    fn update(&self, from: &usize, direction: CardinalDirection) -> Result<usize, MazeError> {
        println!("attempting to go {direction}");
        let to = self
            .get_posn_in_dir(from, direction)
            .and_then(|pos| {
                let cell = self.chars.get(pos).map(|chr| Cell::from(chr))?;
                match cell {
                    Cell::Wall => None,
                    _ => Some(pos),
                }
            })
            .ok_or(MazeError::UpdateError(direction))?;

        println!("newstate:\n{}", self.render(&to));

        Ok(to)
    }

    fn render(&self, loc: &usize) -> String {
        let marked: Vec<String> = self
            .chars
            .iter()
            .enumerate()
            .map(|(idx, chr)| {
                if &idx == loc {
                    String::from("X")
                } else {
                    chr.to_string()
                }
            })
            .collect();

        format!("{}", marked.join(""))
    }
}

impl TryFrom<&str> for TextMaze {
    type Error = MazeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // println!("creating maze from {value}");
        let (chars, maybe_start, maybe_width) =
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

        let start = maybe_start.ok_or(MazeError::CreationError(String::from(
            "TextMaze must specify start location w/ 'S'",
        )))?;
        let width = match maybe_width {
            Some(w) => Ok(w),
            None if !chars.iter().all(|c| c == &'\n') => Ok(chars.len()),
            _ => Err(MazeError::CreationError(String::from(
                "TextMaze cannot have empty lines",
            ))),
        }?;

        Ok(TextMaze {
            chars,
            start,
            width,
        })
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
    UpdateError(CardinalDirection),
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
    #[case::up(("  \nS ", CardinalDirection::North), "X \nS ")]
    #[case::right(("S \n  ", CardinalDirection::East), "SX\n  ")]
    #[case::down((" S\n  ", CardinalDirection::South), " S\n X")]
    #[case::left(("  \n S", CardinalDirection::West), "  \nXS")]
    fn test_move_open(#[case] (state, direction): (&str, CardinalDirection), #[case] exp: String) {
        let maze = TextMaze::try_from(state).expect("maze to create successfully");
        let new_loc = maze
            .update(&maze.start, direction)
            .expect("state to update succesfully");
        let act = maze.render(&new_loc);

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_move_invalid(
        #[values(
            CardinalDirection::North,
            CardinalDirection::East,
            CardinalDirection::South,
            CardinalDirection::West
        )]
        direction: CardinalDirection,
        #[values(WALL_MAZE, TOPL_MAZE, TOPR_MAZE, BOTL_MAZE, BOTR_MAZE)] state: &str,
    ) {
        let maze = TextMaze::try_from(state).expect("maze to create successfully");

        match maze.update(&maze.start, direction) {
            Ok(new_loc) => panic!(
                "should have returned error when trying to move {direction:?} in maze:\n{state}\ninstead, got new state:\n{}",
                maze.render(&new_loc)
            ),

            Err(MazeError::UpdateError(_)) => (),
            Err(e) => panic!("expected UpdateError, got {e:?}"),
        }
    }
}
