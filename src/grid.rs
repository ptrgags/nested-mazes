use std::ops::Range;
use std::fmt::{Debug, Formatter, Result};

use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid_coords::{GridCoords, GRID_SIZE};
use crate::direction::Direction;

const CELL_COUNT: usize = GRID_SIZE * GRID_SIZE;
const HALF_GRID_SIZE: usize = GRID_SIZE / 2;

// RGB image
const IMAGE_SIZE: usize = CELL_COUNT * 3;

#[derive(Copy, Clone)]
pub struct Connection {
    connected: bool,
    blocked: bool,
    is_solution_connection: bool,
    is_maze_exit: bool,
    split_bits: u16
}

impl Connection {
    pub fn new() -> Self {
        Self {
            connected: false,
            blocked: false,
            is_solution_connection: false,
            is_maze_exit: false,
            split_bits: 0
        }
    }

    pub fn is_boundary_exit(&self) -> bool {
        self.is_maze_exit || self.is_solution_connection
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
        let up = self.connections[Direction::Up.to_index()]
            .connected as u8;
        let down = self.connections[Direction::Down.to_index()]
            .connected as u8;
        let left = self.connections[Direction::Left.to_index()]
            .connected as u8;
        let right = self.connections[Direction::Right.to_index()]
            .connected as u8;
        
        up | (down << 1) | (left << 2) | (right << 3)
    }

    pub fn get_solution_bits(&self) -> u8 {
        let up = self.connections[Direction::Up.to_index()]
            .is_solution_connection as u8;
        let down = self.connections[Direction::Down.to_index()]
            .is_solution_connection as u8;
        let left = self.connections[Direction::Left.to_index()]
            .is_solution_connection as u8;
        let right = self.connections[Direction::Right.to_index()]
            .is_solution_connection as u8;
        
        up | (down << 1) | (left << 2) | (right << 3)
    }
}


