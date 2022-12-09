use cgmath::{Vector3, Vector2};

use crate::{CHUNK_SIZE};

pub struct WorldData {
    pub voxel_data: Vec<VoxelChunk>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VoxelChunk {
    pub voxel_data: [i32; CHUNK_SIZE],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,

    pub head_rot: Vector2<i32>,
    pub player_pos: Vector3<i32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct GraphicPref {
    pub field_of_view: f32,
    pub max_ray_length: u32,

    pub chunk_side_len: u32,
    pub chunk_size: u32,

    pub chunk_group_side_len: u32,
    pub chunk_group_size: u32,
}

impl WorldData {
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] { vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len())) }

    pub fn collect() -> WorldData {
        WorldData { voxel_data: vec![
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [0; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] },

            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [0; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] },

            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [0; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [1; CHUNK_SIZE] }, 
            VoxelChunk { voxel_data: [-1; CHUNK_SIZE] },
        ] }
    }
}