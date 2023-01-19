use std::time::Duration;

use cgmath::{Vector3, Vector2};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Position { x: f32, y: f32, z: f32, }

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Rotation { x: f32, y: f32 }

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,

    pub raw_field_of_view: f32,
    pub max_ray_length: u32,

    pub rot: Rotation,

    pub octree_root_index: u32,
    
    pub node_at_pos: u32,
    pub pos: Position,
}

// DataTransport Type
impl Position {
    pub fn new(x: f32, y: f32, z: f32, ) -> Position { Position { x, y, z } }
    pub fn from_vec(vec: Vector3<f32>) -> Position { Position::new(vec.x, vec.y, vec.z, ) }
    pub fn to_vec(&self) -> Vector3<f32> { Vector3::new(self.x, self.y, self.z, ) }
}

// DataTransport Type
impl Rotation {
    pub fn new(x: f32, y: f32, ) -> Rotation { Rotation { x, y } }
    pub fn from_vec(vec: Vector2<f32>) -> Rotation { Rotation::new(vec.x, vec.y, ) }
    pub fn to_vec(&self) -> Vector2<f32> { Vector2::new(self.x, self.y, ) }
}

// Simple Data storage
impl Uniform {
    pub fn empty() -> Uniform {
        Uniform {
            time: 0,
            
            raw_field_of_view: 60.0,
            max_ray_length: 300,

            rot: Rotation::new(0.0, 0.0),

            octree_root_index: 0,

            node_at_pos: 0,
            pos: Position::new(0.0, 0.0, 0.0),
        }
    }

    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, ) {
        self.pos = Position::from_vec(velocity + self.pos.to_vec())
    }

    pub fn apply_rotation(&mut self, rotation: Vector2<f32>, ) {
        self.rot = Rotation::from_vec(rotation + self.rot.to_vec())
    }

    pub fn update_uniform(&mut self, cur_time: Duration, ) {
        self.time = cur_time.as_secs() as u32;
    }
}