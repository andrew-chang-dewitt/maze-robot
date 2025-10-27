use std::fmt::Display;

use crate::RelativeDirection;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}

impl From<&CardinalDirection> for usize {
    fn from(value: &CardinalDirection) -> Self {
        match value {
            &CardinalDirection::North => 0,
            &CardinalDirection::East => 1,
            &CardinalDirection::South => 2,
            &CardinalDirection::West => 3,
        }
    }
}

impl From<CardinalDirection> for usize {
    fn from(value: CardinalDirection) -> Self {
        Self::from(&value)
    }
}

impl CardinalDirection {
    pub fn from_relative(&self, relative: RelativeDirection) -> Self {
        // convert cardinal to relative, wrapping around from 3 to 0 as necessary
        let ord = match relative {
            // direction doesn't change
            RelativeDirection::Forward => usize::from(self),
            // direction increases by 1
            RelativeDirection::Right => usize::from(self) + 1,
            // direction increases by 2
            RelativeDirection::Backward => usize::from(self) + 2,
            // direction increases by 3
            RelativeDirection::Left => usize::from(self) + 3,
        };

        if ord >= 4 {
            CARD_DIR_ARR[ord - 4]
        } else {
            CARD_DIR_ARR[ord]
        }
    }
}

pub const CARD_DIR_ARR: [CardinalDirection; 4] = [
    CardinalDirection::North,
    CardinalDirection::East,
    CardinalDirection::South,
    CardinalDirection::West,
];

impl Display for CardinalDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::North => "North",
            Self::East => "East",
            Self::South => "South",
            Self::West => "West",
        };

        write!(f, "{out}")
    }
}
