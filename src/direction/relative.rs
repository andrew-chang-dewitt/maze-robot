use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RelativeDirection {
    Forward,
    Right,
    Backward,
    Left,
}

pub const REL_DIR_ARR: [RelativeDirection; 4] = [
    RelativeDirection::Forward,
    RelativeDirection::Right,
    RelativeDirection::Backward,
    RelativeDirection::Left,
];

impl Display for RelativeDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::Forward => "Forward",
            Self::Right => "Right",
            Self::Backward => "Backward",
            Self::Left => "Left",
        };

        write!(f, "{out}")
    }
}
