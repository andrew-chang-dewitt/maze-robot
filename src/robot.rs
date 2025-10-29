use std::{error::Error, fmt::Display, marker::PhantomData};

use crate::{
    CARD_DIR_ARR, CardinalDirection, Cell, Maze,
    maze::{MazeError, TextMaze},
};

pub struct Robot<M, Loc>
where
    M: Maze<Loc>,
{
    state: M,
    _phantom_loc: PhantomData<Loc>,
}

impl<M, Loc> Robot<M, Loc>
where
    M: Maze<Loc>,
{
    pub fn new(state: M) -> Self {
        Robot {
            state,
            _phantom_loc: PhantomData,
        }
    }

    pub fn peek(&self, from: &Loc, direction: CardinalDirection) -> (Cell, Option<Loc>) {
        self.state.look_dir(from, direction)
    }

    pub fn peek_all(&self, from: &Loc) -> [(Cell, Option<Loc>); 4] {
        [0, 1, 2, 3].map(|idx| self.peek(from, CARD_DIR_ARR[idx]))
    }

    pub fn go(&self, from: &Loc, direction: CardinalDirection) -> Result<Loc, RobotError> {
        self.state
            .update(from, direction)
            .map_err(|e| RobotError::from((e, self.state.render(from))))
    }
}

impl Robot<TextMaze, usize> {
    pub fn try_new(state: &str) -> Result<(Self, usize), RobotError> {
        TextMaze::try_from(state)
            .map(|maze| {
                let start = maze.start;
                let robot = Robot::new(maze);

                (robot, start)
            })
            .map_err(|e| RobotError::from((e, state)))
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

    fn make_robot(maze: &str) -> (Robot<TextMaze, usize>, usize) {
        Robot::try_new(maze).expect("Robot creates successfully")
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
        let (rob, start) = make_robot(WALL_MAZE);

        match rob.peek(&start, direction) {
            (Cell::Wall, _) => (),
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
        let (rob, start) = make_robot(OPEN_MAZE);

        match rob.peek(&start, direction) {
            (Cell::Open, _) => (),
            _ => panic!("Expected peeking in {direction:?} to return Cell::Open"),
        }
    }

    #[rstest]
    // r#"S \n"
    //   "  "#;
    #[case((TOPL_MAZE,CardinalDirection::North),(Cell::Wall, None))]
    #[case((TOPL_MAZE,CardinalDirection::West), (Cell::Wall, None))]
    #[case((TOPL_MAZE,CardinalDirection::South),(Cell::Open, Some(3)))]
    #[case((TOPL_MAZE,CardinalDirection::East), (Cell::Open, Some(1)))]
    fn test_peek_topl_corner(
        #[case] (maze, dir): (&str, CardinalDirection),
        #[case] exp: (Cell, Option<usize>),
    ) {
        let (rob, start) = make_robot(maze);
        let act = rob.peek(&start, dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((TOPR_MAZE,CardinalDirection::North),Cell::Wall)]
    #[case((TOPR_MAZE,CardinalDirection::East),Cell::Wall)]
    #[case((TOPR_MAZE,CardinalDirection::South),Cell::Open)]
    #[case((TOPR_MAZE,CardinalDirection::West),Cell::Open)]
    fn test_peek_topr_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let (rob, start) = make_robot(maze);
        let (act, _) = rob.peek(&start, dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTL_MAZE,CardinalDirection::South),Cell::Wall)]
    #[case((BOTL_MAZE,CardinalDirection::West),Cell::Wall)]
    #[case((BOTL_MAZE,CardinalDirection::North),Cell::Open)]
    #[case((BOTL_MAZE,CardinalDirection::East),Cell::Open)]
    fn test_peek_botl_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let (rob, start) = make_robot(maze);
        let (act, _) = rob.peek(&start, dir);

        assert_eq!(act, exp)
    }

    #[rstest]
    #[case((BOTR_MAZE,CardinalDirection::South),Cell::Wall)]
    #[case((BOTR_MAZE,CardinalDirection::East),Cell::Wall)]
    #[case((BOTR_MAZE,CardinalDirection::North),Cell::Open)]
    #[case((BOTR_MAZE,CardinalDirection::West),Cell::Open)]
    fn test_peek_botr_corner(#[case] (maze, dir): (&str, CardinalDirection), #[case] exp: Cell) {
        let (rob, start) = make_robot(maze);
        let (act, _) = rob.peek(&start, dir);

        assert_eq!(act, exp)
    }

    #[test]
    fn test_peek_all() {
        let (rob, start) = make_robot("+S \n+F+");
        let act = rob.peek_all(&start).map(|(c, _)| c);
        let exp = [Cell::Wall, Cell::Open, Cell::Finish, Cell::Wall];

        assert_eq!(act, exp)
    }

    #[rstest]
    fn test_peek_finish() {
        let (rob, start) = make_robot(FNSH_MAZE);
        let (act, _) = rob.peek(&start, CardinalDirection::East);

        assert_eq!(act, Cell::Finish)
    }

    //  OPEN_MAZE:
    //  r#"   \n"
    //    " S \n"
    //    "   "#;
    #[rstest]
    #[case(CardinalDirection::North, 1)]
    #[case(CardinalDirection::East, 6)]
    #[case(CardinalDirection::South, 9)]
    #[case(CardinalDirection::West, 4)]
    fn test_go_open(#[case] direction: CardinalDirection, #[case] exp: usize) {
        let (rob, start) = make_robot(OPEN_MAZE);

        let act = rob
            .go(&start, direction)
            .expect("robot should move successfully");

        assert_eq!(exp, act);
    }
}
