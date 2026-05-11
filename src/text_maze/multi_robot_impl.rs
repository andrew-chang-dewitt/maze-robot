use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    text_maze::MultiTextMaze,
    traits::{MazeError, MultiMaze, MultiMazeHandle, Robot, RobotInternal},
};

#[derive(Debug)]
pub struct MultiTextRobot(RobotInternal);

impl MultiTextRobot {
    pub fn swarm(text: &str, n: usize) -> Result<Vec<Self>, MazeError> {
        let maze = MultiTextMaze::try_from((text, n))?;
        let shared: Rc<RefCell<Box<dyn MultiMaze>>> = Rc::new(RefCell::new(Box::new(maze)));

        let robots = (0..n)
            .map(|id| {
                let handle = MultiMazeHandle::new(Rc::clone(&shared), id);
                MultiTextRobot(RobotInternal::new(handle))
            })
            .collect();

        Ok(robots)
    }
}

impl Robot for MultiTextRobot {
    fn get_internal(&self) -> &RobotInternal {
        &self.0
    }
}

impl Display for MultiTextRobot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Cell, Direction};
    use rstest::rstest;

    use super::*;

    const WALL_MAZE: &str = "+++\n+S+\n+++";
    const OPEN_MAZE: &str = "   \n S \n   ";
    const FNSH_MAZE: &str = "SF";
    const TOPL_MAZE: &str = "S \n  ";
    const TOPR_MAZE: &str = " S\n  ";
    const BOTL_MAZE: &str = "  \nS ";
    const BOTR_MAZE: &str = "  \n S";

    fn make_robot(maze: &str) -> MultiTextRobot {
        MultiTextRobot::swarm(maze, 1)
            .expect("swarm creates successfully")
            .into_iter()
            .next()
            .expect("swarm has at least one robot")
    }

    #[rstest]
    fn test_peek_wall(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) {
        let rob = make_robot(WALL_MAZE);

        match rob.peek(direction).expect("should not fail") {
            Cell::Wall => (),
            _ => panic!("expected Cell::Wall peeking {direction:?}"),
        }
    }

    #[rstest]
    fn test_peek_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) {
        let rob = make_robot(OPEN_MAZE);

        match rob.peek(direction).expect("should not fail") {
            Cell::Open => (),
            _ => panic!("expected Cell::Open peeking {direction:?}"),
        }
    }

    #[rstest]
    #[case((TOPL_MAZE, Direction::North), Cell::Wall)]
    #[case((TOPL_MAZE, Direction::West), Cell::Wall)]
    #[case((TOPL_MAZE, Direction::South), Cell::Open)]
    #[case((TOPL_MAZE, Direction::East), Cell::Open)]
    fn test_peek_topl_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        assert_eq!(rob.peek(dir).expect("should not fail"), exp);
    }

    #[rstest]
    #[case((TOPR_MAZE, Direction::North), Cell::Wall)]
    #[case((TOPR_MAZE, Direction::East), Cell::Wall)]
    #[case((TOPR_MAZE, Direction::South), Cell::Open)]
    #[case((TOPR_MAZE, Direction::West), Cell::Open)]
    fn test_peek_topr_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        assert_eq!(rob.peek(dir).expect("should not fail"), exp);
    }

    #[rstest]
    #[case((BOTL_MAZE, Direction::South), Cell::Wall)]
    #[case((BOTL_MAZE, Direction::West), Cell::Wall)]
    #[case((BOTL_MAZE, Direction::North), Cell::Open)]
    #[case((BOTL_MAZE, Direction::East), Cell::Open)]
    fn test_peek_botl_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        assert_eq!(rob.peek(dir).expect("should not fail"), exp);
    }

    #[rstest]
    #[case((BOTR_MAZE, Direction::South), Cell::Wall)]
    #[case((BOTR_MAZE, Direction::East), Cell::Wall)]
    #[case((BOTR_MAZE, Direction::North), Cell::Open)]
    #[case((BOTR_MAZE, Direction::West), Cell::Open)]
    fn test_peek_botr_corner(#[case] (maze, dir): (&str, Direction), #[case] exp: Cell) {
        let rob = make_robot(maze);
        assert_eq!(rob.peek(dir).expect("should not fail"), exp);
    }

    #[test]
    fn test_peek_finish() {
        let rob = make_robot(FNSH_MAZE);
        assert_eq!(
            rob.peek(Direction::East).expect("should not fail"),
            Cell::Finish
        );
    }

    #[rstest]
    fn test_go_open(
        #[values(Direction::North, Direction::East, Direction::South, Direction::West)]
        direction: Direction,
    ) -> Result<(), MazeError> {
        let rob = make_robot(OPEN_MAZE);
        rob.go(direction)
    }

    #[test]
    fn test_render() {
        let rob = make_robot(OPEN_MAZE);
        assert_eq!(rob.to_string(), "Robot state:\n   \n 0 \n   ");
    }
}
