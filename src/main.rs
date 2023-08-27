use std::{
    borrow::BorrowMut,
    io::Write,
    mem, thread,
    time::{Duration, Instant},
};

use ash::vk;
use cgmath::Vector2;
use env_logger::fmt::{Color, Formatter};
use input::Input;
use interface::interface::Interface;
use log::Record;
use nalgebra_glm::{cross, normalize, vec3_to_vec4, Vec2};
use pipe::engine::Engine;
use tree::octree::Octree;
use uniform::Uniform;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

mod bit;
mod input;
mod interface;
mod pipe;
mod tree;
mod uniform;
mod vector;

const DEFAULT_STORAGE_BUFFER_SIZE: u64 = 1342177280;
const DEFAULT_UNIFORM_BUFFER_SIZE: u64 = 16384;

pub struct RenderState {
    pub out_of_date: bool,
    pub idle: bool,

    pub frame_time: Duration,
}

// Complete Render Pipeline
pub struct Render {
    state: RenderState,
    event_loop: EventLoop<()>,

    pref: Pref,
    uniform: Uniform,
    octree: Octree,

    input: Input,

    interface: Interface,
    graphic_pipe: Engine,
}

// General Setting
pub struct Pref {
    pub pref_present_mode: vk::PresentModeKHR,
    pub img_filter: vk::Filter,
    pub img_scale: f32,

    pub name: String,
    pub engine_name: String,

    pub start_window_size: vk::Extent2D,

    pub use_render_res: bool,
    pub render_res: vk::Extent2D,

    pub mov_speed: f32,
}

fn main() {
    let log_format = |buf: &mut Formatter, record: &Record| {
        let mut buf_style = buf.style();

        buf_style.set_color(Color::Yellow).set_bold(true);

        let time = chrono::Local::now().format("%H:%M:%S");

        writeln!(
            buf,
            "[ {} {} ] {}",
            time,
            buf_style.value(record.level()),
            record.args(),
        )
    };

    env_logger::builder().format(log_format).init();

    log::info!("Starting Application ...");
    thread::spawn(|| loop {});

    let mut render = Render::get_render();
    render.execute(Instant::now());

    render.graphic_pipe.drop_graphic(&render.interface);
}

impl Render {
    pub fn get_render() -> Render {
        let event_loop = EventLoop::new();

        let pref = Pref {
            pref_present_mode: vk::PresentModeKHR::IMMEDIATE,
            img_filter: vk::Filter::LINEAR,
            img_scale: 1.0,

            name: env!("CARGO_PKG_NAME").to_string(),
            engine_name: "Engine".to_string(),

            start_window_size: vk::Extent2D {
                width: 800,
                height: 600,
            },

            use_render_res: true,
            render_res: vk::Extent2D {
                width: 1920,
                height: 1080,
            },

            mov_speed: 0.05,
        };

        let state = RenderState {
            out_of_date: false,
            idle: false,
            frame_time: Duration::ZERO,
        };

        let mut octree = Octree::default();

        let input = Input::new();
        let mut uniform = Uniform::new(octree.root_span);

        octree.test_scene();

        let interface = Interface::init(&event_loop, &pref);
        uniform.res = Vec2::new(
            interface.surface.surface_res.width as f32,
            interface.surface.surface_res.height as f32,
        );

        let mut graphic_pipe = Engine::create_base(&interface, &uniform, &octree);
        // graphic_pipe = graphic_pipe.create_compute(&interface, &uniform, &octree);
        graphic_pipe = graphic_pipe
            .create_jfa_comp(&interface, &uniform, &octree)
            .create_graphic(&interface, &uniform, &octree);

        Render {
            state,
            event_loop,
            pref,
            uniform,
            octree,
            input,
            interface,
            graphic_pipe,
        }
    }

