use std::fmt::{Debug, Formatter, Result};

const GRID_SIZE: usize = 8;
const CELL_COUNT: usize = GRID_SIZE * GRID_SIZE;

const UP: usize = 0;
const DOWN: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;

#[derive(Copy, Clone)]
pub struct Cell {
    /// Flag for if the DFS has visited this node yet.
    visited: bool,
    /// 4 bit flags for connectivity to neighbor cells
    connections: [bool; 4],
    /// 4 flags for the connectivity but only for cells on the solution
    /// path
    solution_connections: [bool; 4],
}

impl Cell {
    /// Create a default cell. The caller is responsible for 
    pub fn new() -> Self {
        Self {
            visited: false,
            // By default, assume we're not connected to any neighbors
            connections: [false; 4],
            // there's no concept of a "solution" until we make the maze
            solution_connections: [false; 4],
        }
    }

    pub fn get_connection_bits(&self) -> u8 {
        let up = self.connections[UP] as u8;
        let down = self.connections[DOWN] as u8;
        let left = self.connections[LEFT] as u8;
        let right = self.connections[RIGHT] as u8;
        
        up | (down << 1) | (left << 2) | (right << 3)
    }
}


pub struct Grid {
    /// Cells, stored in row-major fashion
    cells: [Cell; CELL_COUNT],
}

impl Grid {
    pub fn new() -> Self {
        let mut cells = [Cell::new(); CELL_COUNT];
        Self::mark_boundaries(&mut cells);
        Self {
            cells
        }
    }

    fn mark_boundaries(cells: &mut [Cell; CELL_COUNT]) {
        for i in 0..GRID_SIZE {
            // top boundary
            cells[i].connections[UP] = false;

            // bottom boundary
            cells[(GRID_SIZE - 1) * GRID_SIZE + i].connections[DOWN] = false;

            // left boundary
            cells[i * GRID_SIZE].connections[LEFT] = false;

            // right boundary
            cells[i * GRID_SIZE + (GRID_SIZE - 1)].connections[RIGHT] = false;
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
                let connection_bits = 
                    self.cells[i * GRID_SIZE + j].get_connection_bits();
                let grid_char = GRID_CHARACTERS[connection_bits as usize];
                write!(f, "{}", grid_char);
            }
            write!(f, "\n");
        }

        Ok(())
    }
}