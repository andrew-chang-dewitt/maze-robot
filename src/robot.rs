use std::{error::Error, fmt::Display};

use crate::{
    CARD_DIR_ARR, CardinalDirection, Cell, Maze,
    maze::{MazeError, TextMaze},
};

pub struct Robot<M: Maze> {
    state: M,
    facing: CardinalDirection,
}

impl<M: Maze> Robot<M> {
    pub fn new(state: M, facing: CardinalDirection) -> Self {
        Robot { state, facing }
    }

    pub fn peek(&self, direction: CardinalDirection) -> Cell {
        self.state.look_dir(direction)
    }

    pub fn peek_all(&self) -> [Cell; 4] {
        [0, 1, 2, 3].map(|idx| self.peek(CARD_DIR_ARR[idx]))
    }

    pub fn go(&mut self, direction: CardinalDirection) -> Result<(), RobotError> {
        self.state
            .update(direction)
            .map_err(|e| RobotError::from((e, &self.state)))?;
        self.facing = direction;
        println!("Robot moved, now facing {}", self.facing);

        Ok(())
    }

    pub fn get_facing(&self) -> CardinalDirection {
        self.facing
    }
}

impl TryFrom<(&str, CardinalDirection)> for Robot<TextMaze> {
    type Error = RobotError;

    fn try_from((text, facing): (&str, CardinalDirection)) -> Result<Self, Self::Error> {
        let maze = TextMaze::try_from(text).map_err(|e| RobotError::from((e, text)))?;

        Ok(Robot::new(maze, facing))
    }
}

#[derive(Debug)]
pub enum RobotError {
    CreationError(String, String),
    NavigationError(CardinalDirection, String),
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
        Robot::try_from((maze, CardinalDirection::North)).expect("Robot creates successfully")
    }

    #[rstest]
    fn test_peek_wall(
        #[values(
            CardinalDirection::North,
            CardinalDirection::East,
            CardinalDirection::South,
            CardinalDirection::West
        )]
        direction: CardinalDirection,
    ) {
        let rob = make_robot(WALL_MAZE);

        match rob.peek(direction) {
            Cell::Wall => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Wall"),
        }
    }

    #[rstest]
    fn test_peek_open(
        #[values(
            CardinalDirection::North,
            CardinalDirection::East,
            CardinalDirection::South,
            CardinalDirection::West
        )]
        direction: CardinalDirection,
    ) {
        let rob = make_robot(OPEN_MAZE);

        match rob.peek(direction) {
            Cell::Open => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Open"),
        }
    }

    #[rstest]
    #[case((TOPL_MAZE,CardinalDirection::North),Cell::Wall)]
    #[case((TOPL_MAZE,CardinalDirection::West),Cell::Wall)]
    #[case((TOPL_MAZE,CardinalDirection::South),Cell::Open)]
    #[case((TOPL_MAZE,CardinalDirection::East),Cell::Open)]
    fn test_peek_topl_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((TOPR_MAZE,CardinalDirection::North),Cell::Wall)]
    #[case((TOPR_MAZE,CardinalDirection::East),Cell::Wall)]
    #[case((TOPR_MAZE,CardinalDirection::South),Cell::Open)]
    #[case((TOPR_MAZE,CardinalDirection::West),Cell::Open)]
    fn test_peek_topr_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTL_MAZE,CardinalDirection::South),Cell::Wall)]
    #[case((BOTL_MAZE,CardinalDirection::West),Cell::Wall)]
    #[case((BOTL_MAZE,CardinalDirection::North),Cell::Open)]
    #[case((BOTL_MAZE,CardinalDirection::East),Cell::Open)]
    fn test_peek_botl_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTR_MAZE,CardinalDirection::South),Cell::Wall)]
    #[case((BOTR_MAZE,CardinalDirection::East),Cell::Wall)]
    #[case((BOTR_MAZE,CardinalDirection::North),Cell::Open)]
    #[case((BOTR_MAZE,CardinalDirection::West),Cell::Open)]
    fn test_peek_botr_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let rob = make_robot(maze);
        let act = rob.peek(dir);

        assert_eq!(act, exp)
    }

    #[test]
    fn test_peek_all() {
        let rob = make_robot("+S \n+F+");
        let act = rob.peek_all();
        let exp = [Cell::Wall, Cell::Open, Cell::Finish, Cell::Wall];

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_peek_finish() {
        let rob = make_robot(FNSH_MAZE);
        let act = rob.peek(CardinalDirection::East);

        assert_eq!(act, Cell::Finish)
    }

    #[rstest]
    fn test_go_open(
        #[values(
            CardinalDirection::North,
            CardinalDirection::East,
            CardinalDirection::South,
            CardinalDirection::West
        )]
        direction: CardinalDirection,
    ) -> Result<(), RobotError> {
        let mut rob = make_robot(OPEN_MAZE);

        rob.go(direction)
    }
}