    pub fn execute(&mut self, app_start: Instant) {
        self.event_loop
            .borrow_mut()
            .run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::WindowEvent {
                        event:
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        virtual_keycode: Some(keycode),
                                        state,
                                        ..
                                    },
                                ..
                            },
                        ..
                    } =>
                    // Handle KeyboardInput
                    {
                        self.input.handle_key_input(
                            &keycode,
                            &state,
                            &mut self.uniform,
                            &self.pref,
                            &self.octree,
                            &self.interface,
                        )
                    }

                    Event::WindowEvent {
                        event: WindowEvent::CursorMoved { position, .. },
                        ..
                    } => {
                        self.input.handle_mouse_input(position, &mut self.uniform);
                        self.interface.window.set_cursor_visible(false);
                        self.interface
                            .window
                            .set_cursor_position(PhysicalPosition::new(
                                self.uniform.res.x / 2.0,
                                self.uniform.res.y / 2.0,
                            ))
                            .unwrap();
                    }

                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    Event::MainEventsCleared =>
                    // Adjust Surface and Draw
                    {
                        if self.state.out_of_date {
                            let dim = self.interface.window.inner_size();
                            if dim.width > 0 && dim.height > 0 {
                                // Not Minimized
                                self.graphic_pipe.recreate_swapchain(
                                    &mut self.interface,
                                    &mut self.uniform,
                                    &self.pref,
                                );

                                self.state.idle = false;
                                self.state.out_of_date = false;
                            } else {
                                // Minimized
                                self.state.idle = true;
                            }
                        } else {
                            // Update Octree
                            // self.octree = Octree::default();
                            // self.octree.test_scene();
                            // self.graphic_pipe.update_buffer(&self.interface, self.graphic_pipe.octree_buffer_memory, &self.octree.data.clone(), );

                            // Update Uniform
                            self.uniform.update_uniform(app_start.elapsed());

                            self.graphic_pipe.uniform_buffer.rewrite_mem(
                                &self.interface,
                                mem::align_of::<Uniform>() as u64,
                                mem::size_of::<Uniform>() as u64,
                                &[self.uniform],
                            );

                            // Draw and capture FrameTime
                            let start = Instant::now();
                            self.state.out_of_date = self
                                .graphic_pipe
                                .draw_graphic(&self.interface, &self.pref, &self.uniform)
                                .expect("RENDER_FAILED");
                            self.state.frame_time = start.elapsed();

                            if self.input.key_down[VirtualKeyCode::W as usize] == true {
                                self.uniform.velocity +=
                                    nalgebra_glm::normalize(&self.uniform.look_dir)
                                        * self.pref.mov_speed;
                                self.uniform.apply_velocity();
                            }
                            if self.input.key_down[VirtualKeyCode::S as usize] == true {
                                self.uniform.velocity -=
                                    nalgebra_glm::normalize(&self.uniform.look_dir)
                                        * self.pref.mov_speed;
                                self.uniform.apply_velocity();
                            }
                            if self.input.key_down[VirtualKeyCode::A as usize] == true {
                                self.uniform.velocity -= vec3_to_vec4(&normalize(&cross(
                                    &nalgebra_glm::normalize(&self.uniform.look_dir.xyz()),
                                    &self.uniform.cam_up.xyz(),
                                ))) * self.pref.mov_speed;
                                self.uniform.apply_velocity();
                            }
                            if self.input.key_down[VirtualKeyCode::D as usize] == true {
                                self.uniform.velocity += vec3_to_vec4(&normalize(&cross(
                                    &nalgebra_glm::normalize(&self.uniform.look_dir.xyz()),
                                    &self.uniform.cam_up.xyz(),
                                ))) * self.pref.mov_speed;
                                self.uniform.apply_velocity();
                            }
                            if self.input.key_down[VirtualKeyCode::LShift as usize] == true {
                                self.pref.mov_speed = 0.3;
                            } else {
                                self.pref.mov_speed = 0.05;
                            }
                        }
                    }

                    Event::LoopDestroyed => self.interface.wait_for_gpu().expect("DEVICE_LOST"),
                    _ => (),
                }
            });
    }
}
