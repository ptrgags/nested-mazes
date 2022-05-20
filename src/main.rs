mod dfs;
mod direction;
mod grid;
mod grid_coords;

use crate::grid::Grid;
use crate::dfs::DFSMaze;

fn main() {
    let mut grid = Grid::new();
    grid.mark_boundaries();

    let mut maze_gen = DFSMaze::new();
    maze_gen.maze_fill(&mut grid);

    println!("{:?}", grid);
}
