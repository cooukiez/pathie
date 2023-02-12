use std::time::Duration;

use ash::vk;
use cgmath::{Vector3, Vector2};

use crate::{octree::{Octree, ROOT_SPAN}, service::vector_two_boundary};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,
    pub resolution: Vector2<f32>,

    pub mouse_pos: Vector2<f32>,

    pub root_span: f32,

    pub pos: Vector3<f32>,
}

// Simple Data storage
impl Uniform {
    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.resolution = Vector2::new(resolution.width as f32, resolution.height as f32, );
    }
    
    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, ) {
        self.pos += velocity;
    }

    pub fn move_mouse(&mut self, mouse_velocity: Vector2<f32>, ) {
        self.mouse_pos += mouse_velocity;
        vector_two_boundary(Vector2::new(0.0, 0.0, ), self.resolution, &mut self.mouse_pos, )
    }

    pub fn update_uniform(&mut self, cur_time: Duration, octree: &mut Octree, ) {
        self.time = cur_time.as_millis() as u32;
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            time: 0,
            resolution: Vector2::new(0.0, 0.0),

            mouse_pos: Vector2::new(0.0, 0.0),

            root_span: ROOT_SPAN,

            pos: Vector3::new(5.0, 5.0, 5.0, )
        }
    }
}