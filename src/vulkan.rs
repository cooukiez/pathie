use std::{ffi::{CString, c_void, CStr}, error::Error, time::Duration};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk::{self, DebugUtilsMessengerEXT, SurfaceFormatKHR}, Entry, Instance, Device, util::Align,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{window::{Window, WindowBuilder}, event_loop::EventLoop, dpi::PhysicalSize, monitor::MonitorHandle};
use crate::{data::{Uniform, GraphicPref}, DEBUG, CHUNK_SIZE};

pub struct EngineStatus {
    pub recreate_swapchain: bool,
    pub idle: bool,

    pub frame_time: Duration,
}

pub struct Vulkan {
    pub status: EngineStatus,

    pub window: Window,
    pub monitor_list: Vec<MonitorHandle>,
    pub monitor: MonitorHandle,

    pub instance: Instance,
    pub debug_util: Option<DebugUtils>,
    pub debug_util_messenger: Option<DebugUtilsMessengerEXT>,

    pub surface: Surface,
    pub surface_khr: vk::SurfaceKHR,

    pub physical_device: vk::PhysicalDevice,
    pub physical_device_prop: vk::PhysicalDeviceProperties,
    pub physical_device_memory_prop: vk::PhysicalDeviceMemoryProperties,

    pub graphics_queue_index: u32,
    pub present_queue_index: u32,

    pub device: Device,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub command_pool: vk::CommandPool,
    pub command_buffer_list: Vec<vk::CommandBuffer>,

    pub swapchain_loader: Swapchain,
    pub swapchain_khr: vk::SwapchainKHR, 
    pub swapchain_image_list: Vec<vk::Image>,
    pub swapchain_image_view_list: Vec<vk::ImageView>, 

    pub extent: vk::Extent2D,
    pub scaled_extent: vk::Extent2D,
    pub surface_format: SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub surface_capability: vk::SurfaceCapabilitiesKHR,

    pub available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub fence: vk::Fence,
}

impl Vulkan {
    pub fn init_instance(event_loop: &EventLoop<()>, entry: &Entry) -> (Window, Vec<MonitorHandle>, MonitorHandle, Instance, Option<DebugUtils>, Option<DebugUtilsMessengerEXT>, Surface, vk::SurfaceKHR) {
        log::info!("Creating Window and EventLoop ...");
        
        let window = WindowBuilder::new().with_title(crate::NAME).with_inner_size(PhysicalSize::new(crate::WIDTH, crate::HEIGHT, )).with_resizable(true).build(event_loop).unwrap();

        let monitor_list: Vec<MonitorHandle> = event_loop.available_monitors().collect();
        let monitor = monitor_list.first().expect("ERR_NO_MONITOR").clone();
        log::info!("Moniter is [ {} ]", monitor.name().unwrap(), );

        let (major, minor) = match entry.try_enumerate_instance_version().unwrap() { Some(version) => ( vk::api_version_major(version), vk::api_version_minor(version), ), None => (1, 0), };
        log::info!("Vulkan {:?}.{:?} supported ...", major, minor, );

        log::info!("Creating VulkanInstance ...");
        let name = CString::new(crate::NAME).unwrap();
        let engine_name = CString::new(crate::ENGINE_NAME).unwrap();

        let mut extension_name_vec = ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap().to_vec();
        let app_info = vk::ApplicationInfo::builder().application_name(name.as_c_str()).application_version(vk::make_api_version(0, 0, 1, 0, )).engine_name(engine_name.as_c_str()).engine_version(vk::make_api_version(0, 0, 1, 0, )).api_version(vk::make_api_version(0, major, minor, 0, ));
        extension_name_vec.push(DebugUtils::name().as_ptr());

        let instance_create_info = vk::InstanceCreateInfo::builder().application_info(&app_info).enabled_extension_names(&extension_name_vec);
        let instance = unsafe { entry.create_instance(&instance_create_info, None, ).unwrap() };
        
        let msg_severity = vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE | vk::DebugUtilsMessageSeverityFlagsEXT::INFO | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
        let msg_type = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;

        let callback_info = vk::DebugUtilsMessengerCreateInfoEXT::builder().flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty()).message_severity(msg_severity).message_type(msg_type).pfn_user_callback(Some(Self::vulkan_debug_callback));

