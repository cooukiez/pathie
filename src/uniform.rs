use std::time::Duration;

use ash::vk;
use cgmath::{Vector3, Vector2, Vector4};

use crate::{octree::{Octree, Traverse}, service::Vector};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub traverse: Traverse,
    pub pos: Vector4<f32>,
    pub mouse_pos: Vector2<f32>,
    pub resolution: Vector2<f32>,
    pub time: u32,
}

// Simple Data storage
impl Uniform {
    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.resolution = Vector2::new(resolution.width as f32, resolution.height as f32, );
    }
    
    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, ) {
        self.pos += velocity.extend(0.0);
    }

    pub fn move_mouse(&mut self, mouse_velocity: Vector2<f32>, ) {
        self.mouse_pos += mouse_velocity;
        self.mouse_pos = self.mouse_pos.boundary(Vector2::from([0.0; 2]), self.resolution);
    }

    pub fn update_uniform(&mut self, cur_time: Duration, octree: &mut Octree, ) {
        self.time = cur_time.as_millis() as u32;
        self.traverse = octree.node_at_pos(self.pos);
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            pos: Vector4::new(5.0, 5.0, 5.0, 0.0, ),
            mouse_pos: Vector2::default(),
            resolution: Vector2::default(),
            time: 0,
            traverse: Traverse::default()
        }
    }
}