use std::path::Path;
use std::io::Cursor;

use chrono::{Datelike, Utc};
use serde_json::json;

use crate::grid::Grid;
use crate::direction::Direction;
use crate::grid_coords::GRID_SIZE;
use crate::dfs::DFSMaze;

const HALF_GRID_SIZE: usize = GRID_SIZE / 2;

pub struct Tile {
    pub level: usize,
    pub x: usize,
    pub y: usize,
    grid: Grid,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            level: 0,
            x: 0,
            y: 0,
            grid: Grid::new()
        }
    }

    pub fn make_root(maze_gen: &mut DFSMaze) -> Self {
        let mut root = Self::new();
        root.grid.mark_boundaries();
        root.grid.mark_exit(Direction::Down, 3);
        root.grid.mark_exit(Direction::Up, 5);
        maze_gen.maze_fill(&mut root.grid);
        maze_gen.clear();

        root
    }

    pub fn subdivide(&self, maze_gen: &mut DFSMaze) -> [Self; 4] {
        let bottom = self.grid.get_horizontal_seam(0, Direction::Down);
        let h_middle = self.grid.get_horizontal_seam(HALF_GRID_SIZE, Direction::Down);
        let top = self.grid.get_horizontal_seam(GRID_SIZE - 1, Direction::Up);
        
        let left = self.grid.get_vertical_seam(0, Direction::Left);
        let v_middle = self.grid.get_vertical_seam(HALF_GRID_SIZE, Direction::Left);
        let right = self.grid.get_vertical_seam(GRID_SIZE - 1, Direction::Right);
        
        // In Morton order:
        // Southwest
        let mut sw = Self::new();
        sw.level = self.level + 1;
        sw.x = self.x << 1;
        sw.y = self.y << 1;
        sw.grid.set_boundary(
            // Right
            &v_middle[0..4],
            // Up
            &h_middle[0..4],
            // Left
            &left[0..4],
            // Down
            &bottom[0..4]
        );
        self.grid.propagate_interior(&mut sw.grid, 0..4, 0..4);

        // Southeast
        let mut se = Self::new();
        se.level = self.level + 1;
        se.x = self.x << 1 | 1;
        se.y = self.y << 1;
        se.grid.set_boundary(
            // Right
            &right[0..4],
            // Up
            &h_middle[4..8],
            // Left
            &v_middle[0..4],
            // Down
            &bottom[4..8]
        );
        self.grid.propagate_interior(&mut se.grid, 4..8, 0..4);

        // Northwest
        let mut nw = Self::new();
        nw.level = self.level + 1;
        nw.x = self.x << 1;
        nw.y = self.y << 1 | 1;
        nw.grid.set_boundary(
            // Right
            &v_middle[4..8],
            // Up
            &top[0..4],
            // Left
            &left[4..8],
            // Down
            &h_middle[0..4]
        );
        self.grid.propagate_interior(&mut nw.grid, 0..4, 4..8);

        // Northeast
        let mut ne = Self::new();
        ne.level = self.level + 1;
        ne.x = self.x << 1;
        ne.y = self.y << 1 | 1;
        ne.grid.set_boundary(
            // Right
            &right[4..8],
            // Up
            &top[4..8],
            // Left
            &v_middle[4..8],
            // Down
            &h_middle[4..8],
        );
        self.grid.propagate_interior(&mut ne.grid, 4..8, 4..8);

        let mut result = [sw, se, nw, ne];

        for i in 0..4 {
            maze_gen.maze_fill(&mut result[i].grid);
            maze_gen.clear();
        }

        result
    }

    pub fn write_glb(&self, tiles_dir: &Path) {
        let glb_path = tiles_dir.join(self.make_filename());
        println!("Generating {:?}", glb_path);

        let image_buffer = self.make_image_buffer();
        let image_length = image_buffer.len();

    }

    fn make_image_buffer(&self) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());

        image::write_buffer_with_format(
            &mut cursor,
            &self.grid.to_image_bytes(),
            GRID_SIZE as u32, 
            GRID_SIZE as u32,
            image::ColorType::Rgb8,
            image::ImageOutputFormat::Png
        ).expect("could not serialize image");

        cursor.into_inner()
    }

    fn make_filename(&self) -> String {
        format!("{}.{}.{}.glb", self.level, self.x, self.y)
    }

    fn make_matrix(&self) -> [f64; 16] {
        // level 0: 1
        // level 2: 1/2
        // level 3: 1/4
        let s = (0.5f64).powi(self.level as i32);

        let tile_width = 2.0 / s;
        let dx = (self.x as f64) * tile_width;
        // the +y direction is really -z in glTF's coordinate system
        let dz = -(self.y as f64) * tile_width;

        // TODO: compute this given the level
        let offset_x = 0.0;
        let offset_z = 0.0;

        let tx = offset_x + dx;
        let tz = offset_z + dz;

        [
            s, 0.0, 0.0, 0.0,
            0.0, s, 0.0, 0.0,
            0.0, 0.0, s, 0.0,
            tx, 0.0, tz, 0.0
        ]
    }

    fn make_gltf_json(&self, image_byte_length: usize) -> serde_json::Value {
        json!({
            "asset": {
                "version": "2.0",
                "generator": "Nested mazes generator from https://github.com/ptrgags/nested-mazes",
                "copyright": format!("© {} Peter Gagliardi", Utc::now().year())
            },
            "extensionsUsed": ["EXT_mesh_features"],
            "scene": 0,
            "scenes": [
                {
                    "name": "Scene",
                    "nodes": [
                        0
                    ]
                }
            ],
            "nodes": [
                {
                    "mesh": 0,
                    "name": "Maze Quad",
                    "matrix": self.make_matrix()
                }
            ],
            "meshes": [
                {
                    "name": "Maze Quad",
                    "primitives": [
                        {
                            "attributes": {
                                "POSITION": 0,
                                "TEXCOORD_0": 1,
                                "NORMAL": 2
                            },
                            "indices": 3,
                            "extensions": {
                                "featureIds": [
                                    {
                                        "featureCount": 16,
                                        "label": "connections",
                                        "texture": {
                                            "index": 0,
                                            "texCoord": 0,
                                            "channels": [0]
                                        }
                                    },
                                    {
                                        "featureCount": 16,
                                        "label": "solutions",
                                        "texture": {
                                            "index": 0,
                                            "texCoord": 0,
                                            "channels": [1]
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            ],
            "textures": [
                {
                    "sampler": 0,
                    "source": 0
                }
            ],
            "samplers": [
                {
                    "magFilter": 9728,
                    "minFilter": 9728
                }
            ],
            "images": [
                {
                    "name": "Feature ID Texture",
                    "bufferView": 4,
                    "mimeType": "image/png"
                }
            ],
            "accessors": [
                {
                    "name": "Position",
                    "bufferView": 0,
                    "type": "VEC3",
                    "componentType": 5126,
                    "count": 4,
                    "max": [1, 0, 1],
                    "min": [-1, 0, -1]
                },
                {
                    "name": "UVs",
                    "bufferView": 1,
                    "type": "VEC2",
                    "componentType": 5126,
                    "count": 4,
                },
                {
                    "name": "Normals",
                    "bufferView": 2,
                    "type": "VEC3",
                    "componentType": 5126,
                    "count": 4,
                },
                {
                    "name": "Indices",
                    "bufferView": 3,
                    "type": "SCALAR",
                    "componentType": 5123,
                    "count": 6
                }
            ],
            "bufferViews": [
                {
                    "name": "Position",
                    "buffer": 1,
                    "byteOffset": 0,
                    "byteLength": 48
                },
                {
                    "name": "UVs",
                    "buffer": 1,
                    "byteOffset": 48,
                    "byteLength": 32
                },
                {
                    "name": "Normals",
                    "buffer": 1,
                    "byteOffset": 48 + 32,
                    "byteLength": 48
                },
                {
                    "name": "Indices",
                    "buffer": 1,
                    "byteOffset": 48 + 32 + 48,
                    "byteLength": 6
                },
                {
                    "name": "Feature ID Texture",
                    "buffer": 0,
                    "byteOffset": 0,
                    "byteLength": image_byte_length
                }
            ],
            "buffers": [
                {
                    "name": "Binary Chunk",
                    "byteLength": image_byte_length
                },
                {
                    "name": "Shared Geometry",
                    "byteLength": 0, // TODO
                    "uri": "geometry.bin"
                },
            ]
        })
    }
}