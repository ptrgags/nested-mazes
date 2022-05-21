mod dfs;
mod direction;
mod grid;
mod grid_coords;

use crate::grid::Grid;
use crate::dfs::DFSMaze;
use crate::grid_coords::GRID_SIZE;

fn main() {
    let mut grid = Grid::new();
    grid.mark_boundaries();

    let mut maze_gen = DFSMaze::new();
    maze_gen.maze_fill(&mut grid);

    image::save_buffer(
        "output/grid.png", 
        &grid.to_image_bytes(),
        GRID_SIZE as u32, 
        GRID_SIZE as u32,
        image::ColorType::Rgb8
    ).unwrap();

    image::save_buffer(
        "output/grid_debug.png", 
        &grid.to_debug_image_bytes(),
        GRID_SIZE as u32,
        GRID_SIZE as u32,
        image::ColorType::Rgb8
    ).unwrap();

    println!("{:?}", grid);
}
