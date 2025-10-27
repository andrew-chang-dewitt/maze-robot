pub mod cardinal;
pub mod relative;

pub use cardinal::{CARD_DIR_ARR, CardinalDirection};
pub use relative::{REL_DIR_ARR, RelativeDirection};

pub trait Direction {
    fn reverse(&self) -> Self;
}

impl Direction for CardinalDirection {
    fn reverse(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

impl Direction for RelativeDirection {
    fn reverse(&self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
}
