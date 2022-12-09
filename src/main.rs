
use std::{io::Write, error::Error, thread, time::{Instant, Duration}, ptr::null_mut};
use ash::{Entry, vk};
use cgmath::{Vector3, Vector2};
use data::{WorldData, Uniform};
use env_logger::fmt::Color;

use pipeline::{Render};
use vulkan::{Vulkan, EngineStatus, PipelineData, BufferObj};
use winit::{event_loop::{ControlFlow, EventLoop}, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, window::{Fullscreen}};

mod vulkan;
mod pipeline;
mod data;

// const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const ENGINE_NAME: &str = "VulkanEngine";

const DEFAULT_STORAGE_BUFFER_SIZE: u64 = 10485760;
const DEFAULT_UNIFORM_BUFFER_SIZE: u64 = 16384;
  
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const CHUNK_SIDE_LEN: usize = 8;
const CHUNK_SIZE: usize = 512;
const CHUNK_GROUP_SIDE_LEN: usize = 3;
const CHUNK_GROUP_SIZE: usize = 27;

const DEFAULT_FOV: f32 = 60.0;
const DEFAULT_MAX_RAY_LEN: u32 = 750;

static mut UNIFORM: Uniform = Uniform { 
    time: 0,

    field_of_view: DEFAULT_FOV,
    max_ray_length: DEFAULT_MAX_RAY_LEN,

    head_rot: Vector2::new(0, 0),
    player_pos: Vector3::new(1, 1, 1),

    chunk_side_len: CHUNK_SIDE_LEN as u32,
    chunk_size: CHUNK_SIZE as u32,

    chunk_group_side_len: CHUNK_GROUP_SIDE_LEN as u32,
    chunk_group_size: CHUNK_GROUP_SIZE as u32,
};

pub struct Pref {
    // Render
    pub pref_present_mode: vk::PresentModeKHR,
    pub img_scale: u32,

    // Tracer
    pub field_of_view: f32,
    pub max_ray_len: u32,

    // Movement
    pub key_rot_control_inc: i32,
}
 
fn main() {
    env_logger::builder().format(|buf, record| { let mut bold = buf.style(); bold.set_color(Color::Yellow).set_bold(true); writeln!(buf, "[ {} {} ] {}", chrono::Local::now().format("%H:%M:%S"), bold.value(record.level(), ), record.args(), ) }).init();
    let app_start = Instant::now();

    thread::spawn(|| { loop { } });

    run_graphic_related(app_start);
}

fn run_graphic_related(app_start: Instant) {
    let pref = Pref { 
        pref_present_mode: vk::PresentModeKHR::IMMEDIATE, 
        img_scale: 3, 
        field_of_view: DEFAULT_FOV, 
        max_ray_len: DEFAULT_MAX_RAY_LEN, 
        key_rot_control_inc: 5 
    };

    let event_loop = EventLoop::new();
    let entry = unsafe { Entry::load().unwrap() };
    let status = EngineStatus { recreate_swapchain: false, idle: false, frame_time: Duration::ZERO };

    // Init VulkanLib Part
    let (window, monitor_list, monitor, instance, debug_util, debug_util_messenger, surface, surface_khr, ) = Vulkan::init_instance(&event_loop, &entry);
    let (physical_device, graphics_queue_index, present_queue_index, physical_device_prop, physical_device_memory_prop) = Vulkan::init_physical_device(&instance, &surface, surface_khr, );
    let (device, graphics_queue, present_queue, swapchain_loader, ) = Vulkan::init_device_and_command_pool(graphics_queue_index, present_queue_index, &[1.0f32], &instance, physical_device, );
    let (surface_format, present_mode, surface_capability, extent, scaled_extent, swapchain_khr, swapchain_image_list, swapchain_image_view_list, ) = Vulkan::init_swapchain(&surface, physical_device, surface_khr, vk::PresentModeKHR::FIFO, &window, &pref.img_scale, &swapchain_loader, graphics_queue_index, present_queue_index, &device, );
    let (command_pool, command_buffer_list, ) = Vulkan::init_command_pool(graphics_queue_index, &device, &swapchain_image_list);
    let (available, render_finished, fence, ) = Vulkan::init_sync(&device);
    
    let mut vulkan = Vulkan { status, window, monitor_list, monitor, instance, debug_util, debug_util_messenger, surface, surface_khr, physical_device, physical_device_prop, physical_device_memory_prop, graphics_queue_index, present_queue_index, device, graphics_queue, present_queue, swapchain_loader, swapchain_khr, swapchain_image_list, swapchain_image_view_list, extent, scaled_extent, surface_format, present_mode, surface_capability, command_pool, command_buffer_list, available, render_finished, fence };

    // Init ComputeRenderPipeline
    let image = PipelineData::init_image(vk::ImageLayout::UNDEFINED, vulkan.surface_format.format, &vulkan.scaled_extent, &vulkan.device, &vulkan.physical_device_memory_prop, );

    let uniform_list: Vec<BufferObj> = vec![PipelineData::init_storage_buffer(vk::BufferUsageFlags::UNIFORM_BUFFER, DEFAULT_UNIFORM_BUFFER_SIZE, &vulkan.device, &vulkan.physical_device_memory_prop, )];

    let world_data = WorldData::collect();
    let buffer_list: Vec<BufferObj> = vec![PipelineData::init_storage_buffer(vk::BufferUsageFlags::STORAGE_BUFFER, DEFAULT_STORAGE_BUFFER_SIZE, &vulkan.device, &vulkan.physical_device_memory_prop, )];
    
    PipelineData::update_uniform_buffer(&vulkan.device, uniform_list[0].buffer_mem, unsafe { &[UNIFORM] }, );
    PipelineData::update_voxel_buffer(&vulkan.device, buffer_list[0].buffer_mem, &world_data.voxel_data, );
    
    let (descriptor_pool, descriptor_set_layout_list, ) = Render::init_descriptor_pool(&buffer_list, &uniform_list, &vulkan.device, &image, );
    let descriptor_set_list = Render::update_descriptor_pool(descriptor_pool, &descriptor_set_layout_list, &vulkan.device, vk::ImageLayout::GENERAL, &image, &buffer_list, &uniform_list, );
    let (compute_pipeline, pipeline_layout, ) = Render::init_compute_pipeline(&vulkan.device, &descriptor_set_layout_list, );

    Render::record_command_pool(&vulkan.command_buffer_list, &vulkan.device, compute_pipeline, pipeline_layout, &descriptor_set_list, &vulkan.extent, &vulkan.scaled_extent, &image, &vulkan.swapchain_image_list, 0, );

    let mut render = Render { image, buffer_list, uniform_list, descriptor_pool, descriptor_set_layout_list, pipeline_layout, descriptor_set_list, compute_pipeline };

    event_loop.run(move | event, _, control_flow | { * control_flow = ControlFlow::Poll; handle_event(&event, &mut vulkan, &mut render, &pref, &app_start, control_flow, ); });
}

