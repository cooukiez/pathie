use cgmath::{Vector3, Array, Vector2};

// Position / Mask Utility
pub fn pos_to_index(pos: Vector3<f32>, side_len: i32, ) -> usize {
    Vector3 {
        x: ((pos.x as i32) % side_len),
            y: ((pos.y as i32) % side_len) * side_len * side_len,
                z: ((pos.z as i32) % side_len) * side_len
    }.sum() as usize
}

pub fn index_to_pos(index: u32, side_len: u32, ) -> Vector3<f32> { 
    Vector3 {
        x: ((index % (side_len * side_len)) % side_len) as f32,
            y:(index / (side_len * side_len)) as f32,
                z: ((index % (side_len * side_len)) / side_len) as f32
    }
}

pub fn add_dir_to_mask(mask: Vector3<f32>, dir_mask: Vector3<f32>, ) -> Vector3<f32> {
    Vector3 {
        x: (mask.x - dir_mask.x).abs(),
                y: (mask.y - dir_mask.y).abs(),
                    z: (mask.z - dir_mask.z).abs()
    }
}

// VectorThree Utility
pub fn step_vec_three(edge: Vector3<f32>, input: Vector3<f32>, ) -> Vector3<u32> {
    Vector3 {
        x: (edge.x < input.x) as u32,
                y: (edge.y < input.y) as u32,
                    z: (edge.z < input.z) as u32
    }
}

pub fn floor_vec_three(vec: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: vec.x.floor(),
                y: vec.y.floor(),
                    z: vec.z.floor()
    }
}



pub fn sign_vec_three(vec: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: vec.x.signum(),
                y: vec.y.signum(),
                    z: vec.z.signum()
    }
}

// VectorTwo Utility

// Set Vector between boundary
pub fn vector_two_boundary(min: Vector2<f32>, max: Vector2<f32>, vec: &mut Vector2<f32>, ) {
    // Check Min
    if vec.x < min.x { vec.x = min.x; }
        if vec.y < min.y { vec.y = min.y; }

    // Check Max
    if vec.x > max.x { vec.x = max.x; }
        if vec.y > max.y { vec.y = max.y; }
}