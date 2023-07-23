use std::time::Duration;

use ash::vk;
use cgmath::{Vector2, Vector3, Vector4};

use crate::{
    tree::{octree::Octree, trace::PosInfo},
    vector::Vector,
};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub view_proj: nalgebra_glm::Mat4,
    pub pos: Vector4<f32>,

    pub res: Vector2<f32>,

    pub mouse_delta: Vector2<f32>,
    pub mouse_pos: Vector2<f32>,

    // x = Yaw | y = Pitch
    pub rot: Vector2<f32>,

    pub root_span: f32,
    pub time: u32,

    pub padding: [u32; 2],

    pub pos_info: PosInfo,
}

// Simple Data storage
impl Uniform {
    pub fn new(root_span: f32) -> Self {
        Self {
            root_span,
            ..Default::default()
        }
    }

    pub fn apply_resolution(&mut self, resolution: vk::Extent2D) {
        self.res = Vector2::new(resolution.width as f32, resolution.height as f32);
    }

    pub fn apply_velocity(&mut self, velocity: Vector3<f32>, octree: &Octree) {
        self.pos += velocity.extend(0.0);
        self.pos_info = octree.node_at_pos(self.pos.truncate());
    }

    pub fn move_mouse(&mut self, absolute_mouse_pos: Vector2<f32>) {
        let old_mouse_pos = self.mouse_pos.clone();

        self.mouse_pos = absolute_mouse_pos;

        self.mouse_pos = self.mouse_pos.boundary(- self.res / 2.0, self.res / 2.0);

        self.mouse_delta = self.mouse_pos - old_mouse_pos;

        self.rot -= self.mouse_delta;
    }

    pub fn get_projection(fov: f32, res: nalgebra_glm::Vec2, near: f32) -> nalgebra_glm::Mat4 {
        let fov = 1.0 / (fov / 2.0).tan();
        let aspect = res.x / res.y;
        nalgebra_glm::mat4(
            fov / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            -fov,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            near,
            0.0,
        )
    }

    pub fn update_uniform(&mut self, cur_time: Duration) {
        self.time = cur_time.as_millis() as u32;

        let view = nalgebra_glm::translation(
            &nalgebra_glm::make_vec3(&[0.1; 3]),
        );

        let rot_angle = nalgebra_glm::atan2(
            &nalgebra_glm::Vec1::new(self.mouse_delta.y),
            &nalgebra_glm::Vec1::new(self.mouse_delta.x),
        );

        let rot_x = nalgebra_glm::rotate(
            &view,
            self.mouse_delta.x,
            &nalgebra_glm::Vec3::new(1.0, 0.0, 0.0),
        );
        let rot_y = nalgebra_glm::rotate(
            &view,
            self.mouse_delta.y,
            &nalgebra_glm::Vec3::new(0.0, 1.0, 0.0),
        );

        self.view_proj = view * Self::get_projection(20.0, nalgebra_glm::Vec2::new(self.res.x, self.res.y), 20.0);

        /*
        let rotation = nalgebra_glm::rotate(&nalgebra_glm::make_mat4(&[1.0; 16]), angle, axis);
        let matrix = translation * rotation;

        let ortho_proj =
            nalgebra_glm::ortho(0.0, self.res.x, 0.0, self.res.y, 0.0, 1000.0);
        */
    }
}

impl Default for Uniform {
    fn default() -> Self {
        Self {
            view_proj: Default::default(),
            pos: Vector4::new(0.5, 0.5, 0.5, 0.0),
            res: Vector2::default(),
            mouse_delta: Vector2::default(),
            mouse_pos: Vector2::default(),
            rot: Vector2::default(),
            root_span: Default::default(),
            time: Default::default(),
            padding: Default::default(),
            pos_info: Default::default(),
        }
    }
}
