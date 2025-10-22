use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

pub trait Maze {
    fn look_direction(&self, direction: Direction) -> Result<Cell, MazeError>;
    fn move_direction(&mut self, direction: Direction) -> Result<(), MazeError>;
}

trait PrivateMaze {
    type Key: Copy + Eq;
    type Value: Into<Cell>;

    fn get_location(&self) -> Self::Key;
    fn get_loc_in_dir_from(
        &self,
        from: Self::Key,
        direction: Direction,
    ) -> Result<Self::Key, PrivateMazeError<Self::Key>>;
    fn get_value(&self, location: Self::Key) -> Result<Self::Value, PrivateMazeError<Self::Key>>;
    fn set_location(&mut self, location: Self::Key) -> Result<(), PrivateMazeError<Self::Key>>;
}

impl<P: PrivateMaze> Maze for P {
    fn look_direction(&self, direction: Direction) -> Result<Cell, MazeError> {
        self.get_loc_in_dir_from(self.get_location(), direction)
            .and_then(|l| self.get_value(l))
            .map(|v| v.into())
            .map_err(|e| MazeError::from(e))
    }

    fn move_direction(&mut self, direction: Direction) -> Result<(), MazeError> {
        let target = self
            .get_loc_in_dir_from(self.get_location(), direction)
            .map_err(|e| MazeError::from(e))?;

        self.set_location(target).map_err(|e| MazeError::from(e))
    }
}

#[derive(Debug)]
pub enum Cell {
    Open,
    Finish,
    Wall,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MazeError {
    DirectionOutOfBounds(Direction),
    NavigationError(Direction),
    UnknownError(String),
}

#[derive(Debug, Eq, PartialEq)]
enum PrivateMazeError<L> {
    DirectionOutOfBounds(Direction),
    LocationInvalid(L),
}

impl<E> From<PrivateMazeError<E>> for MazeError {
    fn from(value: PrivateMazeError<E>) -> Self {
        match value {
            PrivateMazeError::LocationInvalid(_) => Self::UnknownError(String::from(
                "Error occurred while working with current location.",
            )),
            PrivateMazeError::DirectionOutOfBounds(d) => Self::DirectionOutOfBounds(d),
        }
    }
}

#[derive(Debug)]
pub struct TextMaze {
    chars: Vec<char>,
    loc: usize,
    width: usize,
    height: usize,
}

impl TextMaze {
    pub fn new(maze: String) -> Self {
        let res = maze.chars().enumerate().fold(
            TextMaze {
                chars: vec![],
                loc: 0,
                width: 0,
                height: maze.lines().count(),
            },
            |mut acc, (idx, chr)| {
                // take note of important cells in maze
                match chr {
                    // like start location
                    'S' => {
                        acc.loc = idx;
                    }
                    // and width
                    '\n' if acc.width == 0 => {
                        acc.width = idx;
                    }
                    _ => (),
                };
                // then push character
                acc.chars.push(chr);
                // and move to next
                acc
            },
        );

        // println!("[TextMaze::new] created new maze: {res}");

        res
    }
}

impl Into<Cell> for char {
    fn into(self) -> Cell {
        match self {
            '+' => Cell::Wall,
            'F' => Cell::Finish,
            _ => Cell::Open,
        }
    }
}

impl PrivateMaze for TextMaze {
    type Key = usize;
    type Value = char;

    fn get_loc_in_dir_from(
        &self,
        from: Self::Key,
        direction: Direction,
    ) -> Result<Self::Key, PrivateMazeError<Self::Key>> {
        // println!("[TextMaze::lookup_by_direction] called w/ from: {from:?}, dir: {direction:?}");
        match direction {
            Direction::Up => {
                // no up if in top row
                if from <= self.width {
                    Err(PrivateMazeError::DirectionOutOfBounds(direction))
                } else {
                    Ok(from - self.width - 1)
                }
            }
            Direction::Right => {
                let res = from + 1;

                // no right if past end of maze or in right col
                if res >= self.chars.len() || Some(&'\n') == self.chars.get(res) {
                    Err(PrivateMazeError::DirectionOutOfBounds(direction))
                } else {
                    Ok(res)
                }
            }
            Direction::Down => {
                let res = from + self.width + 1;

                // no down if in bottom row
                if res >= self.chars.len() {
                    Err(PrivateMazeError::DirectionOutOfBounds(direction))
                } else {
                    Ok(res)
                }
            }
            Direction::Left => {
                // no left if at beginning, otherwise decrement to go left
                if from == 0 {
                    Err(PrivateMazeError::DirectionOutOfBounds(direction))
                } else {
                    let res = from - 1;

                    // no left if in left col
                    if Some(&'\n') == self.chars.get(res) {
                        Err(PrivateMazeError::DirectionOutOfBounds(direction))
                    } else {
                        Ok(res)
                    }
                }
            }
        }
    }

