use std::time::Duration;

use ash::vk;
use cgmath::{Vector2, Vector3, Vector4};

use crate::{
    tree::{octree::Octree, trace::PosInfo},
    vector::Num,
    vector::Vector,
};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub view_proj: nalgebra_glm::Mat4,
    pub pos: Vector4<f32>,
    pub look_dir: nalgebra_glm::Vec3,
    pub cam_front: nalgebra_glm::Vec3,
    pub cam_pos: nalgebra_glm::Vec3,
    pub cam_up: nalgebra_glm::Vec3,

    pub res: Vector2<f32>,

    // x = Yaw | y = Pitch
    pub mouse_delta: Vector2<f32>,
    pub mouse_pos: Vector2<f32>,

    pub mouse_rot: Vector2<f32>,

    pub root_span: f32,
    pub time: u32,

    pub padding: [u32; 2],

    pub pos_info: PosInfo,
}

// Simple Data storage
impl Uniform {
    pub fn new(root_span: f32) -> Self {
        Self {
            pos: Vector4::new(0.0, 0.0, -10.0, 0.0),
            cam_up: nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
            root_span,

            ..Default::default()
        }
    }

    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.res = Vector2::new(resolution.width as f32, resolution.height as f32);
    }

    pub fn apply_velocity(&mut self, vc: nalgebra_glm::Vec3, octree: &Octree) {
        self.pos += Vector4::new(vc.x, vc.y, vc.z, 0.0);
        self.pos_info = octree.node_at_pos(self.pos.truncate());
        self.cam_pos += vc;
    }

    pub fn move_mouse(&mut self, mouse_delta: Vector2<f32>) {
        // Update mouse pos
        self.mouse_delta = mouse_delta;

        self.mouse_rot += self.mouse_delta * 0.1;
        self.mouse_rot.y = self.mouse_rot.y.boundary(-89.0, 89.0);

        // Update cam
        let yaw = self.mouse_rot.x.to_radians();
        let pitch = self.mouse_rot.y.to_radians();

        log::info!("{}", self.mouse_rot.x);

        self.look_dir = nalgebra_glm::Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
    }

    pub fn update_uniform(&mut self, cur_time: Duration) {
        self.time = cur_time.as_millis() as u32;

        let view = nalgebra_glm::translation(&nalgebra_glm::make_vec3(&[0.0; 3]));

        self.cam_front = self.cam_pos + nalgebra_glm::normalize(&self.look_dir);
        let look_at = nalgebra_glm::look_at(
            &self.cam_pos,
            &self.cam_front,
            &self.cam_up,
        );

        let projection = nalgebra_glm::perspective(
            (self.res.x / self.res.y) as f32,
            45f32.to_radians(),
            0.1,
            100.0,
        );

        self.view_proj = view * projection * look_at;
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            view_proj: Default::default(),
            pos: Vector4::new(0.5, 0.5, 0.5, 0.0),
            look_dir: Default::default(),
            cam_pos: Default::default(),
            cam_front: Default::default(),
            cam_up: Default::default(),
            res: Vector2::default(),
            mouse_delta: Vector2::default(),
            mouse_pos: Vector2::default(),
            mouse_rot: Vector2::default(),
            root_span: Default::default(),
            time: Default::default(),
            padding: Default::default(),
            pos_info: Default::default(),
        }
    }
}
