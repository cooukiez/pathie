use cgmath::{Vector3, Vector2};

use crate::{FOV, MAX_RAY_LEN};

pub struct WorldData {
    pub basic_data: Vec<VoxelChunk>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VoxelChunk { 
    pub voxel_data: [u32; 256],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub field_of_view: f32,
    pub max_ray_length: u32,

    pub head_rot: Vector2<u32>,
    pub player_pos: Vector3<u32>,
}

impl WorldData {
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] {
        vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len()))
    }

    pub fn get_uniform_data() -> Uniform { Uniform { field_of_view: FOV, max_ray_length: MAX_RAY_LEN, head_rot: Vector2::new(0, 0, ), player_pos: Vector3::new(0, 0, 0, ) } }

    pub fn collect() -> WorldData {
        // Remove Later
        let mut basic_voxel_input: Vec<u32> = vec![];
        for index in 0 .. 256 { basic_voxel_input.push(index); }

        let voxel_chunk = VoxelChunk { voxel_data: WorldData::vec_to_array(basic_voxel_input) };
        WorldData { basic_data: vec![voxel_chunk] }
    }
}