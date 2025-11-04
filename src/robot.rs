use std::{error::Error, fmt::Display};

use crate::{
    Cell, Direction, Maze,
    maze::{MazeError, TextMaze},
};

pub struct Robot<M: Maze> {
    state: M,
}

impl<M: Maze> Robot<M> {
    fn new(state: M) -> Self {
        Robot { state }
    }

    fn peek(&self, direction: Direction) -> Cell {
        self.state.look_dir(direction)
    }

    fn go(&mut self, direction: Direction) -> Result<(), RobotError> {
        self.state.update(direction).map_err(|e| e.into())
    }
}

impl TryFrom<&str> for Robot<TextMaze> {
    type Error = RobotError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let maze = TextMaze::try_from(value).map_err(|e| RobotError::from(e))?;

        Ok(Robot::new(maze))
    }
}

#[derive(Debug)]
pub enum RobotError {
    CreationError(String),
    NavigationError(Direction),
}

impl Display for RobotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::CreationError(msg) => format!("CreationError: {msg}"),
            Self::NavigationError(dir) => {
                format!("UpdateError: unable to go {dir} from current location")
            }
        };

        write!(f, "MazeError:{out}")
    }
}

impl Error for RobotError {}

impl From<MazeError> for RobotError {
    fn from(value: MazeError) -> RobotError {
        match value {
            MazeError::CreationError(msg) => Self::CreationError(msg),
            MazeError::MoveError(dir) => Self::NavigationError(dir),
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
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
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
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) {
        let rob = make_robot(OPEN_MAZE);

        match rob.peek(direction) {
            Cell::Open => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Open"),
        }
    }

    #[rstest]
    #[case((TOPL_MAZE,Direction::North),Cell::Wall)]
    #[case((TOPL_MAZE,Direction::West),Cell::Wall)]
    #[case((TOPL_MAZE,Direction::South),Cell::Open)]
    #[case((TOPL_MAZE,Direction::East),Cell::Open)]
    fn test_peek_topl_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((TOPR_MAZE,Direction::North),Cell::Wall)]
    #[case((TOPR_MAZE,Direction::East),Cell::Wall)]
    #[case((TOPR_MAZE,Direction::South),Cell::Open)]
    #[case((TOPR_MAZE,Direction::West),Cell::Open)]
    fn test_peek_topr_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTL_MAZE,Direction::South),Cell::Wall)]
    #[case((BOTL_MAZE,Direction::West),Cell::Wall)]
    #[case((BOTL_MAZE,Direction::North),Cell::Open)]
    #[case((BOTL_MAZE,Direction::East),Cell::Open)]
    fn test_peek_botl_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTR_MAZE,Direction::South),Cell::Wall)]
    #[case((BOTR_MAZE,Direction::East),Cell::Wall)]
    #[case((BOTR_MAZE,Direction::North),Cell::Open)]
    #[case((BOTR_MAZE,Direction::West),Cell::Open)]
    fn test_peek_botr_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_peek_finish() {
        let rob = make_robot(FNSH_MAZE);
        let act = rob.peek(Direction::East);

        assert_eq!(act, Cell::Finish)
    }

    #[rstest]
    fn test_go_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) -> Result<(), RobotError> {
        let mut rob = make_robot(OPEN_MAZE);

        rob.go(direction)
    }
}
