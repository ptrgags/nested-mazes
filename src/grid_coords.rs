use crate::direction::Direction;

// Each tile is a fixed 8x8 grid
pub const GRID_SIZE: usize = 8;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct GridCoords {
    pub x: usize,
    // using a y-up coordinate system.
    pub y: usize
}

impl GridCoords {
    pub fn to_index(&self) -> usize {
        self.y * GRID_SIZE + self.x
    }

    pub fn get_neighbors(&self) -> Vec<Self> {
        let mut result = Vec::new();
        if self.x > 0 {
            result.push(Self {
                x: self.x - 1, 
                y: self.y
            });
        }

        if self.x < GRID_SIZE - 1 {
            result.push(Self {
                x: self.x + 1,
                y: self.y
            });
        }

        if self.y > 0 {
            result.push(Self {
                x: self.x,
                y: self.y - 1
            });
        }

        if self.y < GRID_SIZE - 1 {
            result.push(Self {
                x: self.x,
                y: self.y + 1
            });
        }

        result
    }

    pub fn get_direction(from: Self, to: Self) -> Option<Direction> {
        let dx = to.x as isize - from.x as isize;
        let dy = to.y as isize - from.y as isize;

        match (dx, dy)  {
            (1, 0) => Some(Direction::Right),
            (-1, 0) => Some(Direction::Left),
            (0, 1) => Some(Direction::Up),
            (0, -1) => Some(Direction::Down),
            _ => None
        } 
    }
}