pub fn handle_event(event: &Event<()>, vulkan: &mut Vulkan, render: &mut Render, pref: &Pref, app_start: &Instant, control_flow: &mut ControlFlow, ) {
    match event {
        Event::WindowEvent { event: WindowEvent::Resized(..), .. } => { log::info!("Window -> Resize ..."); vulkan.status.recreate_swapchain = true; }
        Event::MainEventsCleared => { vulkan.status.recreate_swapchain = draw(vulkan, render, pref, app_start, ).expect("TICK_FAILED"); }
        Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(keycode), state, .. }, .. }, .. } => handle_input(keycode, state, &vulkan, &pref, ),
        Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => * control_flow = ControlFlow::Exit,
        Event::LoopDestroyed => Vulkan::wait_for_gpu(&vulkan.device).expect("FAILED_WORK"), _ => (),
    }
}

pub fn handle_input(keycode: &VirtualKeyCode, state: &ElementState, vulkan: &Vulkan, pref: &Pref, ) {
    match keycode {
        &VirtualKeyCode::F if state == &ElementState::Pressed => { vulkan.window.set_fullscreen(Some(Fullscreen::Exclusive(vulkan.monitor.video_modes().next().expect("ERR_NO_MONITOR_MODE").clone()))); },
        &VirtualKeyCode::W if state == &ElementState::Pressed => { unsafe { UNIFORM.head_rot.y += pref.key_rot_control_inc }; },
        &VirtualKeyCode::S if state == &ElementState::Pressed => { unsafe { UNIFORM.head_rot.y -= pref.key_rot_control_inc }; },
        &VirtualKeyCode::A if state == &ElementState::Pressed => { unsafe { UNIFORM.head_rot.x += pref.key_rot_control_inc }; },
        &VirtualKeyCode::D if state == &ElementState::Pressed => { unsafe { UNIFORM.head_rot.x -= pref.key_rot_control_inc }; },
        &VirtualKeyCode::Escape if state == &ElementState::Pressed => { vulkan.window.set_fullscreen(None); }, _ => (),
    }
}

