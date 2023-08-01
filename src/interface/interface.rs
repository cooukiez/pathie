use crate::{
    interface::{
        phydev::PhyDeviceGroup,
        surface::SurfaceGroup,
        swapchain::SwapchainGroup,
    },
    Pref,
};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{DynamicRendering, Swapchain},
    },
    vk,
    Device, Entry, Instance,
};
use raw_window_handle::HasRawDisplayHandle;
use std::{
    error::Error,
    ffi::{c_void, CStr, CString},
};
use winit::{event_loop::EventLoop, monitor::MonitorHandle, window::WindowBuilder};

pub struct Interface {
    pub entry: Entry,
    pub instance: Instance,
    pub debug_util_loader: DebugUtils,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,

    pub window: winit::window::Window,
    pub monitor_list: Vec<MonitorHandle>,
    pub monitor: MonitorHandle,

    pub surface: SurfaceGroup,
    pub phy_device: PhyDeviceGroup,

    pub device: Device,
    pub present_queue: vk::Queue,

    pub swapchain: SwapchainGroup,

    pub pool: vk::CommandPool,
    pub setup_cmd_buffer: vk::CommandBuffer,
    pub draw_cmd_buffer: vk::CommandBuffer,

    pub present_complete: vk::Semaphore,
    pub render_complete: vk::Semaphore,

    pub draw_cmd_fence: vk::Fence,
    pub setup_cmd_fence: vk::Fence,
}

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let base_zeroed: $base = mem::zeroed();
            std::ptr::addr_of!(base_zeroed.$field) as isize
                - std::ptr::addr_of!(base_zeroed) as isize
        }
    }};
}

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Flag;
    let message = CStr::from_ptr((*callback_data).p_message);

    match flag {
        Flag::VERBOSE => log::info!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
        Flag::INFO => log::info!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
        Flag::WARNING => log::info!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
        _ => log::info!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
    }

    log::info!("");

    return vk::FALSE;
}

impl Interface {
    pub fn init(event_loop: &EventLoop<()>, pref: &Pref) -> Self {
        unsafe {
            log::info!("Creating Window and EventLoop ...");
            let window = WindowBuilder::new()
                .with_title(pref.name.clone())
                .with_inner_size(winit::dpi::LogicalSize::new(
                    f64::from(pref.start_window_size.width),
                    f64::from(pref.start_window_size.height),
                ))
                .build(event_loop)
                .unwrap();

            // Get list of monitor and choose one
            let monitor_list: Vec<MonitorHandle> = event_loop.available_monitors().collect();
            let monitor = monitor_list.first().expect("ERR_NO_MONITOR").clone();
            log::info!("Moniter is [ {} ]", monitor.name().unwrap(),);

            let entry = Entry::load().unwrap();

            log::info!("Creating VulkanInstance ...");
            let name = CString::new(pref.name.clone()).unwrap();
            let engine_name = CString::new(pref.engine_name.clone()).unwrap();

            let mut ext_name_list =
                ash_window::enumerate_required_extensions(window.raw_display_handle())
                    .unwrap()
                    .to_vec();
            ext_name_list.push(DebugUtils::name().as_ptr());

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                ext_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
                ext_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
            }

            let (major, minor) = match entry.try_enumerate_instance_version().unwrap() {
                Some(version) => (
                    vk::api_version_major(version),
                    vk::api_version_minor(version),
                ),
                None => (1, 0),
            };

            log::info!("Vulkan {:?}.{:?} supported ...", major, minor,);

            let app_info = vk::ApplicationInfo::builder()
                .application_name(name.as_c_str())
                .application_version(vk::make_api_version(0, 0, 1, 0))
                .engine_name(engine_name.as_c_str())
                .engine_version(vk::make_api_version(0, 0, 1, 0))
                .api_version(vk::make_api_version(0, major, minor, 0));

            let create_flag = if cfg!(any(target_os = "macos", target_os = "ios",)) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&ext_name_list)
                .flags(create_flag);

            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("ERR_CREATE_INSTANCE");

