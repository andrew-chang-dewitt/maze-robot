use std::fmt::Display;

pub trait Maze {
    type Key;

    fn lookup_by_direction(&self, from: Self::Key, direction: Direction) -> Option<Self::Key>;
}

#[derive(Debug)]
pub enum MazeError {
    DirectionOutOfBounds(Direction),
}

#[derive(Debug)]
pub struct TextMaze {
    maze: Vec<char>,
    pub start: usize,
    width: usize,
    height: usize,
}

impl TextMaze {
    fn new(maze: String) -> Self {
        let res = maze.chars().enumerate().fold(
            TextMaze {
                maze: vec![],
                start: 0,
                width: 0,
                height: maze.lines().count(),
            },
            |mut acc, (idx, chr)| {
                // take note of important cells in maze
                match chr {
                    // like start location
                    'S' => {
                        acc.start = idx;
                    }
                    // and width
                    '\n' if acc.width == 0 => {
                        acc.width = idx;
                    }
                    _ => (),
                };
                // then push character
                acc.maze.push(chr);
                // and move to next
                acc
            },
        );

        println!("[TextMaze::new] created new maze: {res}");

        res
    }
}

impl Maze for TextMaze {
    type Key = usize;

    fn lookup_by_direction(&self, from: Self::Key, direction: Direction) -> Option<Self::Key> {
        println!("[TextMaze::lookup_by_direction] called w/ from: {from:?}, dir: {direction:?}");
        let result = match direction {
            Direction::Up => {
                // no up if in top row
                if from <= self.width {
                    None
                } else {
                    Some(from - self.width - 1)
                }
            }
            Direction::Right => {
                let res = from + 1;

                // no right if past end of maze or in right col
                if res >= self.maze.len() || Some(&'\n') == self.maze.get(res) {
                    None
                } else {
                    Some(res)
                }
            }
            Direction::Down => {
                let res = from + self.width + 1;

                // no down if in bottom row
                if res >= self.maze.len() {
                    None
                } else {
                    Some(res)
                }
            }
            Direction::Left => {
                // no left if at beginning, otherwise decrement to go left
                let res = if from == 0 { None } else { Some(from - 1) };

                match res {
                    // ensure left doesn't wrap up a column
                    Some(v) if self.maze.get(v) == Some(&'\n') => None,
                    // otherwise left is valid
                    _ => res,
                }
            }
        };

        if let Some(neighbor) = result {
            println!(
                "[TextMaze::lookup_by_direction] {direction:#?} from {:?} (@{from}) is {neighbor}",
                self.maze.get(from).unwrap()
            );

            let cell_value = self
                .maze
                .get(result.expect("will have neighbor position at this point"))
                .expect("maze position must exist at this point");

            println!("[TextMaze::lookup_by_direction] with value of {cell_value:?}");

            if cell_value == &'+' {
                None
            } else {
                Some(neighbor)
            }
        } else {
            // println!(
            //     "[TextMaze::lookup_by_direction] no cell {direction:#?} from {:?} (@{from})",
            //     self.maze.get(from).unwrap()
            // );
            None
        }
    }
}

impl Display for TextMaze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str: String = self.maze.iter().collect();
        let as_lines: Vec<String> = as_str.lines().map(|l| format!("  {l}")).collect();
        let maze: String = as_lines.join("\n");

        write!(
            f,
            "{} x {} maze starting @ {}:\n{}",
            self.width, self.height, self.start, maze,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[cfg(test)]
mod test_text_mase {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::from_1_start(5, Some(0))]
    #[case::from_1_end(8, Some(3))]
    #[case::from_0_start(0, None)]
    #[case::from_0_end(4, None)]
    #[case::to_wall(7, None)]
    fn look_up(#[case] from: usize, #[case] exp: Option<usize>) {
        let test_maze = r#"FB+D
SACE"#;
        let maze = TextMaze::new(String::from(test_maze));

        let act = maze.lookup_by_direction(from, Direction::Up);
        let msg = format!(
            "expected UP from {:?} to be {:?}, got {:?}",
            maze.maze.get(from),
            exp.map(|v| maze.maze.get(v)).flatten(),
            act.map(|v| maze.maze.get(v)).flatten(),
        );

        assert_eq!(act, exp, "{msg}");
    }

    #[rstest]
    #[case::from_0_start(0, Some(5))]
    #[case::from_0_end(3, Some(8))]
    #[case::from_1_start(5, None)]
    #[case::from_1_end(8, None)]
    #[case::to_wall(2, None)]
    fn look_down(#[case] from: usize, #[case] exp: Option<usize>) {
        let test_maze = r#"FBCD
SA+E"#;
        let maze = TextMaze::new(String::from(test_maze));

        let act = maze.lookup_by_direction(from, Direction::Down);
        let msg = format!(
            "expected DOWN from {:?} to be {:?}, got {:?}",
            maze.maze.get(from),
            exp.map(|v| maze.maze.get(v)).flatten(),
            act.map(|v| maze.maze.get(v)).flatten(),
        );

        assert_eq!(act, exp, "{msg}");
    }

    #[rstest]
    #[case::from_0_start(0, Some(1))]
    #[case::from_0_end(1, None)]
    #[case::from_end_end(10, None)]
    #[case::to_wall(6, None)]
    fn look_right(#[case] from: usize, #[case] exp: Option<usize>) {
        let test_maze = r#"SA
CB
D+
EF"#;
        let maze = TextMaze::new(String::from(test_maze));

        let act = maze.lookup_by_direction(from, Direction::Right);
        let msg = format!(
            "expected LEFT from {:?} to be {:?}, got {:?}",
            maze.maze.get(from),
            exp.map(|v| maze.maze.get(v)).flatten(),
            act.map(|v| maze.maze.get(v)).flatten(),
        );

        assert_eq!(act, exp, "{msg}");
    }

    #[rstest]
    #[case::from_0_end(1, Some(0))]
    #[case::from_0_start(0, None)]
    #[case::from_1_start(3, None)]
    #[case::to_wall(7, None)]
    fn look_left(#[case] from: usize, #[case] exp: Option<usize>) {
        let test_maze = r#"SA
CB
+D
EF"#;
        let maze = TextMaze::new(String::from(test_maze));

        let act = maze.lookup_by_direction(from, Direction::Left);
        let msg = format!(
            "expected RIGHT from {:?} to be {:?}, got {:?}",
            maze.maze.get(from),
            exp.map(|v| maze.maze.get(v)).flatten(),
            act.map(|v| maze.maze.get(v)).flatten(),
        );

        assert_eq!(act, exp, "{msg}");
    }
}