        let mut debug_util = None;
        let mut debug_util_messenger = None;

        if DEBUG {
            let util = DebugUtils::new(entry, &instance, );
            let util_messenger = unsafe { util.create_debug_utils_messenger(&callback_info, None, ).unwrap() };

            debug_util = Some(util); debug_util_messenger = Some(util_messenger);
        }
        
        let surface = Surface::new(&entry, &instance);
        let surface_khr = unsafe { ash_window::create_surface(&entry, &instance, window.raw_display_handle(), window.raw_window_handle(), None, ).unwrap() };

        (window, monitor_list, monitor, instance, debug_util, debug_util_messenger, surface, surface_khr, )
    }

    unsafe extern "system" fn vulkan_debug_callback(flag: vk::DebugUtilsMessageSeverityFlagsEXT, msg_type: vk::DebugUtilsMessageTypeFlagsEXT, callback_data: * const vk::DebugUtilsMessengerCallbackDataEXT, _: *mut c_void, ) -> vk::Bool32 {
        use vk::DebugUtilsMessageSeverityFlagsEXT as Flag; 
        let message = CStr::from_ptr((* callback_data).p_message);

        match flag { Flag::VERBOSE => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ), Flag::INFO => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ), Flag::WARNING => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ), _ => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(), ), } vk::FALSE
    }

    pub fn find_memorytype_index(memory_prop: &vk::PhysicalDeviceMemoryProperties, memory_req: &vk::MemoryRequirements, flag: vk::MemoryPropertyFlags, ) -> Option<u32> { memory_prop.memory_types[..memory_prop.memory_type_count as _].iter().enumerate().find(| (index, memory_type) | { (1 << index) & memory_req.memory_type_bits != 0 && memory_type.property_flags & flag == flag }).map(| (index, _, ) | index as _) }

    pub fn check_physical_device(instance: &Instance, device: &vk::PhysicalDevice, surface_khr: vk::SurfaceKHR, surface: &Surface, ) -> (bool, Option<u32>, Option<u32>, ) {
        let mut graphics_queue = None; let mut present_queue = None;
        let device = *device; let prop = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (index, family, ) in prop.iter().filter(| filter | filter.queue_count > 0).enumerate() {
            let index = index as u32; graphics_queue = None; present_queue = None;
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && family.queue_flags.contains(vk::QueueFlags::COMPUTE) && graphics_queue.is_none() { graphics_queue = Some(index); }

            let present_support = unsafe { surface.get_physical_device_surface_support(device, index, surface_khr, ).expect("NO_DEVICE_SURFACE_SUPPORT") };
            if present_support && present_queue.is_none() { present_queue = Some(index); }
            if graphics_queue.is_some() && present_queue.is_some() { break; }
        }

        let extension_prop = unsafe { instance.enumerate_device_extension_properties(device).expect("NO_DEVICE_EXT_PROP") };
        let extention_support = extension_prop.iter().any(| ext | { let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }; Swapchain::name() == name });
        let format_list = unsafe { surface.get_physical_device_surface_formats(device, surface_khr).expect("NO_DEVICE_SURFACE_FORMAT") };
        let present_mode_list = unsafe { surface.get_physical_device_surface_present_modes(device, surface_khr).expect("NO_DEVICE_PRESENT_MODE") };

        (graphics_queue.is_some() && present_queue.is_some() && extention_support && !format_list.is_empty() && !present_mode_list.is_empty(), graphics_queue, present_queue, )
    }

    pub fn get_physical_device_prop(instance: &Instance, physical_device: vk::PhysicalDevice, ) -> (vk::PhysicalDeviceProperties, vk::PhysicalDeviceMemoryProperties) { let physical_device_prop = unsafe { instance.get_physical_device_properties(physical_device) }; let physical_device_memory_prop = unsafe { instance.get_physical_device_memory_properties(physical_device) }; (physical_device_prop, physical_device_memory_prop, ) }

    pub fn init_physical_device(instance: &Instance, surface: &Surface, surface_khr: vk::SurfaceKHR, ) -> (vk::PhysicalDevice, u32, u32, vk::PhysicalDeviceProperties, vk::PhysicalDeviceMemoryProperties ) {
        log::info!("Getting PhysicalDevice ...");
        let physical_device_list = { let mut physical_device_list = unsafe { instance.enumerate_physical_devices().unwrap() }; physical_device_list.sort_by_key(| device | { let prop = unsafe { instance.get_physical_device_properties(*device) }; match prop.device_type { vk::PhysicalDeviceType::DISCRETE_GPU => 0, vk::PhysicalDeviceType::INTEGRATED_GPU => 1, _ => 2, } }); physical_device_list };
    
        let mut graphics_queue = None; let mut present_queue = None;
        let physical_device = physical_device_list.into_iter().find(| device | { let (suitable, graphics_new_val, present_new_val, ) = Vulkan::check_physical_device(&instance, &device, surface_khr, surface, ); graphics_queue = graphics_new_val; present_queue = present_new_val; suitable }).expect("NO_SUITABLE_DEVICE");

        let (physical_device_prop, physical_device_mem_prop, ) = Vulkan::get_physical_device_prop(instance, physical_device);
        let device_name = unsafe { CStr::from_ptr(physical_device_prop.device_name.as_ptr()) }.to_str().unwrap();

        log::info!("Selected PhysicalDevice [ {} ]", &device_name, );
        log::info!("Max WorkGroupSize is [ {} x {} x {} ]", physical_device_prop.limits.max_compute_work_group_size[0], physical_device_prop.limits.max_compute_work_group_size[1], physical_device_prop.limits.max_compute_work_group_size[2], );
        log::info!("Max WorkGroupInvocation [ {} ]", physical_device_prop.limits.max_compute_work_group_invocations, );
        log::info!("Max WorkGroupCount is [ {} x {} x {} ]", physical_device_prop.limits.max_compute_work_group_count[0], physical_device_prop.limits.max_compute_work_group_count[1], physical_device_prop.limits.max_compute_work_group_count[2], );

        (physical_device, graphics_queue.unwrap(), present_queue.unwrap(), physical_device_prop, physical_device_mem_prop, )
    }

    pub fn init_device_and_command_pool(graphics_queue_index: u32, present_queue_index: u32, queue_prioritiy: &[f32], instance: &Instance, physical_device: vk::PhysicalDevice, ) -> (Device, vk::Queue, vk::Queue, Swapchain, ) {
        log::info!("Init LogicalDevice ...");

        let queue_create_info = { let mut index_list = vec![graphics_queue_index, present_queue_index]; index_list.dedup(); index_list.iter().map(| index | { vk::DeviceQueueCreateInfo::builder().queue_family_index(* index).queue_priorities(&queue_prioritiy).build() }).collect::<Vec<_>>() };

        let device_extension = [Swapchain::name().as_ptr()];
        let device_create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_create_info).enabled_extension_names(&device_extension);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None, ).unwrap() };

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0, ) };
        let present_queue = unsafe { device.get_device_queue(present_queue_index, 0, ) };

        let swapchain_loader = Swapchain::new(instance, &device, );

        (device, graphics_queue, present_queue, swapchain_loader, )
    }

    pub fn init_swapchain(surface: &Surface, physical_device: vk::PhysicalDevice, surface_khr: vk::SurfaceKHR, pref_present_mode: vk::PresentModeKHR, window: &Window, img_scale: &u32, loader: &Swapchain, graphics_queue_index: u32, present_queue_index: u32, device: &Device, ) -> (SurfaceFormatKHR, vk::PresentModeKHR, vk::SurfaceCapabilitiesKHR, vk::Extent2D, vk::Extent2D, vk::SwapchainKHR, Vec<vk::Image>, Vec<vk::ImageView>, ) {
        log::info!("Init Swapchain ...");

        let format = { let format_list = unsafe { surface.get_physical_device_surface_formats(physical_device, surface_khr, ).unwrap() }; if format_list.len() == 1 && format_list[0].format == vk::Format::UNDEFINED { vk::SurfaceFormatKHR { format: vk::Format::B8G8R8A8_UNORM, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR, } }  else { * format_list.iter().find(| format | { format.format == vk::Format::B8G8R8A8_UNORM && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR }).unwrap_or(&format_list[0]) }};
        log::info!("Surface Format is [ {:?} ] ...", format);

        let present_mode = { let present_mode_list = unsafe { surface.get_physical_device_surface_present_modes(physical_device, surface_khr, ).expect("NO_PRESENT_MODE") }; if present_mode_list.contains(&pref_present_mode) { pref_present_mode } else { vk::PresentModeKHR::FIFO } };
        log::info!("Surface PresentMode is [ {:?} ] ...", present_mode);

        let capability = unsafe { surface.get_physical_device_surface_capabilities(physical_device, surface_khr, ).unwrap() };

        let extent = { if capability.current_extent.width != std::u32::MAX { capability.current_extent } else { let min = capability.min_image_extent; let max = capability.max_image_extent; let width = window.inner_size().width.min(max.width).max(min.width); let height = window.inner_size().height.min(max.height).max(min.height); vk::Extent2D { width, height } } };
        log::info!("Swapchain Extent is [ {:?} ] ...", extent);
        let scaled_extent = vk::Extent2D { width: extent.width / img_scale, height: extent.height / img_scale };
        log::info!("Scaled Extent is [ {:?} ] ...", scaled_extent);

        let image_count = capability.min_image_count;
        log::info!("Swapchain ImageCount is [ {:?} ] ...", image_count);

        let queue_family = [graphics_queue_index, present_queue_index];
        let swapchain_info = { let mut builder = vk::SwapchainCreateInfoKHR::builder().surface(surface_khr).min_image_count(image_count).image_format(format.format).image_color_space(format.color_space).image_extent(extent).image_array_layers(1).image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT); builder = if graphics_queue_index != present_queue_index { builder.image_sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&queue_family) } else { builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE) }; builder.pre_transform(capability.current_transform).composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE).present_mode(present_mode).clipped(true) };
        let swapchain_khr = unsafe { loader.create_swapchain(&swapchain_info, None, ).unwrap() };

        let image_list = unsafe { loader.get_swapchain_images(swapchain_khr).unwrap() };
        let image_view_list = image_list.iter().map(| image | { let view_info = vk::ImageViewCreateInfo::builder().image(* image).view_type(vk::ImageViewType::TYPE_2D).format(format.format).subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, }); unsafe { device.create_image_view(&view_info, None, ) } }).collect::<Result<Vec<_>, _>>().unwrap();

        (format, present_mode, capability, extent, scaled_extent, swapchain_khr, image_list, image_view_list, )
    }

    pub fn init_command_pool(graphics_queue_index: u32, device: &Device, swapchain_image_list: &Vec<vk::Image>, ) -> (vk::CommandPool, Vec<vk::CommandBuffer>, ) {
        let command_pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(graphics_queue_index).flags(vk::CommandPoolCreateFlags::empty()); 
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None, ).unwrap() };

        let allocate_info = vk::CommandBufferAllocateInfo::builder().command_pool(command_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(swapchain_image_list.len() as u32);
        let command_buffer_list = unsafe { device.allocate_command_buffers(&allocate_info).unwrap() };

        (command_pool, command_buffer_list, )
    }

    pub fn wait_for_gpu(device: &Device) -> Result<(), Box<dyn Error>> { unsafe { Ok(device.device_wait_idle().unwrap()) } }

    pub fn init_sync(device: &Device) -> (vk::Semaphore, vk::Semaphore, vk::Fence, ) {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();

        let available = unsafe { device.create_semaphore(&semaphore_info, None, ).unwrap() };
        let render_finished = unsafe { device.create_semaphore(&semaphore_info, None, ).unwrap() };

        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_info, None).unwrap() };

        (available, render_finished, fence, )
    }

    pub fn clean_up_swap_recreate(device: &Device, command_pool: vk::CommandPool, command_buffer_list: &Vec<vk::CommandBuffer>, swapchain_image_view_list: &Vec<vk::ImageView>, swapchain: &Swapchain, swapchain_khr: vk::SwapchainKHR) {
        unsafe { device.free_command_buffers(command_pool, command_buffer_list, ); }
        unsafe { device.destroy_command_pool(command_pool, None, ); }

        unsafe { swapchain.destroy_swapchain(swapchain_khr, None, ) }
        swapchain_image_view_list.iter().for_each(| image_view | unsafe { device.destroy_image_view(* image_view, None, ) });
    }
}

