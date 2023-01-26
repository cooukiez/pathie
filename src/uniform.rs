use std::time::Duration;

use ash::vk;
use cgmath::{Vector3, Vector2, Vector4};

use crate::octree::Octree;

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VecFour { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VecThree { pub x: f32, pub y: f32, pub z: f32 }

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct VecTwo { pub x: f32, pub y: f32 }


#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,
    pub resolution: VecTwo,

    pub raw_field_of_view: f32,
    pub max_ray_length: u32,
    pub max_distance: f32,

    pub rot: VecTwo,

    pub root_span: f32,
    pub root_center: VecThree,
    pub max_recursion: u32,

    pub node_at_pos: u32,
    pub node_at_pos_span: f32,

    pub pos: VecThree,
}

// DataTransport Type
impl VecFour {
    pub fn new(x: f32, y: f32, z: f32, w: f32, ) -> VecFour { VecFour { x, y, z, w } }
    pub fn from_vec(vec: Vector4<f32>) -> VecFour { VecFour::new(vec.x, vec.y, vec.z, vec.w, ) }
    pub fn from_float(in_float: f32) -> VecFour { VecFour::new(in_float, in_float, in_float, in_float, ) }
    pub fn to_vec(&self) -> Vector4<f32> { Vector4::new(self.x, self.y, self.z, self.w, ) }
}

// DataTransport Type
impl VecThree {
    pub fn new(x: f32, y: f32, z: f32, ) -> VecThree { VecThree { x, y, z } }
    pub fn from_vec(vec: Vector3<f32>) -> VecThree { VecThree::new(vec.x, vec.y, vec.z, ) }
    pub fn from_float(in_float: f32) -> VecThree { VecThree::new(in_float, in_float, in_float, ) }
    pub fn to_vec(&self) -> Vector3<f32> { Vector3::new(self.x, self.y, self.z, ) }
}

// DataTransport Type
impl VecTwo {
    pub fn new(x: f32, y: f32, ) -> VecTwo { VecTwo { x, y } }
    pub fn from_vec(vec: Vector2<f32>) -> VecTwo { VecTwo::new(vec.x, vec.y, ) }
    pub fn from_float(in_float: f32) -> VecTwo { VecTwo::new(in_float, in_float, ) }
    pub fn to_vec(&self) -> Vector2<f32> { Vector2::new(self.x, self.y, ) }
}

// Simple Data storage
impl Uniform {
    pub fn empty() -> Uniform {
        Uniform {
            time: 0,
            resolution: VecTwo::new(0.0, 0.0),
            
            raw_field_of_view: 60.0,
            max_ray_length: 100,
            max_distance: 30.0,

            rot: VecTwo::new(0.0, 0.0),

            root_span: 64.0,
            root_center: VecThree::from_float(0.0),
            max_recursion: 100,

            node_at_pos: 0,
            node_at_pos_span: 64.0,

            pos: VecThree::from_float(- 20.0),
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

    pub fn update_uniform(&mut self, cur_time: Duration, octree: &mut Octree, ) {
        self.time = cur_time.as_millis() as u32;
        (self.node_at_pos, self.node_at_pos_span, ) = octree.node_at_pos(self.pos.to_vec());
    }
}