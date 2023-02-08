use std::time::Duration;

use ash::vk;
use cgmath::{Vector3, Vector2};

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
    pub resolution: Vector2<f32>,

    pub raw_field_of_view: f32,
    pub max_ray_length: u32,
    pub max_distance: f32,

    pub mouse_pos: Vector2<f32>,

    pub root_span: f32,
    pub max_recursion: u32,

    pub pos: Vector3<f32>,

    pub test: Vector2<f32>,
}

// Simple Data storage
impl Uniform {
    pub fn empty() -> Uniform {
        Uniform {
            time: 0,
            resolution: Vector2::new(0.0, 0.0),
            
            raw_field_of_view: 60.0,
            max_ray_length: 4096,
            max_distance: 4096.0,

            mouse_pos: Vector2::new(0.0, 0.0),

            root_span: 4096.0,
            max_recursion: 10,

            pos: Vector3::new(255.1, 255.1, 0.1, ),

            test: Vector2::new(123.0, 123.0, ),
        }
    }

    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.resolution = Vector2::new(resolution.width as f32, resolution.height as f32, );
    }
    
    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, ) {
        self.pos += velocity;
    }

    pub fn move_mouse(&mut self, mouse_velocity: Vector2<f32>, ) {
        self.mouse_pos += mouse_velocity;
    }

    pub fn update_uniform(&mut self, cur_time: Duration, octree: &mut Octree, ) {
        self.time = cur_time.as_millis() as u32;
    }
}