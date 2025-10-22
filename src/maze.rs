use std::{error::Error, fmt::Display};

use crate::{Cell, Direction};

pub trait Maze {
    fn look_dir(&self, direction: Direction) -> Cell;
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

pub struct TextMaze {
    chars: Vec<char>,
    loc: usize,
    width: usize,
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
                                if idx % width != 0 {
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
        let width = maybe_width.ok_or(MazeError::CreationError(String::from(
            "TextMaze cannot have empty lines",
        )))?;

        Ok(TextMaze { chars, loc, width })
    }
}

impl Maze for TextMaze {
    fn look_dir(&self, direction: Direction) -> Cell {
        match direction {
            Direction::Up => todo!(),
            Direction::Down => todo!(),
            Direction::Right => todo!(),
            Direction::Left => todo!(),
        }
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
