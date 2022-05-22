mod dfs;
mod direction;
mod grid;
mod grid_coords;

use crate::grid::Grid;
use crate::dfs::DFSMaze;
use crate::grid_coords::GRID_SIZE;
use crate::direction::Direction;

fn main() {
    let mut grid = Grid::new();
    grid.mark_boundaries();

    let mut maze_gen = DFSMaze::new();
    maze_gen.maze_fill(&mut grid);

    grid.mark_exit(Direction::Down, 3);
    grid.mark_exit(Direction::Up, 5);

    println!("{:?}", grid);

    let mut sw = Grid::new();
    let mut se = Grid::new();
    let mut nw = Grid::new();
    let mut ne = Grid::new();
    // TODO: propagate boundary constraints
    //child_grid.mark_boundaries();
    grid.propagate_interior(&mut sw, 0..4, 0..4);
    grid.propagate_interior(&mut se, 4..8, 0..4);
    grid.propagate_interior(&mut nw, 0..4, 4..8);
    grid.propagate_interior(&mut ne, 4..8, 4..8);

    maze_gen.clear();
    maze_gen.maze_fill(&mut sw);
    maze_gen.clear();
    maze_gen.maze_fill(&mut se);
    maze_gen.clear();
    maze_gen.maze_fill(&mut nw);
    maze_gen.clear();
    maze_gen.maze_fill(&mut ne);

    println!("{:?}{:?}", nw, sw);

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

}
