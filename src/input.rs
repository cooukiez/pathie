use cgmath::{Vector2, Vector3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, VirtualKeyCode},
    window::Fullscreen,
};

use crate::{interface::Interface, uniform::Uniform};

const MOVEMENT_INC: f32 = 10.0;

#[derive(PartialEq, Clone, Copy)]
pub enum Action {
    NONE,

    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,

    JUMP,
    SHIFT,

    FULLSCREEN,
    ESCAPE,

    RESET,
}

pub struct Input {
    pub binding_list: [Action; 256],
}

impl Input {
    pub fn new() -> Input {
        let mut binding_list = [Action::NONE; 256];

        binding_list[VirtualKeyCode::W as usize] = Action::FORWARD;
        binding_list[VirtualKeyCode::S as usize] = Action::BACKWARD;
        binding_list[VirtualKeyCode::A as usize] = Action::LEFT;
        binding_list[VirtualKeyCode::D as usize] = Action::RIGHT;

        binding_list[VirtualKeyCode::Space as usize] = Action::JUMP;
        binding_list[VirtualKeyCode::LShift as usize] = Action::SHIFT;

        binding_list[VirtualKeyCode::F as usize] = Action::FULLSCREEN;
        binding_list[VirtualKeyCode::Escape as usize] = Action::ESCAPE;

        binding_list[VirtualKeyCode::R as usize] = Action::RESET;

        Input { binding_list }
    }

    pub fn handle_key_input(
        &self,
        keycode: &VirtualKeyCode,
        state: &ElementState,
        interface: &Interface,
        uniform: &mut Uniform,
    ) {
        if state == &ElementState::Pressed {
            match self.binding_list[*keycode as usize] {
                Action::FORWARD => uniform.apply_velocity(Vector3::new(0.0, 0.0, MOVEMENT_INC)),
                Action::BACKWARD => uniform.apply_velocity(Vector3::new(0.0, 0.0, -MOVEMENT_INC)),
                Action::LEFT => uniform.apply_velocity(Vector3::new(MOVEMENT_INC, 0.0, 0.0)),
                Action::RIGHT => uniform.apply_velocity(Vector3::new(-MOVEMENT_INC, 0.0, 0.0)),

                Action::JUMP => uniform.apply_velocity(Vector3::new(0.0, MOVEMENT_INC, 0.0)),
                Action::SHIFT => uniform.apply_velocity(Vector3::new(0.0, -MOVEMENT_INC, 0.0)),

                Action::FULLSCREEN => interface.window.set_fullscreen(Some(Fullscreen::Exclusive(
                    interface
                        .monitor
                        .video_modes()
                        .next()
                        .expect("ERR_NO_MONITOR_MODE")
                        .clone(),
                ))),
                Action::ESCAPE => interface.window.set_fullscreen(None),
                Action::RESET => interface
                    .window
                    .set_cursor_position(PhysicalPosition::new(
                        uniform.resolution.x / 2.0,
                        uniform.resolution.x / 2.0,
                    ))
                    .unwrap(),

                _ => (),
            }
        }
    }

    pub fn handle_mouse_input(&self, position: PhysicalPosition<f64>, uniform: &mut Uniform) {
        let relative_mouse_pos = Vector2::new(position.x as f32, position.y as f32);
        let absolute_mouse_pos = relative_mouse_pos - uniform.resolution / 2.0;
        uniform.move_mouse(absolute_mouse_pos);
    }
}
