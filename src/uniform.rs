use std::time::Duration;

use ash::vk;
use nalgebra_glm::{look_at, normalize, perspective, translation, Mat4, Vec2, Vec3, Vec4};

use crate::vector::{Num, Vector};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub view_proj: Mat4,

    pub pos: Vec4,
    pub velocity: Vec4,

    pub cam_pos: Vec4,
    pub cam_front: Vec4,
    pub cam_up: Vec4,
    pub look_dir: Vec4,

    pub res: Vec2,

    // x = Yaw | y = Pitch
    pub mouse_delta: Vec2,
    // Mouse pos prepared for conversion into rotation
    pub mouse_rot: Vec2,

    pub root_span: f32,
    pub time: u32,

    pub padding: [u32; 2],
}

// Simple Data storage
impl Uniform {
    pub fn new(root_span: f32) -> Self {
        Self {
            pos: Vec4::new(0.0, 0.0, -10.0, 0.0),
            cam_up: Vec4::new(0.0, 1.0, 0.0, 0.0),
            root_span,

            ..Default::default()
        }
    }

    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.res = Vec2::new(resolution.width as f32, resolution.height as f32);
    }

    pub fn apply_velocity(&mut self) {
        self.pos += Vec4::new(self.velocity.x, self.velocity.y, self.velocity.z, 0.0);
        self.cam_pos += self.velocity;
        self.velocity = Vec4::default();
    }

    pub fn move_mouse(&mut self, mouse_delta: Vec2) {
        // Update mouse pos
        self.mouse_delta = mouse_delta;

        self.mouse_rot += self.mouse_delta * 0.05;
        self.mouse_rot.y = self.mouse_rot.y.boundary(-89.0, 89.0);

        // Update cam
        let yaw = self.mouse_rot.x.to_radians();
        let pitch = self.mouse_rot.y.to_radians();

        self.look_dir = Vec4::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
            0.0,
        );
    }

    pub fn update_uniform(&mut self, cur_time: Duration) {
        self.time = cur_time.as_millis() as u32;

        let view = translation(&Vec3::ftv(0.0));

        let projection = perspective(
            (self.res.x / self.res.y) as f32,
            45f32.to_radians(),
            0.1,
            100.0,
        );

        self.cam_front = self.cam_pos + normalize(&self.look_dir);

        let look_at = look_at(
            &self.cam_pos.xyz(),
            &self.cam_front.xyz(),
            &self.cam_up.xyz(),
        );

        self.view_proj = view * projection * look_at;
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            view_proj: Default::default(),
            pos: Default::default(),
            velocity: Default::default(),
            cam_pos: Default::default(),
            cam_front: Default::default(),
            cam_up: Default::default(),
            look_dir: Default::default(),
            res: Default::default(),
            mouse_delta: Default::default(),
            mouse_rot: Default::default(),
            root_span: Default::default(),
            time: Default::default(),
            padding: Default::default(),
        }
    }
}
