use cgmath::{Vector3, Array};

pub fn pos_to_index(pos: Vector3<f32>, side_len: i32, ) -> u32 {
    Vector3 {
        x: ((pos.x as i32) % side_len),
            y: ((pos.x as i32) % side_len) * side_len * side_len,
                z: ((pos.x as i32) % side_len) * side_len
    }.sum() as u32
}

pub fn index_to_pos(index: u32, side_len: u32, ) -> Vector3<f32> { 
    Vector3 {
        x: ((index % (side_len * side_len)) % side_len) as f32,
            y:(index / (side_len * side_len)) as f32,
                z: ((index % (side_len * side_len)) / side_len) as f32
    }
}