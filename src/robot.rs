use std::{error::Error, fmt::Display};

use crate::{
    Cell, Direction, Maze,
    maze::{MazeError, TextMaze},
};

pub struct Robot<M: Maze> {
    state: M,
}

impl<M: Maze> Robot<M> {
    pub fn new(state: M) -> Self {
        Robot { state }
    }

    pub fn peek(&self, direction: Direction) -> Cell {
        self.state.look_dir(direction)
    }

    pub fn go(&mut self, direction: Direction) -> Result<(), RobotError> {
        self.state
            .update(direction)
            .map_err(|e| RobotError::from((e, &self.state)))
    }
}

impl TryFrom<&str> for Robot<TextMaze> {
    type Error = RobotError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let maze = TextMaze::try_from(value).map_err(|e| RobotError::from((e, value)))?;

        Ok(Robot::new(maze))
    }
}

#[derive(Debug)]
pub enum RobotError {
    CreationError(String, String),
    NavigationError(Direction, String),
}

impl Display for RobotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::CreationError(msg, state) => format!("CreationError: {msg}, with\n{state}"),
            Self::NavigationError(dir, state) => {
                format!(
                    "NavigationError: unable to go {dir} from current location, see state:\n{state}"
                )
            }
        };

        write!(f, "RobotError::{out}")
    }
}

impl Error for RobotError {}

impl<S: Display> From<(MazeError, S)> for RobotError {
    fn from((maze_err, state): (MazeError, S)) -> Self {
        match maze_err {
            MazeError::CreationError(msg) => RobotError::CreationError(msg, state.to_string()),
            MazeError::UpdateError(dir) => RobotError::NavigationError(dir, state.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    pub const WALL_MAZE: &str = r#"+++
+S+
+++"#;
    pub const OPEN_MAZE: &str = r#"   
 S 
   "#;
    pub const FNSH_MAZE: &str = r#"SF"#;
    pub const TOPL_MAZE: &str = r#"S 
  "#;
    pub const TOPR_MAZE: &str = r#" S
  "#;
    pub const BOTL_MAZE: &str = r#"  
S "#;
    pub const BOTR_MAZE: &str = r#"  
 S"#;

    fn make_robot(maze: &str) -> Robot<TextMaze> {
        Robot::try_from(maze).expect("Robot creates successfully")
    }

    #[rstest]
    fn test_peek_wall(
        #[values(Direction::Up, Direction::Right, Direction::Down, Direction::Left)]
        direction: Direction,
    ) {
        let rob = make_robot(WALL_MAZE);

        match rob.peek(direction) {
            Cell::Wall => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Wall"),
        }
    }

    #[rstest]
    fn test_peek_open(
        #[values(Direction::Up, Direction::Right, Direction::Down, Direction::Left)]
        direction: Direction,
    ) {
        let rob = make_robot(OPEN_MAZE);

        match rob.peek(direction) {
            Cell::Open => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Open"),
        }
    }

    #[rstest]
    #[case((TOPL_MAZE,Direction::Up),Cell::Wall)]
    #[case((TOPL_MAZE,Direction::Left),Cell::Wall)]
    #[case((TOPL_MAZE,Direction::Down),Cell::Open)]
    #[case((TOPL_MAZE,Direction::Right),Cell::Open)]
    fn test_peek_topl_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((TOPR_MAZE,Direction::Up),Cell::Wall)]
    #[case((TOPR_MAZE,Direction::Right),Cell::Wall)]
    #[case((TOPR_MAZE,Direction::Down),Cell::Open)]
    #[case((TOPR_MAZE,Direction::Left),Cell::Open)]
    fn test_peek_topr_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTL_MAZE,Direction::Down),Cell::Wall)]
    #[case((BOTL_MAZE,Direction::Left),Cell::Wall)]
    #[case((BOTL_MAZE,Direction::Up),Cell::Open)]
    #[case((BOTL_MAZE,Direction::Right),Cell::Open)]
    fn test_peek_botl_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTR_MAZE,Direction::Down),Cell::Wall)]
    #[case((BOTR_MAZE,Direction::Right),Cell::Wall)]
    #[case((BOTR_MAZE,Direction::Up),Cell::Open)]
    #[case((BOTR_MAZE,Direction::Left),Cell::Open)]
    fn test_peek_botr_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_peek_finish() {
        let rob = make_robot(FNSH_MAZE);
        let act = rob.peek(Direction::Right);

        assert_eq!(act, Cell::Finish)
    }

    #[rstest]
    fn test_go_open(
        #[values(Direction::Up, Direction::Right, Direction::Down, Direction::Left)]
        direction: Direction,
    ) -> Result<(), RobotError> {
        let mut rob = make_robot(OPEN_MAZE);

        rob.go(direction)
    }
}