pub struct Grid {
    /// Cells, stored in row-major fashion, but the rows are y-up
    cells: [Cell; CELL_COUNT],
    rng: ThreadRng
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: [Cell::new(); CELL_COUNT],
            rng: rand::thread_rng()
        }
    }

    pub fn get_cell(&self, coords: GridCoords) -> &Cell {
        &self.cells[coords.to_index()]
    }

    pub fn get_cell_mut(&mut self, coords: GridCoords) -> &mut Cell {
        &mut self.cells[coords.to_index()]
    }

    pub fn can_connect(&mut self, a: GridCoords, b:GridCoords) -> bool {
        let direction = match GridCoords::get_direction(a, b) {
            Some(dir) => dir,
            None => panic!("can_connect can only be called on adjacent coordinates")
        };

        let connection = &self.get_cell(a).connections[direction.to_index()];
        !connection.blocked
    }

    pub fn is_connected(&mut self, a: GridCoords, b:GridCoords) -> bool {
        let direction = match GridCoords::get_direction(a, b) {
            Some(dir) => dir,
            None => panic!("is_connected can only be called on adjacent coordinates")
        };

        let connection = &self.get_cell(a).connections[direction.to_index()];
        connection.connected
    }

    pub fn connect(&mut self, a: GridCoords, b: GridCoords) {
        let direction = match GridCoords::get_direction(a, b) {
            Some(dir) => dir,
            None => panic!("connect can only be called on adjacent coordinates")
        };
        let opposite_dir = direction.get_opposite();

        self.get_cell_mut(a).connections[direction.to_index()].connected = true;
        self.get_cell_mut(b).connections[opposite_dir.to_index()].connected = true;
    }

    pub fn connect_solution(&mut self, a: GridCoords, b: GridCoords) {
        let direction = match GridCoords::get_direction(a, b) {
            Some(dir) => dir,
            None => panic!("connect_solution can only be called on adjacent coordinates")
        };
        let opposite_dir = direction.get_opposite();

        self.get_cell_mut(a).connections[direction.to_index()].is_solution_connection = true;
        self.get_cell_mut(b).connections[opposite_dir.to_index()].is_solution_connection = true;
    }

    pub fn get_exit_directions(&self, current: GridCoords) -> Vec<Direction> {
        let cell = self.get_cell(current);
        let directions = [
            Direction::Right,
            Direction::Up,
            Direction::Left,
            Direction::Down
        ];

        directions
            .into_iter()
            .filter(|d| {
                let connection = &cell.connections[d.to_index()];
                // For seams between tiles, is_solution_connection
                // acts like an exit.
                connection.is_maze_exit || connection.is_solution_connection
            }).collect()
    }

    pub fn get_all_exits(&self) -> Vec<(GridCoords, Direction)> {
        let mut result = Vec::new();

        for i in 0..GRID_SIZE {
            // bottom boundary
            let mut connection = &self.cells[i]
                .connections[Direction::Down.to_index()];
            
            if connection.is_boundary_exit() {
                result.push((GridCoords {x: i, y: 0}, Direction::Down));
            }

            // top boundary
            connection = &self.cells[(GRID_SIZE - 1) * GRID_SIZE + i]
                .connections[Direction::Up.to_index()];

            if connection.is_boundary_exit() {
                result.push(
                    (GridCoords {x: i, y: GRID_SIZE - 1}, Direction::Up)
                );
            }

            // left boundary
            connection = &self.cells[i * GRID_SIZE]
                .connections[Direction::Left.to_index()];

            if connection.is_boundary_exit() {
                result.push((GridCoords {x: 0, y: i}, Direction::Left));
            }

            // right boundary
            connection = &self.cells[i * GRID_SIZE + (GRID_SIZE - 1)]
                .connections[Direction::Right.to_index()];
            
            if connection.is_boundary_exit() {
                result.push(
                    (GridCoords {x: GRID_SIZE - 1, y: i}, Direction::Right)
                );
            }
        }

        result
    }

    pub fn to_image_bytes(&self) -> [u8; IMAGE_SIZE] {
        let mut result = [0; IMAGE_SIZE];
        for row in 0..GRID_SIZE {
            let y = (GRID_SIZE - 1) - row;
            for x in 0..GRID_SIZE {
                let index = y * GRID_SIZE + x;
                let cell = &self.cells[index];
                // Red channel is the connection bits
                result[3 * index] = cell.get_connection_bits();
                // Blue channel is the solution bits
                result[3 * index + 1] = cell.get_solution_bits();
                // Green channel was already initialized to 0
            }
        }
        
        result
    }

    pub fn to_debug_image_bytes(&self) -> [u8; IMAGE_SIZE] {
        let mut image_bytes = self.to_image_bytes();
        for i in 0..image_bytes.len() {
            // Increase the contrast by shifting the 4 connection bits
            // into the 4 high bits of each byte.
            // For the green channel, 0 << 4 == 0 so this is a safe operation.
            image_bytes[i] <<= 4;
        }
        image_bytes
    }

    pub fn mark_boundaries(&mut self) {
        for i in 0..GRID_SIZE {
            // bottom boundary
            self.cells[i]
                .connections[Direction::Down.to_index()].blocked = true;

            // top boundary
            self.cells[(GRID_SIZE - 1) * GRID_SIZE + i]
                .connections[Direction::Up.to_index()].blocked = true;

            // left boundary
            self.cells[i * GRID_SIZE]
                .connections[Direction::Left.to_index()].blocked = true;

            // right boundary
            self.cells[i * GRID_SIZE + (GRID_SIZE - 1)]
                .connections[Direction::Right.to_index()].blocked = true;
        }
    }

    pub fn mark_exit(&mut self, direction: Direction, index: usize) {
        let (x, y) = match direction {
            Direction::Right => (GRID_SIZE - 1, index),
            Direction::Left => (0, index),
            Direction::Up => (index, GRID_SIZE - 1),
            Direction::Down => (index, 0)
        };

        let cell = &mut self.cells[y * GRID_SIZE + x];
        let connection = &mut cell.connections[direction.to_index()];

        // Make a connection that leads "outside" the maze
        connection.blocked = false;
        connection.connected = true;
        connection.is_maze_exit = true;
        connection.is_solution_connection = true;
        // Assign some random bits so when we subdivide we know where exactly
        // to put the exit as we zoom in.
        connection.split_bits = self.rng.gen::<u16>();
    }

    pub fn get_horizontal_seam(
        &self, 
        y: usize, 
        direction: Direction,
    ) -> [Connection; GRID_SIZE] {
        let mut result = [Connection::new(); GRID_SIZE];
        for x in 0..GRID_SIZE {
            result[x] = self.cells[y * GRID_SIZE + x]
                .connections[direction.to_index()];
        }

        result
    }

    pub fn get_vertical_seam(
        &self, 
        x: usize,
        direction: Direction,
    ) -> [Connection; GRID_SIZE] {
        let mut result = [Connection::new(); GRID_SIZE];
        for y in 0..GRID_SIZE {
            result[y] = self.cells[y * GRID_SIZE + x]
                .connections[direction.to_index()];
        }

        result
    }

    pub fn set_boundary(
        &mut self,
        right: &[Connection],
        top: &[Connection],
        left: &[Connection],
        bottom: &[Connection],
    ) {
        self.set_horizontal_boundary(0, bottom, Direction::Down);
        self.set_horizontal_boundary(GRID_SIZE - 1, top, Direction::Up);

        self.set_vertical_boundary(0, left, Direction::Left);
        self.set_vertical_boundary(GRID_SIZE - 1, right, Direction::Right);
    }

    fn set_horizontal_boundary(
        &mut self, 
        y: usize,
        boundary: &[Connection],
        direction: Direction
    ) {
        let row_offset = y * GRID_SIZE;
        let direction_index = direction.to_index();
        for x in 0..HALF_GRID_SIZE {
            let connection = boundary[x];

            let child_x = 2 * x;

            // Each connection subdivides into two connections in the child
            // which has twice the resolution.
            let a_index = row_offset + child_x;
            let b_index = row_offset + (child_x + 1);

            // If it was a wall in the parent, then both halves are a wall
            // in the child
            if connection.blocked {
                self.cells[a_index].connections[direction_index].blocked = true;
                self.cells[b_index].connections[direction_index].blocked = true;
                continue;
            }

            // The split bits determine where exactly the connection is
            let split_bit = connection.split_bits & 1;
            let other_half = (!split_bit) & 1;
            let remaining_bits = connection.split_bits >> 1;

            let split_index = row_offset + (child_x + split_bit as usize);
            let other_index = row_offset + (child_x + other_half as usize);

            {
                // Propagate the connection wherever the split bit indicated
                let split_connection = &mut self.cells[split_index]
                    .connections[direction_index];
                split_connection.connected = connection.connected;
                split_connection.is_solution_connection = connection.is_solution_connection;
                split_connection.is_maze_exit = connection.is_maze_exit;

                // for further subdivision
                split_connection.split_bits = remaining_bits;
            }

            {
                // The other half will be blocked
                let other_connection = &mut self.cells[other_index]
                    .connections[direction_index];
                other_connection.connected = false;
                other_connection.blocked = false;
                other_connection.is_solution_connection = false;
                other_connection.is_maze_exit = false;
                other_connection.split_bits = 0;
            }
        }
    }

    fn set_vertical_boundary(
        &mut self, 
        x: usize,
        boundary: &[Connection],
        direction: Direction
    ) {
        let direction_index = direction.to_index();
        for y in 0..HALF_GRID_SIZE {
            let connection = boundary[y];

            let child_y = 2 * y;

            // Each connection subdivides into two connections in the child
            // which has twice the resolution.
            let a_index = child_y * GRID_SIZE + x;
            let b_index = (child_y + 1) * GRID_SIZE + x;

            // If it was a wall in the parent, then both halves are a wall
            // in the child
            if connection.blocked {
                self.cells[a_index].connections[direction_index].blocked = true;
                self.cells[b_index].connections[direction_index].blocked = true;
                continue;
            }

            // The split bits determine where exactly the connection is
            let split_bit = connection.split_bits & 1;
            let other_half = (!split_bit) & 1;
            let remaining_bits = connection.split_bits >> 1;

            let split_index = (child_y + split_bit as usize) * GRID_SIZE + x;
            let other_index = (child_y + other_half as usize) * GRID_SIZE + x;
            {
                // Propagate the connection wherever the split bit indicated
                let split_connection = &mut self.cells[split_index]
                    .connections[direction_index];
                split_connection.connected = connection.connected;
                split_connection.is_solution_connection = connection.is_solution_connection;
                split_connection.is_maze_exit = connection.is_maze_exit;

                // for further subdivision
                split_connection.split_bits = remaining_bits;
            }

            {
                // The other half will be blocked
                let other_connection = &mut self.cells[other_index]
                    .connections[direction_index];
                other_connection.connected = false;
                other_connection.blocked = false;
                other_connection.is_solution_connection = false;
                other_connection.is_maze_exit = false;
                other_connection.split_bits = 0;
            }
        }
    }

    pub fn propagate_interior(
        &self,
        child: &mut Self,
        x_range: Range<usize>,
        y_range: Range<usize>
    ) {
        // propagate vertical walls on the right edge of cells
        const RIGHT_INDEX: usize = Direction::Right.to_index();
        const LEFT_INDEX: usize = Direction::Left.to_index();
        for y in y_range.clone() {
            for x in x_range.start..(x_range.end - 1) {
                let parent_cell = &self.cells[y * GRID_SIZE + x];
                let parent_right = 
                    &parent_cell.connections[RIGHT_INDEX];

                // If there's no wall, skip
                if parent_right.connected {
                    continue;
                }

                // At the next level of detail, one wall becomes two adjacent
                // walls. Both need to be marked as blocked
                let child_x = 2 * (x % 4) + 1;
                let child_y = 2 * (y % 4);
                child.cells[child_y * GRID_SIZE + child_x]
                    .connections[RIGHT_INDEX].blocked = true;
                child.cells[(child_y + 1) * GRID_SIZE + child_x]
                    .connections[RIGHT_INDEX].blocked = true;

                // Also mark the opposite side of the connection
                child.cells[child_y * GRID_SIZE + (child_x + 1)]
                    .connections[LEFT_INDEX].blocked = true;
                child.cells[(child_y + 1) * GRID_SIZE + (child_x + 1)]
                    .connections[LEFT_INDEX].blocked = true;
            }
        }

        // Same thing but for horizontal walls
        const UP_INDEX: usize = Direction::Up.to_index();
        const DOWN_INDEX: usize = Direction::Down.to_index();
        for y in y_range.start..(y_range.end - 1) {
            for x in x_range.clone() {
                let parent_cell = &self.cells[y * GRID_SIZE + x];
                let parent_up = 
                    &parent_cell.connections[UP_INDEX];

                // If there's no wall, skip
                if parent_up.connected {
                    continue;
                }

                // At the next level of detail, one wall becomes two adjacent
                // walls. Both need to be marked as blocked
                let child_x = 2 * (x % 4);
                let child_y = 2 * (y % 4) + 1;
                child.cells[child_y * GRID_SIZE + child_x]
                    .connections[UP_INDEX].blocked = true;
                child.cells[child_y * GRID_SIZE + (child_x + 1)]
                    .connections[UP_INDEX].blocked = true;

                // Also mark the opposite side of the connection
                child.cells[(child_y + 1) * GRID_SIZE + child_x]
                    .connections[DOWN_INDEX].blocked = true;
                child.cells[(child_y + 1) * GRID_SIZE + (child_x + 1)]
                    .connections[DOWN_INDEX].blocked = true;
            }
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
            let row = (GRID_SIZE - 1) - i;
            for j in 0..GRID_SIZE {
                let index = row * GRID_SIZE + j;
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