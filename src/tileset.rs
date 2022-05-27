use std::fs::{create_dir_all, copy, remove_dir_all, write};
use std::path::Path;

use serde_json::{json, to_string_pretty};

use crate::dfs::DFSMaze;
use crate::tile::Tile;
use crate::geometry::make_buffer;

pub struct MazeTileset {
    output_directory: String,
    levels: usize
}

impl MazeTileset {
    pub fn new(output_directory: &str, levels: usize) -> Self {
        Self {
            output_directory: output_directory.to_string(), 
            levels
        }
    }

    pub fn generate(&self) {
        self.init_directory();
        
        self.generate_common_files();
        self.generate_maze();
    }

    fn init_directory(&self) {
        remove_dir_all(&self.output_directory)
            .expect("could not remove output directory");

        let tiles_dir = Path::new(&self.output_directory).join("tiles");
        create_dir_all(tiles_dir)
            .expect("could not create tiles directory");
    }

    fn generate_common_files(&self) {
        self.generate_tileset_json();

        let subtree_file = Path::new(&self.output_directory)
            .join("0.0.0.subtree.json");
        copy("assets/subtree.json", subtree_file)
            .expect("could not copy subtree file");

        let tileset_file = Path::new(&self.output_directory)
            .join("tileset_walls.png");
        copy("assets/walls-test.png", tileset_file)
            .expect("could not copy tileset");

        let geometry_data = make_buffer();
        let geometry_path = Path::new(&self.output_directory)
            .join("tiles/geometry.bin");
        write(geometry_path, geometry_data)
            .expect("could not write geometry buffer");
    }

    fn generate_tileset_json(&self) {
        let tileset_json = json!({
            "asset": {
                "version": "1.1",
            },
            "geometricError": 1 << (self.levels + 1),
            "schema": {
                "classes": {
                    "tileset": {
                        "properties": {
                            "wall_tileset_uri": {
                                "type": "STRING"
                            },
                            "wall_tileset_uri": {
                                "type": "STRING"
                            }
                        }
                    }
                }
            },
            "metadata": {
                "class": "tileset",
                "properties": {
                    "wall_tileset_uri": "tileset_walls.png",
                    "connection_tileset_uri": "tileset_connections.png"
                }
            },
            "refine": "REPLACE",
            "root": {
                "boundingVolume": {
                    "box": [
                        0, 0, 0,
                        1, 0, 0,
                        0, 1, 0,
                        0, 0, 1
                    ]
                },
                "geometricError": 1 << self.levels,
                "content": {
                    "uri": "tiles/{level}.{x}.{y}.glb"
                },
                "implicitTiling": {
                    "subdivisionScheme": "QUADTREE",
                    "availableLevels": self.levels,
                    "subtreeLevels": self.levels,
                    "subtrees": {
                        "uri": "{level}.{x}.{y}.subtree.json"
                    }
                }
            }
        });

        let tileset_path = Path::new(&self.output_directory)
            .join("tileset.json");
        let json_string = to_string_pretty(&tileset_json)
            .expect("Could not serialize JSON");
        write(tileset_path, json_string)
            .expect("could not write tileset JSON");
    }

    fn generate_maze(&self) {
        let mut maze_gen = DFSMaze::new();
        let root = Tile::make_root(&mut maze_gen);
        let mut stack = vec![root];

        let tiles_dir = Path::new(&self.output_directory).join("tiles");

        // depth-first pre-order generation of the tileset using a stack
        while let Some(tile) = stack.pop() {
            tile.write_glb(&tiles_dir);

            if tile.level < self.levels - 1 {
                let child_tiles = tile.subdivide(&mut maze_gen);

                // Since we're using a stack, push the tiles
                // backwards so the DFS is more like Morton order.
                stack.extend(child_tiles.into_iter().rev());
            }
        }
    }
}