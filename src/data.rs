use cgmath::{Vector3};

pub struct WorldData {
    pub basic_voxel_data: Vec<BasicVoxel>,
    pub uniform_buffer: Uniform,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct BasicVoxel {
    pub pos: Vector3<u32>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub head_rot: Vector3<u32>,
    pub player_pos: Vector3<u32>,
}

impl WorldData {
    pub fn collect() -> WorldData {
        WorldData { basic_voxel_data: vec![BasicVoxel { pos: Vector3::new(5, 2, 3, ) }], uniform_buffer: Uniform { head_rot: Vector3::new(0, 0, 0, ), player_pos: Vector3::new(0, 0, 0, ) } }
    }
}