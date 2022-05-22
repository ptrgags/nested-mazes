use std::collections::HashSet;

use rand::Rng;
use rand::rngs::ThreadRng;

use crate::grid_coords::{GridCoords, GRID_SIZE};
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

    pub fn clear(&mut self) {
        self.visited.clear();
        self.stack.clear();
    }

    pub fn make_maze(&mut self, grid: &mut Grid, start_cell: GridCoords) {
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