pub fn draw(vulkan: &mut Vulkan, render: &mut Render, pref: &Pref, app_start: &Instant, ) -> Result<bool, Box<dyn Error>> {
    if vulkan.status.recreate_swapchain { let dim = vulkan.window.inner_size(); if dim.width > 0 && dim.height > 0 { recreate_swapchain(vulkan, render, pref.pref_present_mode, &pref, ); return Ok(false); } }

    let fence = vulkan.fence;
    unsafe { vulkan.device.wait_for_fences(&[fence], true, std::u64::MAX, ).unwrap() };

    let next_image_result = unsafe { vulkan.swapchain_loader.acquire_next_image(vulkan.swapchain_khr, std::u64::MAX, vulkan.available, vk::Fence::null(), ) };
    let image_index = match next_image_result { Ok((image_index, _, )) => image_index, Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return Ok(true); } Err(error) => panic!("ERROR_AQUIRE_IMAGE -> {}", error, ), };

    unsafe { vulkan.device.reset_fences(&[fence]).unwrap() };

    let start = Instant::now();

    Render::record_command_pool(&vulkan.command_buffer_list, &vulkan.device, render.compute_pipeline, render.pipeline_layout, &render.descriptor_set_list, &vulkan.extent, &vulkan.scaled_extent, &render.image, &vulkan.swapchain_image_list, image_index as usize, );

    let wait_stage_list = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let wait_sema_list = [vulkan.available];
    let signal_sema_list = [vulkan.render_finished];

    let command_buffer_list = [vulkan.command_buffer_list[image_index as usize]];
    let submit_info = [vk::SubmitInfo::builder().wait_semaphores(&wait_sema_list).wait_dst_stage_mask(&wait_stage_list).command_buffers(&command_buffer_list).signal_semaphores(&signal_sema_list).build()];
    unsafe { vulkan.device.queue_submit(vulkan.graphics_queue, &submit_info, fence).unwrap() };

    let swapchain_list = [vulkan.swapchain_khr];
    let image_index_list = [image_index];
    let present_info = vk::PresentInfoKHR::builder().wait_semaphores(&wait_sema_list).swapchains(&swapchain_list).image_indices(&image_index_list);

    let present_result = unsafe { vulkan.swapchain_loader.queue_present(vulkan.present_queue, &present_info, ) };

    vulkan.status.frame_time = start.elapsed();

    unsafe { UNIFORM.time = app_start.elapsed().as_millis() as u32 };
    PipelineData::update_uniform_buffer(&vulkan.device, render.uniform_list[0].buffer_mem, unsafe { &[UNIFORM] }, );

    match present_result { Ok(is_suboptimal) if is_suboptimal => { return Ok(true); } Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return Ok(true); } Err(error) => panic!("ERROR_PRESENT_SWAP -> {}", error, ), _ => { } } Ok(false)
}

pub fn recreate_swapchain(vulkan: &mut Vulkan, render: &mut Render, pref_present_mode: vk::PresentModeKHR, pref: &Pref, ) {
    Vulkan::wait_for_gpu(&vulkan.device).unwrap();

    Vulkan::clean_up_swap_recreate(&vulkan.device, vulkan.command_pool, &vulkan.command_buffer_list, &vulkan.swapchain_image_view_list, &vulkan.swapchain_loader, vulkan.swapchain_khr);
    Render::clean_up_swap_recreate(&vulkan.device, render.compute_pipeline, render.pipeline_layout, &render.descriptor_set_layout_list, render.descriptor_pool, &render.image, );

    let (surface_format, present_mode, surface_capability, extent, scaled_extent, swapchain_khr, swapchain_image_list, swapchain_image_view_list, ) = Vulkan::init_swapchain(&vulkan.surface, vulkan.physical_device, vulkan.surface_khr, pref_present_mode, &vulkan.window, &pref.img_scale, &vulkan.swapchain_loader, vulkan.graphics_queue_index, vulkan.present_queue_index, &vulkan.device, ); vulkan.surface_format = surface_format; vulkan.present_mode = present_mode; vulkan.surface_capability = surface_capability; vulkan.extent = extent; vulkan.scaled_extent = scaled_extent; vulkan.swapchain_khr = swapchain_khr; vulkan.swapchain_image_list = swapchain_image_list; vulkan.swapchain_image_view_list = swapchain_image_view_list;
    let (command_pool, command_buffer_list, ) = Vulkan::init_command_pool(vulkan.graphics_queue_index, &vulkan.device, &vulkan.swapchain_image_list); vulkan.command_pool = command_pool; vulkan.command_buffer_list = command_buffer_list;

    let image = PipelineData::init_image(vk::ImageLayout::UNDEFINED, vulkan.surface_format.format, &vulkan.scaled_extent, &vulkan.device, &vulkan.physical_device_memory_prop, ); render.image = image;

    let (descriptor_pool, descriptor_set_layout_list, ) = Render::init_descriptor_pool(&render.buffer_list, &render.uniform_list, &vulkan.device, &render.image, ); render.descriptor_pool = descriptor_pool; render.descriptor_set_layout_list = descriptor_set_layout_list;
    let descriptor_set_list = Render::update_descriptor_pool(render.descriptor_pool, &render.descriptor_set_layout_list, &vulkan.device, vk::ImageLayout::GENERAL, &render.image, &render.buffer_list, &render.uniform_list, ); render.descriptor_set_list = descriptor_set_list;

    let (compute_pipeline, pipeline_layout, ) = Render::init_compute_pipeline(&vulkan.device, &render.descriptor_set_layout_list, ); render.compute_pipeline = compute_pipeline; render.pipeline_layout = pipeline_layout;
}