use std::fmt::{Debug, Formatter, Result};

use crate::grid_coords::{GridCoords, GRID_SIZE};
use crate::direction::Direction;

const CELL_COUNT: usize = GRID_SIZE as usize * GRID_SIZE as usize;

#[derive(Copy, Clone)]
pub struct Connection {
    connected: bool,
    blocked: bool,
    is_solution_connection: bool,
    is_maze_exit: bool,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            connected: false,
            blocked: false,
            is_solution_connection: false,
            is_maze_exit: false
        }
    }
}

#[derive(Copy, Clone)]
pub struct Cell {
    /// 4 connections to neighboring cells. This connection struct will have
    /// the same value as 
    connections: [Connection; 4],
}

impl Cell {
    /// Create a default cell. The caller is responsible for 
    pub fn new() -> Self {
        Self {
            // By default, assume we're not connected to any neighbors
            connections: [Connection::new(); 4]
        }
    }

    pub fn get_connection_bits(&self) -> u8 {
        let up = self.connections[Direction::Up.to_index()].connected as u8;
        let down = self.connections[Direction::Down.to_index()].connected as u8;
        let left = self.connections[Direction::Left.to_index()].connected as u8;
        let right = self.connections[Direction::Right.to_index()].connected as u8;
        
        up | (down << 1) | (left << 2) | (right << 3)
    }
}


pub struct Grid {
    /// Cells, stored in row-major fashion
    cells: [Cell; CELL_COUNT],
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: [Cell::new(); CELL_COUNT]
        }
    }

    pub fn get_cell(&self, coords: GridCoords) -> &Cell {
        &self.cells[coords.to_index()]
    }

    pub fn get_cell_mut(&mut self, coords: GridCoords) -> &mut Cell {
        &mut self.cells[coords.to_index()]
    }

    pub fn connect(&mut self, a: GridCoords, b: GridCoords) {
        let direction = match GridCoords::get_direction(a, b) {
            Some(dir) => dir,
            None => panic!("Connect can only be called on adjacent coordinates")
        };
        let opposite_dir = direction.get_opposite();

        self.get_cell_mut(a).connections[direction.to_index()].connected = true;
        self.get_cell_mut(b).connections[opposite_dir.to_index()].connected = true;
    }

    pub fn mark_boundaries(&mut self) {
        for i in 0..GRID_SIZE {
            // top boundary
            self.cells[i]
                .connections[Direction::Up.to_index()].blocked = true;

            // bottom boundary
            self.cells[(GRID_SIZE - 1) * GRID_SIZE + i]
                .connections[Direction::Down.to_index()].blocked = true;

            // left boundary
            self.cells[i * GRID_SIZE]
                .connections[Direction::Left.to_index()].blocked = true;

            // right boundary
            self.cells[i * GRID_SIZE + (GRID_SIZE - 1)]
                .connections[Direction::Right.to_index()].blocked = true;
        }
    }
}

/// Print the grid 
const GRID_CHARACTERS: [char; 16] = [
    // bits are:
    // right left down up
    ' ', // 0b0000
    '╵', // 0b0001
    '╷', // 0b0010
    '│', // 0b0011
    '╴', // 0b0100
    '┘', // 0b0101
    '┐', // 0b0110
    '┤', // 0b0111
    '╶', // 0b1000
    '└', // 0b1001
    '┌', // 0b1010
    '├', // 0b1011
    '─', // 0b1100
    '┴', // 0b1101
    '┬', // 0b1110
    '┼', // 0b1111
];

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let row = (GRID_SIZE - 1) - i;
                let index = row as usize * GRID_SIZE as usize + j as usize;
                let connection_bits = 
                    self.cells[index].get_connection_bits();
                let grid_char = GRID_CHARACTERS[connection_bits as usize];
                write!(f, "{}", grid_char);
            }
            write!(f, "\n");
        }

        Ok(())
    }
}