use std::fs::File;
use std::io::Cursor;
use std::io::prelude::*;
use std::path::Path;

use chrono::{Datelike, Utc};
use serde_json::{json, to_string};

use crate::direction::Direction;
use crate::dfs::DFSMaze;
use crate::geometry::get_buffer_size;
use crate::grid::Grid;
use crate::grid_coords::GRID_SIZE;

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
        ne.x = self.x << 1 | 1;
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

        let image_buffer = self.make_image_buffer();
        let image_length = image_buffer.len() as u32;
        let binary_padding_length = Self::get_padding_length(image_length);
        let binary_chunk_length = image_length + binary_padding_length;
        let binary_padding = Self::make_padding(binary_padding_length, b'\0');
        assert!(
            binary_chunk_length % 4 == 0,
            "binary chunk not a multiple of 4 bytes"
        );

        let gltf_json = self.make_gltf_json(image_length);
        let json_string = to_string(&gltf_json)
            .expect("could not serialize glTF JSON");
        let json_bytes = json_string.as_bytes();
        let json_length = json_bytes.len() as u32;
        let json_padding_length = Self::get_padding_length(json_length);
        let json_padding = Self::make_padding(json_padding_length, b' ');
        let json_chunk_length = json_length + json_padding_length;
        assert!(
            json_chunk_length % 4 == 0,
            "json chunk not a multiple of 4 bytes"
        );
        
        const HEADER_LENGTH: u32 = 12;
        const CHUNK_HEADER_LENGTH: u32 = 8;
        let total_length =
            HEADER_LENGTH +
            CHUNK_HEADER_LENGTH +
            json_chunk_length +
            CHUNK_HEADER_LENGTH +
            binary_chunk_length;

        const GLTF_VERSION: u32 = 2;

        let mut file = File::create(glb_path).expect("Could not create file");
        // GLB header
        file.write_all(b"glTF").expect("Could not write magic");
        file.write_all(&GLTF_VERSION.to_le_bytes())
            .expect("Could not write version");
        file.write_all(&total_length.to_le_bytes())
            .expect("Could not write glTF length");
        
        // JSON chunk
        file.write_all(&json_chunk_length.to_le_bytes())
            .expect("Could not write JSON chunk length");
        file.write_all(b"JSON").expect("Could not write JSON chunk magic");
        file.write_all(&json_bytes).expect("Could not write JSON data");
        file.write_all(&json_padding).expect("Could not write JSON padding");

        // Binary chunk
        file.write_all(&binary_chunk_length.to_le_bytes())
            .expect("Could not write BIN chunk length");
        file.write_all(b"BIN\0").expect("Could not write BIN chunk magic");
        file.write_all(&image_buffer).expect("Could not write binary buffer");
        file.write_all(&binary_padding)
            .expect("Could not write binary padding");

    }

    fn get_padding_length(length: u32) -> u32 {
        const GLB_ALIGNMENT: u32 = 4;
        // modulo but go from [1, GLB_ALIGNMENT] instead of 
        // [0, GLB_ALIGNMENT - 1]
        let leftover = (length - 1) % GLB_ALIGNMENT + 1;
        GLB_ALIGNMENT - leftover
    }

    fn make_padding(length: u32, padding_char: u8) -> Vec<u8> {
        (0..length).map(|_| padding_char).collect()
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
        // 2^level = 1, 2, 4, 8, ...
        let power_of_two = (1 << self.level) as f64;
        // 1 / 2^level = 1, 1/2, 1/4, ...
        let inv_power_of_two = 1.0 / power_of_two;

        // Each tile is half as small as its parent in each dimension
        let scale = inv_power_of_two;

        // The root tile goes from -1 to 1 in the x and z directions so 
        // it has size 2. Each level is half the size of the previous one
        let tile_width = 2.0 * inv_power_of_two;

        // The first level offset is (0, 0)
        // The second is (-1/2, 0, 1/2)
        // The third is (-3/4, 0, 3/4)
        // ...
        // In general, (-(2^level - 1) / 2^level, 0, (2^level - 1) / 2^level)
        // this distance is the term (2^level - 1) / 2^level
        let offset_distance = (power_of_two - 1.0) * inv_power_of_two;

        // Offsets are in glTF coordinates so the x coordinate increases in
        // the +x direction and the y coordinate increases in the -z direction
        let offset_x = -offset_distance;
        let offset_z = offset_distance;
        
        let dx = (self.x as f64) * tile_width;
        let dz = -(self.y as f64) * tile_width;

        let tx = offset_x + dx;
        let ty = 0.0;
        let tz = offset_z + dz;

        [
            scale, 0.0, 0.0, 0.0,
            0.0, scale, 0.0, 0.0,
            0.0, 0.0, scale, 0.0,
            tx, ty, tz, 1.0
        ]
    }

    fn make_gltf_json(&self, image_byte_length: u32) -> serde_json::Value {
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
                                "EXT_mesh_features": {
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
                    "componentType": 5126, // float
                    "count": 4,
                    "max": [1, 0, 1],
                    "min": [-1, 0, -1]
                },
                {
                    "name": "UVs",
                    "bufferView": 1,
                    "type": "VEC2",
                    "componentType": 5126, // float
                    "count": 4,
                },
                {
                    "name": "Normals",
                    "bufferView": 2,
                    "type": "VEC3",
                    "componentType": 5126, // float
                    "count": 4,
                },
                {
                    "name": "Indices",
                    "bufferView": 3,
                    "type": "SCALAR",
                    "componentType": 5121, // unsigned byte
                    "count": 6
                }
            ],
            "bufferViews": [
                {
                    "name": "Position",
                    "buffer": 1,
                    "byteOffset": 0,
                    "byteLength": 48,
                    "target": 34962 // array buffer
                },
                {
                    "name": "UVs",
                    "buffer": 1,
                    "byteOffset": 48,
                    "byteLength": 32,
                    "target": 34962 // array buffer
                },
                {
                    "name": "Normals",
                    "buffer": 1,
                    "byteOffset": 48 + 32,
                    "byteLength": 48,
                    "target": 34962 // array buffer
                },
                {
                    "name": "Indices",
                    "buffer": 1,
                    "byteOffset": 48 + 32 + 48,
                    "byteLength": 6,
                    "target": 34963 // element array buffer
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
                    "byteLength": get_buffer_size(),
                    "uri": "geometry.bin"
                },
            ]
        })
    }
}