use std::fmt::Display;

use maze_robot::controller::{MazeError, Robot, RobotInternal};

use crate::text_maze::TextMaze;

#[derive(Debug)]
pub struct TextRobot(RobotInternal);

impl TryFrom<&str> for TextRobot {
    type Error = MazeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let maze = TextMaze::try_from(value)?;

        Ok(TextRobot(RobotInternal::new(maze)))
    }
}

impl Robot for TextRobot {
    fn get_internal(&self) -> &RobotInternal {
        &self.0
    }
}

impl Display for TextRobot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use maze_robot::controller::{Cell, Direction};
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

    fn make_robot(maze: &str) -> TextRobot {
        TextRobot::try_from(maze).expect("Robot creates successfully")
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
