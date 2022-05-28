mod dfs;
mod direction;
mod geometry;
mod grid;
mod grid_coords;
mod tile;
mod tileset;

use crate::tileset::MazeTileset;

fn main() {
    let tileset = MazeTileset::new("output/maze", 7);
    tileset.generate();
}
