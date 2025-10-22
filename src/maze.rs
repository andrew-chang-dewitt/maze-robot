use std::{error::Error, fmt::Display};

use crate::{Cell, Direction};

pub trait Maze {
    fn look_dir(&self, direction: Direction) -> Cell;
}

pub struct TextMaze {
    chars: Vec<char>,
    loc: usize,
    width: usize,
}

/// maze encoded as str where:
/// - 'S' is starting location
/// - 'F' is ending location
/// - '+' & out of bounds are considered walls
/// - all others are considered open
impl Maze for TextMaze {
    fn look_dir(&self, direction: Direction) -> Cell {
        // println!("looking {direction:?} from {}", self.loc);

        match direction {
            Direction::Up => {
                // no up if in top row
                if self.loc <= self.width {
                    Cell::Wall
                } else {
                    Cell::from(
                        self.chars
                            .get(self.loc - self.width - 1)
                            .expect("Looking Up should find Cell"),
                    )
                }
            }
            Direction::Down => {
                // go down one row by adding width & accounting for newline char
                let pos = self.loc + self.width + 1;
                // no down if past end of chars vec
                if pos >= self.chars.len() {
                    Cell::Wall
                } else {
                    Cell::from(self.chars.get(pos).expect("Looking Down should find Cell"))
                }
            }
            Direction::Right => {
                // go right one col by incrementing pos
                let pos = self.loc + 1;
                // no right if past end of chars vec
                if pos >= self.chars.len() {
                    Cell::Wall
                } else {
                    Cell::from(self.chars.get(pos).expect("Looking Right should find Cell"))
                }
            }
            Direction::Left => {
                // no left if loc already at start
                if self.loc == 0 {
                    Cell::Wall
                } else {
                    // go left one col by decrementing pos
                    let pos = self.loc - 1;
                    Cell::from(self.chars.get(pos).expect("Looking Left should find Cell"))
                }
            }
        }
    }
}

impl TryFrom<&str> for TextMaze {
    type Error = MazeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        println!("creating maze from {value}");
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
}

impl Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::CreationError(msg) => format!("CreationError: {msg}"),
        };

        write!(f, "MazeError:{out}")
    }
}

impl Error for MazeError {}
