use cgmath::Vector3;
use winit::{event::{VirtualKeyCode, ElementState}, window::Fullscreen};

use crate::{vulkan::Vulkan, UNIFORM};

#[derive(PartialEq)]
pub enum Action {
    X,
    Y,
    Z,

    FULLSCREEN,
    ESCAPE,
}

#[derive(PartialEq)]
pub enum BindingType {
    GENERAL,
    MOVEMENT,
}

pub struct Binding {
    pub key: VirtualKeyCode,
    pub action: Action,
    pub value: i32,
    pub binding_type: BindingType,
}

pub struct Keyboard {
    pub binding_list: Vec<Binding>,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        let forward = Binding { key: VirtualKeyCode::W, action: Action::Z, value: 1, binding_type: BindingType::MOVEMENT };
        let backward = Binding { key: VirtualKeyCode::S, action: Action::Z, value: -1, binding_type: BindingType::MOVEMENT };
        let left = Binding { key: VirtualKeyCode::A, action: Action::X, value: -1, binding_type: BindingType::MOVEMENT };
        let right = Binding { key: VirtualKeyCode::D, action: Action::X, value: 1, binding_type: BindingType::MOVEMENT };

        let fullscreen = Binding { key: VirtualKeyCode::F, action: Action::FULLSCREEN, value: 0, binding_type: BindingType::GENERAL };
        let escape = Binding { key: VirtualKeyCode::Escape, action: Action::ESCAPE, value: 0, binding_type: BindingType::GENERAL };

        let binding_list: Vec<Binding> = vec![forward, backward, left, right, fullscreen, escape];

        Keyboard { binding_list }
    }

    pub fn change_pos(action: &Action, value: f32, ) {
        match action {
            Action::X => unsafe { UNIFORM.X += value },
            Action::Y => unsafe { UNIFORM.Y += value },
            Action::Z => unsafe { UNIFORM.Z += value }, _ => (),
        }
    }

    pub fn general_operation(action: &Action, vulkan: &Vulkan, ) {
        match action {
            Action::FULLSCREEN => { vulkan.window.set_fullscreen(Some(Fullscreen::Exclusive(vulkan.monitor.video_modes().next().expect("ERR_NO_MONITOR_MODE").clone()))); },
            Action::ESCAPE => { vulkan.window.set_fullscreen(None); }, _ => (),
        }
    }

    pub fn handle_input(keyboard: &Keyboard, keycode: &VirtualKeyCode, state: &ElementState, vulkan: &Vulkan, ) {
        for binding in &keyboard.binding_list {
            if keycode == &binding.key && state == &ElementState::Pressed {
                if binding.binding_type == BindingType::MOVEMENT { Keyboard::change_pos(&binding.action, binding.value as f32); }
                if binding.binding_type == BindingType::GENERAL { Keyboard::general_operation(&binding.action, vulkan); }
            }
        }
    }
}