            // Debug part -> Validation layer stuff
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_util_loader = DebugUtils::new(&entry, &instance);
            let debug_call_back = debug_util_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();

            let mut surface = SurfaceGroup::new(&entry, &instance, &window);

            log::info!("Creating PhyDevice ...");
            let phy_device = PhyDeviceGroup::default()
                .get_phy_device_list(&instance)
                .get_suitable_phy_device(&instance, &surface)
                .get_phy_device_prop(&instance);

            log::info!("Load Surface information ...");
            surface = surface.get_surface_info(&phy_device, &window, pref);

            let device_ext_list = [
                Swapchain::name().as_ptr(),
                // DynamicRendering::name().as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios",))]
                KhrPortabilitySubsetFn::name().as_ptr(),
            ];

            let feature = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };

            log::info!("Get QueueList ...");
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(phy_device.queue_family_index)
                .queue_priorities(&[1f32]);

            let mut dynamic_rendering_feature =
                vk::PhysicalDeviceDynamicRenderingFeaturesKHR::builder().dynamic_rendering(true);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_ext_list)
                .enabled_features(&feature)
                .push_next(&mut dynamic_rendering_feature);

            let device: Device = instance
                .create_device(phy_device.device, &device_create_info, None)
                .unwrap();

            let present_queue = device.get_device_queue(phy_device.queue_family_index, 0);

            log::info!("Creating Swapchain ...");
            let mut swapchain = SwapchainGroup::new(&instance, &device).create_swapchain(&surface);

            log::info!("Creating CommandPool ...");
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(phy_device.queue_family_index);

            let pool = device.create_command_pool(&pool_create_info, None).unwrap();

            log::info!("Creating CommandBuffer ...");
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffer_list = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();

            let setup_cmd_buffer = command_buffer_list[0];
            let draw_cmd_buffer = command_buffer_list[1];

            log::info!("Load PresentImgList ...");
            swapchain = swapchain.get_present_img(&surface, &device);

            log::info!("Init Fence ...");
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_cmd_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");
            let setup_cmd_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");

            log::info!("Init Semaphore ...");
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let render_complete = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            log::info!("Interface finished ...");
            Interface {
                entry,
                instance,

                debug_util_loader,
                debug_call_back,

                window,
                monitor_list,
                monitor,

                surface,

                phy_device,

                device,
                present_queue,

                swapchain,

                pool,
                setup_cmd_buffer,
                draw_cmd_buffer,

                present_complete,
                render_complete,

                draw_cmd_fence,
                setup_cmd_fence,
            }
        }
    }

    pub fn swap_draw_next<Function: FnOnce(u32)>(
        &self,
        function: Function,
    ) -> Result<bool, Box<dyn Error>> {
        unsafe {
            let next_image = self.swapchain.loader.acquire_next_image(
                self.swapchain.swapchain,
                std::u64::MAX,
                self.present_complete,
                vk::Fence::null(),
            );

            let present_index = match next_image {
                Ok((present_index, _)) => present_index,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return Ok(true);
                }
                Err(error) => panic!("ERROR_AQUIRE_IMAGE -> {}", error,),
            };

            function(present_index);

            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: 1,
                p_wait_semaphores: &self.render_complete,
                swapchain_count: 1,
                p_swapchains: &self.swapchain.swapchain,
                p_image_indices: &present_index,
                ..Default::default()
            };

            let present_result = self
                .swapchain
                .loader
                .queue_present(self.present_queue, &present_info);

            match present_result {
                Ok(is_suboptimal) if is_suboptimal => {
                    return Ok(true);
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return Ok(true);
                }
                Err(error) => panic!("ERROR_PRESENT_SWAP -> {}", error,),
                _ => {}
            }

            Ok(false)
        }
    }

    pub fn wait_for_gpu(&self) -> Result<(), Box<dyn Error>> {
        unsafe { Ok(self.device.device_wait_idle().unwrap()) }
    }
}