pub struct BufferObj {
    pub buffer: vk::Buffer,
    pub buffer_mem: vk::DeviceMemory,
}

pub struct ImageObj {
    pub image: vk::Image,
    pub image_mem: vk::DeviceMemory,
    pub image_view: vk::ImageView,
}

pub struct PipelineData {
    pub buffer_list: Vec<BufferObj>,
    pub uniform_list: Vec<BufferObj>,
    pub image_list: Vec<ImageObj>,
}

impl PipelineData { 
    pub fn init_image(image_layout: vk::ImageLayout, format: vk::Format, extent: &vk::Extent2D, device: &Device, mem_prop: &vk::PhysicalDeviceMemoryProperties, ) -> ImageObj {
        log::info!("Init Image - ImageLayout [ {:?} ] ...", image_layout);

        let image_info = vk::ImageCreateInfo::builder().format(format).extent(vk::Extent3D { width: extent.width, height: extent.height, depth: 1 }).mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1).tiling(vk::ImageTiling::OPTIMAL).usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC).sharing_mode(vk::SharingMode::EXCLUSIVE).initial_layout(image_layout).image_type(vk::ImageType::TYPE_2D).build();
        let image = unsafe { device.create_image(&image_info, None, ).unwrap() };

        let mem_requirement = unsafe { device.get_image_memory_requirements(image) };
        let mem_index = Vulkan::find_memorytype_index(mem_prop, &mem_requirement, vk::MemoryPropertyFlags::DEVICE_LOCAL, ).expect("NO_SUITABLE_MEM_TYPE_INDEX");

        let allocate_info = vk::MemoryAllocateInfo::builder().allocation_size(mem_requirement.size).memory_type_index(mem_index).build();
        let image_mem = unsafe { device.allocate_memory(&allocate_info, None, ).unwrap() };
        unsafe { device.bind_image_memory(image, image_mem, 0, ).expect("UNABLE_TO_BIND_MEM"); }

        let image_view_info = vk::ImageViewCreateInfo::builder().view_type(vk::ImageViewType::TYPE_2D).format(format).subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, }).image(image).components(vk::ComponentMapping { r: vk::ComponentSwizzle::R, g: vk::ComponentSwizzle::G, b: vk::ComponentSwizzle::B, a: vk::ComponentSwizzle::A, }).build();
        let image_view = unsafe { device.create_image_view(&image_view_info, None, ).unwrap() };

        ImageObj { image, image_mem, image_view }
    }

    pub fn init_storage_buffer(usage: vk::BufferUsageFlags, size: u64, device: &Device, mem_prop: &vk::PhysicalDeviceMemoryProperties, ) -> BufferObj {
        log::info!("Init Buffer [ {:?} ] ...", usage);

        let buffer_info = vk::BufferCreateInfo::builder().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE).build();
        let buffer = unsafe { device.create_buffer(&buffer_info, None, ).unwrap() };
        
        let mem_requirement = unsafe { device.get_buffer_memory_requirements(buffer) };
        let mem_index = Vulkan::find_memorytype_index(mem_prop, &mem_requirement, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, ).expect("NO_SUITABLE_MEM_TYPE_INDEX");

        let allocate_info = vk::MemoryAllocateInfo::builder().allocation_size(mem_requirement.size).memory_type_index(mem_index).build();
        let buffer_mem = unsafe { device.allocate_memory(&allocate_info, None, ).unwrap() };
        
        unsafe { device.bind_buffer_memory(buffer, buffer_mem, 0, ).unwrap(); }

        BufferObj { buffer, buffer_mem }
    }

    pub fn update_uniform_buffer(device: &Device, buffer_mem: vk::DeviceMemory, data: &[Uniform], ) {
        let ptr: * mut c_void = unsafe { device.map_memory(buffer_mem, 0, std::mem::size_of::<Uniform>() as u64, vk::MemoryMapFlags::empty(), ).unwrap() };
        let mut slice = unsafe { Align::new(ptr, std::mem::align_of::<Uniform>() as u64, std::mem::size_of::<Uniform>() as u64, ) };
        slice.copy_from_slice(&data); unsafe { device.unmap_memory(buffer_mem); }
    }

    pub fn update_voxel_buffer(device: &Device, buffer_mem: vk::DeviceMemory, data: &[i32; CHUNK_SIZE], ) {
        log::info!("AlignSize VoxBuffer - {}", std::mem::align_of::<[i32; CHUNK_SIZE]>() as u64);
        let ptr: * mut c_void = unsafe { device.map_memory(buffer_mem, 0, std::mem::size_of::<[i32; CHUNK_SIZE]>() as u64, vk::MemoryMapFlags::empty(), ).unwrap() };
        let mut slice = unsafe { Align::new(ptr, std::mem::align_of::<[i32; CHUNK_SIZE]>() as u64, std::mem::size_of::<[i32; CHUNK_SIZE]>() as u64, ) };
        slice.copy_from_slice(data); unsafe { device.unmap_memory(buffer_mem); }
    }

    pub fn update_graphic_pref_buffer(device: &Device, buffer_mem: vk::DeviceMemory, data: &[GraphicPref], ) {
        log::info!("AlignSize GraphicPrefBuffer - {}", std::mem::align_of::<Uniform>() as u64);
        let ptr: * mut c_void = unsafe { device.map_memory(buffer_mem, 0, std::mem::size_of::<GraphicPref>() as u64, vk::MemoryMapFlags::empty(), ).unwrap() };
        let mut slice = unsafe { Align::new(ptr, std::mem::align_of::<GraphicPref>() as u64, std::mem::size_of::<GraphicPref>() as u64, ) };
        slice.copy_from_slice(&data); unsafe { device.unmap_memory(buffer_mem); }
    }

    pub fn create_desc_layout<Type>(list: &Vec<Type>, desc_type: vk::DescriptorType, device: &Device, ) -> vk::DescriptorSetLayout {
        let mut binding_list: Vec<vk::DescriptorSetLayoutBinding> = vec![];
        for (index, _, ) in list.iter().enumerate() { binding_list.push(vk::DescriptorSetLayoutBinding::builder().descriptor_type(desc_type).binding(index as u32).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE).build()) }

        let set_layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&binding_list).build();
        unsafe { device.create_descriptor_set_layout(&set_layout_info, None, ).unwrap() }
    }
}