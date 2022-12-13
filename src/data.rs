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
    pub fn pos_to_index(pos: Vector3<u32>, side_length: u32, chunk_size: u32, ) -> usize { (pos.x + (pos.y * side_length * side_length) + (pos.z * side_length) + (chunk_size / 2)) as usize }

    pub fn get_voxel_at_pos(pos: Vector3<f32>, voxel_data: &Vec<VoxelChunk>, ) -> i32 {
        let pos_as_int = Vector3::new(pos.x as u32, pos.y as u32, pos.z as u32);
        let global_index = WorldData::pos_to_index(pos_as_int, (CHUNK_SIDE_LEN * CHUNK_GROUP_SIDE_LEN) as u32, (CHUNK_SIZE * CHUNK_GROUP_SIZE) as u32);
        log::info!("Index [ {} ]", global_index);

        let chunk_group_lev_index = round::floor(global_index / CHUNK_SIZE);
        let chunk_lev_index = global_index - (chunk_group_lev_index * CHUNK_SIZE);

        log::info!("Group [ {} ]", chunk_group_lev_index);
        log::info!("Chunk [ {} ]", chunk_lev_index);

        // voxel_data[chunk_group_lev_index].voxel_data[chunk_lev_index]
        global_index as i32
    }

    pub fn collect() -> WorldData {
        let mut voxel_data: Vec<VoxelChunk> = vec![];
        for _ in 0 .. 27 { voxel_data.push(VoxelChunk { voxel_data: [0; CHUNK_SIZE] }); }
        log::info!("Len [ {} ]", voxel_data.len());
        log::info!("VoxAtPos ( -512, -512, -512 ) [ {} ]", WorldData::get_voxel_at_pos(Vector3::new(-512.0, -512.0, -512.0), &voxel_data));
        WorldData { voxel_data }
    }

    
}