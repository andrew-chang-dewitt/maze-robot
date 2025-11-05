use std::{cell::RefCell, error::Error, fmt::Display};

use crate::{
    Cell, DIR_ARR, Direction, Maze,
    maze::{MazeError, TextMaze},
    solution::Key,
};

#[derive(Debug)]
pub struct Robot {
    // maze is actually an _external_ enviroment the robot exists _inside_ of
    // the robot has no notion of state itself--it simply looks & attempts to travel in specified
    // directions. as far as it is concerned, no state changes happen for either of those actions.
    // to model this, the **interior mutability** pattern is used by placing the env inside a
    // RefCell inside our Robot, then calling env methods on it to perform Robot actions w/out
    // worrying about any side effects.
    //
    // additionally, a dynamic trait object is used here since the robot doesn't care what kind of
    // maze environment it is in--it always works the same. this keeps the user of Robot from
    // having to know anything about the Maze construct.
    env: RefCell<Box<dyn Maze>>,
}

impl Robot {
    pub fn peek(&self, direction: Direction) -> Cell {
        self.env.borrow().look_dir(direction)
    }

    pub fn peek_all(&self) -> [(Cell, Direction); 4] {
        DIR_ARR.map(|dir| (self.peek(dir), dir))
    }

    pub fn go(&self, direction: Direction) -> Result<(), MazeError> {
        #[cfg(test)]
        {
            println!("[Robot::go] BEGIN go {direction} from {self}");
        }
        self.env
            .borrow_mut()
            .move_dir(direction)
            .map_err(|e| e.into())
    }
}

impl TryFrom<&str> for Robot {
    type Error = MazeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let maze = TextMaze::try_from(value)?;

        Ok(Robot {
            env: RefCell::new(Box::new(maze)),
        })
    }
}

impl Display for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.env.borrow().to_string();

        write!(f, "Robot state:\n{state}")
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

    fn make_robot(maze: &str) -> Robot {
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
    ) -> Result<(), MazeError> {
        let rob = make_robot(OPEN_MAZE);

        rob.go(direction)
    }

    #[rstest]
    fn test_render() {
        let rob = make_robot(OPEN_MAZE);
        let act = rob.to_string();
        let exp = "Robot state:\n   \n X \n   ";

        assert_eq!(act, exp)
    }
}
