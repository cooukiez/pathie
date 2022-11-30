use cgmath::{Vector3, Vector2};

pub struct WorldData {
    pub basic_data: Vec<VoxelChunk>,
    pub uniform_buffer: Uniform,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VoxelChunk { 
    pub voxel_data: [u32; 256],
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub test: u32,
}

impl WorldData {
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] {
        vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len()))
    }

    pub fn collect() -> WorldData {
        // let uniform_buffer = Uniform { head_rot: Vector2::new(0, 0, ), player_pos: Vector3::new(0, 0, 0, ) };
        let uniform_buffer = Uniform { test: 1 };
        let mut basic_voxel_input: Vec<u32> = vec![];
        for index in 0 .. 256 { basic_voxel_input.push(index); }
        let voxel_chunk = VoxelChunk { voxel_data: WorldData::vec_to_array(basic_voxel_input) };
        log::info!("{:?}", voxel_chunk.voxel_data);
        WorldData { basic_data: vec![voxel_chunk], uniform_buffer }
    }
}