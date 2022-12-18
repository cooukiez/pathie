use cgmath::Vector3;

use crate::{CHUNK_SIZE, CHUNK_SIDE_LEN};

pub struct WorldData {
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

    pub fn get_voxel_at_pos(pos: Vector3<f32>, voxel_data: &[i32; CHUNK_SIZE], ) -> i32 {
        let pos_as_int = Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32);
        let index = WorldData::pos_to_index(pos_as_int, CHUNK_SIDE_LEN as i32, CHUNK_SIZE as i32);

        voxel_data[index as usize]
    }

    pub fn set_voxel_at_pos(pos: Vector3<f32>, voxel_data: &mut [i32; CHUNK_SIZE], value: i32, ) {
        let pos_as_int = Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32);
        let index = WorldData::pos_to_index(pos_as_int, CHUNK_SIDE_LEN as i32, CHUNK_SIZE as i32);

        voxel_data[index as usize] = value;
    }

    pub fn collect() -> WorldData {
        let mut voxel_data: [i32; CHUNK_SIZE] = [-1; CHUNK_SIZE];

        // WorldData::set_voxel_at_pos(Vector3::new(-10.0, 5.0, 10.0), &mut voxel_data, 1);
        WorldData::set_voxel_at_pos(Vector3::new(0.0, 0.0, -6.0), &mut voxel_data, 1);
        WorldData::set_voxel_at_pos(Vector3::new(0.0, 0.0, 5.0), &mut voxel_data, 1);

        WorldData { voxel_data }
    }

    
}