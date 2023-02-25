use std::time::Duration;

use ash::vk;
use cgmath::{Vector3, Vector2, Vector4};

use crate::{octree::{Octree, MAX_DEPTH}, service::Vector};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub mask_in_parent: [Vector4<f32>; MAX_DEPTH],

    pub pos: Vector4<f32>,

    pub local_pos: Vector4<f32>,
    pub pos_on_edge: Vector4<f32>,
    
    pub resolution: Vector2<f32>,
    pub mouse_pos: Vector2<f32>,

    pub index: u32,
    pub parent: u32,

    pub span: f32,
    pub depth: i32,

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
        let traverse = octree.get_traverse(self.pos.truncate());

        traverse.mask_in_parent
            .iter()
            .enumerate()
            .for_each(| (index, &mask, ) | self.mask_in_parent[index] = mask.extend(0.0));

        self.local_pos = traverse.local_pos.extend(0.0);
        self.pos_on_edge = traverse.pos_on_edge.extend(0.0);

        self.index = traverse.index;
        self.parent = traverse.parent;

        self.span = traverse.span;
        self.depth = traverse.depth
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            mask_in_parent: [Vector4::default(); MAX_DEPTH],

            pos: Vector4::default(),

            local_pos: Vector4::default(),
            pos_on_edge: Vector4::default(),

            resolution: Vector2::default(),
            mouse_pos: Vector2::default(),
            
            index: 0,
            parent: 0,

            span: 0.0,
            depth: 0,

            time: 0
        }
    }
}