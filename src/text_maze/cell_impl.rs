use maze_robot::controller::Cell;

#[derive(Debug)]
pub struct TextCell(Cell);

/// Map text characters to Cell values, where
/// - 'S' is starting location
/// - 'F' is ending location
/// - '+' is a wall
/// - all others are considered open
impl From<&char> for TextCell {
    fn from(value: &char) -> Self {
        Self(match value {
            'F' => Cell::Finish,
            '+' | '\n' => Cell::Wall,
            _ => Cell::Open,
        })
    }
}

/// convert Cell to TextCell wrapper type
impl From<Cell> for TextCell {
    fn from(value: Cell) -> Self {
        Self(value)
    }
}

/// unwrap TextCell to get underlying Cell type
impl Into<Cell> for TextCell {
    fn into(self) -> Cell {
        self.0
    }
}
