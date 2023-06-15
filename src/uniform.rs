use std::time::Duration;

use ash::vk;
use cgmath::{Vector2, Vector3, Vector4};

use crate::{vector::Vector, tree::{trace::PosInfo, octree::Octree}};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub pos: Vector4<f32>,

    pub resolution: Vector2<f32>,
    pub mouse_pos: Vector2<f32>,

    pub root_span: f32,
    pub time: u32,

    pub padding: [u32; 2],

    pub pos_info: PosInfo,
}

// Simple Data storage
impl Uniform {
    pub fn new(root_span: f32) -> Self {
        Self { root_span, ..Default::default() }
    }

    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.resolution = Vector2::new(resolution.width as f32, resolution.height as f32);
    }

    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, octree: &Octree) {
        self.pos += velocity.extend(0.0);
        self.pos_info = octree.node_at_pos(self.pos.truncate());
    }

    pub fn move_mouse(&mut self, mouse_velocity: Vector2<f32>) {
        self.mouse_pos += mouse_velocity;
        self.mouse_pos = self
            .mouse_pos
            .boundary(Vector2::from([0.0; 2]), self.resolution);
    }

    pub fn update_uniform(&mut self, cur_time: Duration) {
        self.time = cur_time.as_millis() as u32;
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            pos: Vector4::new(0.5, 0.5, 0.5, 0.0),

            resolution: Vector2::default(),
            mouse_pos: Vector2::default(),

            root_span: 0.0,

            time: 0,

            padding: [0; 2],

            pos_info: PosInfo::default(),
        }
    }
}
