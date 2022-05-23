const COUNT: usize = 4;
const POSITIONS_SIZE: usize = COUNT * 3;
const POSITIONS: [f32; POSITIONS_SIZE] = [
    // southwest
    -1.0, 0.0, 1.0,
    // southeast
    1.0, 0.0, 1.0,
    // northeast
    1.0, 0.0, -1.0,
    // northwest
    -1.0, 0.0, -1.0
];

const UVS_SIZE: usize = COUNT * 2;
const UVS: [f32; UVS_SIZE] = [
    // southwest
    0.0, 0.0,
    // southeast
    1.0, 0.0,
    // northeast
    1.0, 1.0,
    // northwest
    0.0, 1.0
];

// All the normals point to +y
const NORMALS_SIZE: usize = COUNT * 3;
const NORMALS: [f32; NORMALS_SIZE] = [
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
];

const INDICES_SIZE: usize = 6;
const INDICES: [u8; INDICES_SIZE] = [
    // southeast triangle
    0, 1, 2,
    // northwest triangle
    2, 3, 0
];

pub fn make_buffer() -> Vec<u8> {
    let mut result = Vec::new();

    for component in POSITIONS {
        result.extend_from_slice(&component.to_le_bytes());
    }

    for component in UVS {
        result.extend_from_slice(&component.to_le_bytes());
    }
    
    for component in NORMALS {
        result.extend_from_slice(&component.to_le_bytes());
    }

    // There are 6 indices, so we add 2 bytes of padding at the end.
    for component in INDICES {
        result.push(component);
    }
    result.push(0x00);
    result.push(0x00);

    result
}