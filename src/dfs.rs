use std::collections::HashSet;

use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid_coords::{GridCoords, GRID_SIZE};
use crate::direction::Direction;
use crate::grid::Grid;

pub struct DFSMaze {
    visited: HashSet<GridCoords>,
    stack: Vec<GridCoords>,
    rng: ThreadRng
}

impl DFSMaze {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            stack: Vec::new(),
            rng: rand::thread_rng()
        }
    }

    pub fn make_maze(&mut self, grid: &mut Grid, start_cell: GridCoords) {
        self.stack.clear();
        self.stack.push(start_cell);

        while self.stack.len() > 0 {
            let current = self.stack[self.stack.len() - 1];
            self.visited.insert(current);

            let unvisited_neighbors: Vec<GridCoords> = current.get_neighbors()
                .into_iter()
                .filter(|x| !self.visited.contains(x) && grid.can_connect(current, *x))
                .collect();

            // Out of options so backtrack
            if unvisited_neighbors.len() == 0 {
                self.stack.pop();
                continue;
            }

            // Two roads diverged in a wood and I randomly picked one.
            let rand_index: usize = 
                self.rng.gen_range(0..unvisited_neighbors.len());
            let neighbor = unvisited_neighbors[rand_index];
            grid.connect(current, neighbor);
            self.stack.push(neighbor);
        }
    }

    pub fn maze_fill(&mut self, grid: &mut Grid) {
        self.visited.clear();

        // we need to make a DFS forest since child tiles may have multiple
        // disjoint sections. 
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                let coords = GridCoords {x, y};
                if !self.visited.contains(&coords) {
                    self.make_maze(grid, coords);
                }
            }
        }
    }
}

type ExitCoords = (GridCoords, Direction);

pub struct DFSSolutionFinder {
    stack: Vec<GridCoords>,
    path: Vec<Direction>,
    visited_cells: HashSet<GridCoords>,
    visited_exits: HashSet<ExitCoords>,
}

impl DFSSolutionFinder {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            visited_cells: HashSet::new(),
            visited_exits: HashSet::new(),
            stack: Vec::new(),
        }
    }

    pub fn solve_all_paths(&mut self, grid: &mut Grid) {
        self.visited_exits.clear();
        self.visited_cells.clear();
        for exit in grid.get_all_exits() {
            if self.visited_exits.contains(&exit) {
                continue;
            }
            self.visited_exits.insert(exit);
            self.solve_single_path(grid, exit);
        }
    }

    fn solve_single_path(&mut self, grid: &mut Grid, first_exit: ExitCoords) {
        self.stack.clear();
        self.path.clear();

        let (start_cell, _) = first_exit;
        self.stack.push(start_cell);

        while self.stack.len() > 0 {
            let current = self.stack[self.stack.len() - 1];
            self.visited_cells.insert(current);

            // Check for an exit
            for exit_dir in grid.get_exit_directions(current) {
                let exit_coords = (current, exit_dir);

                // Filter out the entrance
                if !self.visited_exits.contains(&exit_coords) {
                    // We found the exit!
                    self.visited_exits.insert(exit_coords);
                    self.connect_solution_path(grid, start_cell);
                    return;
                }
            }

            // Look for unvisited, connected neighbors
            let unvisited_neighbors: Vec<GridCoords> = current.get_neighbors()
                .into_iter()
                .filter(|x| {
                    !self.visited_cells.contains(x) && 
                    grid.is_connected(current, *x)
                })
                .collect();

            if unvisited_neighbors.len() == 0 {
                self.stack.pop();
                self.path.pop();
                continue;
            }

            // Try the first option
            let selected_neighbor = unvisited_neighbors[0];
            let selected_direction = 
                GridCoords::get_direction(current, selected_neighbor)
                .unwrap();
            self.stack.push(selected_neighbor);
            self.path.push(selected_direction);
        }
    }

    fn connect_solution_path(&self, grid: &mut Grid, start_cell: GridCoords) {
        let mut current_cell = start_cell;
        for direction in &self.path {
            let neighbor = current_cell.get_adjacent(*direction);
            grid.connect_solution(current_cell, neighbor);
            current_cell = neighbor;
        }
    }
}