    fn get_value(&self, location: Self::Key) -> Result<Self::Value, PrivateMazeError<Self::Key>> {
        todo!()
    }

    fn get_location(&self) -> Self::Key {
        todo!()
    }

    fn set_location(&mut self, location: Self::Key) -> Result<(), PrivateMazeError<Self::Key>> {
        todo!()
    }
}

impl Display for TextMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // mark current location on map
        let marked: String = self
            .chars
            .iter()
            .enumerate()
            .map(|(i, c)| if i == self.loc { &'x' } else { c })
            .collect();
        // then pad each line w/ two spaces for prettier printing
        let as_lines: Vec<String> = marked.lines().map(|l| format!("  {l}")).collect();
        // and join those lines into one string
        let maze: String = as_lines.join("\n");

        write!(f, "{} x {} maze:\n{}", self.width, self.height, maze,)
    }
}

#[cfg(test)]
mod test_text_mase {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::from_1_start(5, Ok(0))]
    #[case::from_1_end(8, Ok(3))]
    #[case::from_0_start(0, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Up)))]
    #[case::from_0_end(4, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Up)))]
    #[case::to_wall(7, Ok(2))]
    fn look_up(#[case] from: usize, #[case] exp: Result<usize, PrivateMazeError<usize>>) {
        let test_maze = r#"FB+D
SACE"#;
        let maze = TextMaze::new(String::from(test_maze));

        let act = maze.get_loc_in_dir_from(from, Direction::Up);

        assert_eq!(act, exp);
    }

    #[rstest]
    #[case::from_0_start(0, Ok(5))]
    #[case::from_0_end(3, Ok(8))]
    #[case::from_1_start(5, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Down)))]
    #[case::from_1_end(8, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Down)))]
    #[case::to_wall(2, Ok(7))]
    fn look_down(#[case] from: usize, #[case] exp: Result<usize, PrivateMazeError<usize>>) {
        let test_maze = r#"FBCD
SA+E"#;
        let maze = TextMaze::new(String::from(test_maze));

        let act = maze.get_loc_in_dir_from(from, Direction::Down);

        assert_eq!(act, exp);
    }

    #[rstest]
    #[case::from_0_start(0, Ok(1))]
    #[case::from_0_end(1, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Right)))]
    #[case::from_end_end(10, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Right)))]
    #[case::to_wall(6, Ok(7))]
    fn look_right(#[case] from: usize, #[case] exp: Result<usize, PrivateMazeError<usize>>) {
        let test_maze = r#"SA
CB
D+
EF"#;
        let maze = TextMaze::new(String::from(test_maze));
        let act = maze.get_loc_in_dir_from(from, Direction::Right);

        assert_eq!(act, exp);
    }

    #[rstest]
    #[case::from_0_end(1, Ok(0))]
    #[case::from_0_start(0, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Left)))]
    #[case::from_1_start(3, Err(PrivateMazeError::DirectionOutOfBounds(Direction::Left)))]
    #[case::to_wall(7, Ok(6))]
    fn look_left(#[case] from: usize, #[case] exp: Result<usize, PrivateMazeError<usize>>) {
        let test_maze = r#"SA
CB
+D
EF"#;
        let maze = TextMaze::new(String::from(test_maze));
        let act = maze.get_loc_in_dir_from(from, Direction::Left);

        assert_eq!(act, exp);
    }
}
