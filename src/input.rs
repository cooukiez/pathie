use winit::{event::{VirtualKeyCode, ElementState}, window::Fullscreen};

use crate::Render;

#[derive(PartialEq, Clone, Copy)]
pub enum Action {
    NONE,

    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,

    JUMP,

    FULLSCREEN,
    ESCAPE,
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

        binding_list[VirtualKeyCode::F as usize] = Action::FULLSCREEN;
        binding_list[VirtualKeyCode::Escape as usize] = Action::ESCAPE;

        Input { binding_list }
    }

    pub fn handle_key_input(&self, keycode: &VirtualKeyCode, state: &ElementState, render: &Render, ) {
        if state == &ElementState::Pressed {
            match self.binding_list[* keycode as usize] {
                Action::FORWARD => unsafe { UNIFORM.z += MOVE_INC_FRONT },
                Action::BACKWARD => unsafe { UNIFORM.z -= MOVE_INC_FRONT },
                Action::LEFT => unsafe { UNIFORM.x -= MOVE_INC_SIDE },
                Action::RIGHT => unsafe { UNIFORM.z += MOVE_INC_SIDE },
 
                Action::JUMP => unsafe { UNIFORM.y += JUMP_INC },
                
                Action::FULLSCREEN => vulkan.window.set_fullscreen(Some(Fullscreen::Exclusive(render..monitor.video_modes().next().expect("ERR_NO_MONITOR_MODE").clone()))),
                Action::ESCAPE => vulkan.window.set_fullscreen(None), _ => (),
            }
        }
    }
}
