use cgmath::Vector3;

use crate::{CHUNK_SIZE, CHUNK_SIDE_LEN, CHUNK_GROUP_SIDE_LEN, CHUNK_GROUP_SIZE};

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

    pub field_of_view: f32,
    pub max_ray_length: u32,

    pub rot_horizontal: f32,
    pub rot_vertical: f32,
    
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct GraphicPref {
    pub empty: u32,
}

impl WorldData {
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] { vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len())) }
    pub fn pos_to_index(pos: Vector3<i32>, side_length: i32, chunk_size: i32, ) -> i32 { pos.x + (pos.y * side_length) + (pos.z * side_length * side_length) + (chunk_size / 2) }

    pub fn get_voxel_at_pos(pos: Vector3<f32>, voxel_data: &Vec<VoxelChunk>, ) -> i32 {
        let pos_as_int = Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32);

        let global_index = WorldData::pos_to_index(pos_as_int, (CHUNK_SIDE_LEN * CHUNK_GROUP_SIDE_LEN) as i32, (CHUNK_SIZE * CHUNK_GROUP_SIZE) as i32);
        
        let group_index = global_index / CHUNK_SIZE as i32;
        let chunk_index = global_index - (group_index * CHUNK_SIZE as i32);

        log::info!("Index [ {} ] | Group [ {} ] | Chunk [ {} ]", global_index, group_index, chunk_index);

        voxel_data[group_index as usize].voxel_data[chunk_index as usize]
    }

    pub fn collect() -> WorldData {
        let mut voxel_data: Vec<VoxelChunk> = vec![];
        for _ in 0 .. 27 { voxel_data.push(VoxelChunk { voxel_data: [0; CHUNK_SIZE] }); }
        voxel_data[7].voxel_data[448] = 1;
        voxel_data[6].voxel_data[384] = 1; 
        log::info!("Len [ {} ]", voxel_data.len());
        log::info!("VoxAtPos [ {} ]", WorldData::get_voxel_at_pos(Vector3::new(0.0, 0.0, -6.0), &voxel_data));
        WorldData { voxel_data }
    }

    
}