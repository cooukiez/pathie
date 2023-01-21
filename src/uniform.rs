use std::time::Duration;

use ash::vk;
use cgmath::{Vector3, Vector2};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VecThree { x: f32, y: f32, z: f32, }

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VecTwo { x: f32, y: f32 }


#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,
    pub resolution: VecTwo,

    pub raw_field_of_view: f32,
    pub max_ray_length: u32,

    pub rot: VecTwo,

    pub octree_root_index: u32,
    
    pub node_at_pos: u32,
    pub pos: VecThree,
}

// DataTransport Type
impl VecThree {
    pub fn new(x: f32, y: f32, z: f32, ) -> VecThree { VecThree { x, y, z } }
    pub fn from_vec(vec: Vector3<f32>) -> VecThree { VecThree::new(vec.x, vec.y, vec.z, ) }
    pub fn to_vec(&self) -> Vector3<f32> { Vector3::new(self.x, self.y, self.z, ) }
}

// DataTransport Type
impl VecTwo {
    pub fn new(x: f32, y: f32, ) -> VecTwo { VecTwo { x, y } }
    pub fn from_vec(vec: Vector2<f32>) -> VecTwo { VecTwo::new(vec.x, vec.y, ) }
    pub fn to_vec(&self) -> Vector2<f32> { Vector2::new(self.x, self.y, ) }
}

// Simple Data storage
impl Uniform {
    pub fn empty() -> Uniform {
        Uniform {
            time: 0,
            resolution: VecTwo::new(0.0, 0.0),
            
            raw_field_of_view: 60.0,
            max_ray_length: 300,

            rot: VecTwo::new(0.0, 0.0),

            octree_root_index: 0,

            node_at_pos: 0,
            pos: VecThree::new(0.0, 0.0, 0.0),
        }
    }

    pub fn apply_resolution(&mut self, in_res: vk::Extent2D) {
        self.resolution = VecTwo::new(in_res.width as f32, in_res.height as f32);
    }
    
    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, ) {
        self.pos = VecThree::from_vec(velocity + self.pos.to_vec())
    }

    pub fn apply_rotation(&mut self, rotation: Vector2<f32>, ) {
        self.rot = VecTwo::from_vec(rotation + self.rot.to_vec())
    }

    pub fn update_uniform(&mut self, cur_time: Duration, ) {
        self.time = cur_time.as_millis() as u32;
    }
}