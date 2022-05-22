#[derive(Copy, Clone)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down
}

impl Direction {
    pub const fn to_index(&self) -> usize {
        *self as usize
    }

    pub fn get_opposite(&self) -> Self {
        match *self {
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Down => Self::Up,
        }
    }
}