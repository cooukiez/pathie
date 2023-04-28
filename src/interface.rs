use crate::Pref;
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{DynamicRendering, Surface, Swapchain},
    },
    vk::{self, MemoryPriorityAllocateInfoEXT, SurfaceTransformFlagsKHR},
    Device, Entry, Instance,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{
    error::Error,
    ffi::{c_char, c_void, CStr, CString},
};
use winit::{event_loop::EventLoop, monitor::MonitorHandle, window::WindowBuilder};

pub struct Interface {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_util_loader: DebugUtils,
    pub window: winit::window::Window,
    pub monitor_list: Vec<MonitorHandle>,
    pub monitor: MonitorHandle,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,

    pub phy_device: vk::PhysicalDevice,
    pub phy_device_prop: vk::PhysicalDeviceProperties,
    pub phy_device_memory_prop: vk::PhysicalDeviceMemoryProperties,
    pub phy_device_feature: vk::PhysicalDeviceFeatures,

    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface_capa: vk::SurfaceCapabilitiesKHR,
    pub pre_transform: SurfaceTransformFlagsKHR,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_res: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub img_count: u32,
    pub present_mode_list: Vec<vk::PresentModeKHR>,
    pub present_mode: vk::PresentModeKHR,

    pub present_img_list: Vec<vk::Image>,
    pub present_img_view_list: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub command_buffer_list: Vec<vk::CommandBuffer>,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub draw_command_fence: vk::Fence,
    pub setup_command_fence: vk::Fence,
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
        Flag::VERBOSE => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
        Flag::INFO => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
        Flag::WARNING => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
        _ => log::debug!("[ {:?} ] {}", msg_type, message.to_str().unwrap(),),
    }

    return vk::FALSE;
}

impl Interface {
    /// Get the queue and device with phy device.
    /// Note that device is the so called logical device.
    /// Also note that queue priority is usually one.

    pub fn get_device_and_queue(
        queue_family_index: u32,
        priority: &[f32],
        device_ext_list: &[*const c_char],
        feature: vk::PhysicalDeviceFeatures,
        instance: &Instance,
        phy_device: vk::PhysicalDevice,
    ) -> (Device, vk::Queue) {
        unsafe {
            log::info!("Get QueueList ...");
            // Queue info with index and priority
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(priority);

            // Dynamic rendering is used later on
            let mut dynamic_rendering_feature =
                vk::PhysicalDeviceDynamicRenderingFeaturesKHR::builder().dynamic_rendering(true);

            // Create device info with predefined ext list and dynamic rendering as addition
            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(device_ext_list)
                .enabled_features(&feature)
                .push_next(&mut dynamic_rendering_feature);

            let device: Device = instance
                .create_device(phy_device, &device_create_info, None)
                .unwrap();

            let present_queue = device.get_device_queue(queue_family_index, 0);

            (device, present_queue)
        }
    }

    /// Load surface or more like get info about surface.
    /// This funciton is necessary for swapchain creation because
    /// it does require info about the surface.
    /// Note -> PreTransform is for rotation.
    /// Note -> the int is the desired image count which is usally three
    /// Note -> List of all present mode available returned

    pub fn load_surface(
        surface_loader: &Surface,
        phy_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        pref: &Pref,
    ) -> (
        vk::SurfaceFormatKHR,
        vk::SurfaceCapabilitiesKHR,
        u32,
        vk::Extent2D,
        SurfaceTransformFlagsKHR,
        Vec<vk::PresentModeKHR>,
        vk::PresentModeKHR,
    ) {
        unsafe {
            log::info!("Load Surface ...");

            // Surface format like which RGB channel type and whatsoever
            let surface_format = surface_loader
                .get_physical_device_surface_formats(phy_device, surface)
                .unwrap()[0];

            // What can your surface do?
            let surface_capa = surface_loader
                .get_physical_device_surface_capabilities(phy_device, surface)
                .unwrap();

            // Often -> Desired image count = 3
            let mut img_count = surface_capa.min_image_count + 1;
            if surface_capa.max_image_count > 0 && img_count > surface_capa.max_image_count {
                img_count = surface_capa.max_image_count;
            }

            // Surface resolution
            let surface_res = match surface_capa.current_extent.width {
                std::u32::MAX => pref.start_window_size,
                _ => surface_capa.current_extent,
            };

            // Rotate screen, mostly used for smartphone app
            let pre_transform = if surface_capa
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capa.current_transform
            };

            // Present mode list of all available
            let present_mode_list = surface_loader
                .get_physical_device_surface_present_modes(phy_device, surface)
                .unwrap();

            // Select present mode based on preferred present mode
            let present_mode = present_mode_list
                .iter()
                .cloned()
                .find(|&mode| mode == pref.pref_present_mode)
                .unwrap_or(vk::PresentModeKHR::FIFO);

            (
                surface_format,
                surface_capa,
                img_count,
                surface_res,
                pre_transform,
                present_mode_list,
                present_mode,
            )
        }
    }

    /// Function for creating swapchain
    /// First get loader and then all the prop. After that
    /// create with loader.

    pub fn create_swapchain(
        surface: vk::SurfaceKHR,
        img_count: u32,
        surface_format: &vk::SurfaceFormatKHR,
        surface_res: vk::Extent2D,
        pre_transform: vk::SurfaceTransformFlagsKHR,
        present_mode: vk::PresentModeKHR,
        swapchain_loader: &Swapchain,
    ) -> vk::SwapchainKHR {
        unsafe {
            log::info!("Creating Swapchain ...");

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface)
                .min_image_count(img_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_res)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        }
    }

    /// Create command pool for vulkan instance.
    /// First create pool then allocate command buffer list
    /// with setup and draw command buffer.

    pub fn create_command_pool(
        queue_family_index: u32,
        device: &Device,
    ) -> (
        vk::CommandPool,
        Vec<vk::CommandBuffer>,
        vk::CommandBuffer,
        vk::CommandBuffer,
    ) {
        unsafe {
            log::info!("Creating CommandPool ...");
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            // Create command pool
            let pool = device.create_command_pool(&pool_create_info, None).unwrap();

            // Allocate command buffer list
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            log::info!("Creating CommandBuffer ...");
            let command_buffer_list = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();

            let setup_command_buffer = command_buffer_list[0];
            let draw_command_buffer = command_buffer_list[1];

            (
                pool,
                command_buffer_list,
                setup_command_buffer,
                draw_command_buffer,
            )
        }
    }

    /// Load the present image list.
    /// Load img from swapchain and then create image view,
    /// pointer to each image, for each image from swapchain.
    /// These can be used to render onto.

    pub fn load_present_img_list(
        swapchain_loader: &Swapchain,
        swapchain: vk::SwapchainKHR,
        surface_format: &vk::SurfaceFormatKHR,
        device: &Device,
    ) -> (Vec<vk::Image>, Vec<vk::ImageView>) {
        unsafe {
            log::info!("Load PresentImgList ...");
            let present_img_list = swapchain_loader.get_swapchain_images(swapchain).unwrap();
            let present_img_view_list: Vec<vk::ImageView> = present_img_list
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        // Change image channel here
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        // Change img range here
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);

                    device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect();

            (present_img_list, present_img_view_list)
        }
    }

    /// This function is for initializing the sync for render the render pipe.
    /// Whe first create setup and draw fence. After that the present and render
    /// semaphore. This may be expanded later on.

    pub fn init_sync(device: &Device) -> (vk::Fence, vk::Fence, vk::Semaphore, vk::Semaphore) {
        unsafe {
            log::info!("Init Fence ...");
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            // Create fence
            let setup_command_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");
            let draw_command_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");

            log::info!("Init Semaphore ...");
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            // Create semaphore
            let present_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            (
                setup_command_fence,
                draw_command_fence,
                present_complete_semaphore,
                rendering_complete_semaphore,
            )
        }
    }

    /// Initialize vulkan instance with pref.
    /// Return an instance object with all var initialized.
    /// This will create the base for vulkan application upto fence and
    /// semaphore creation without creating pipeline.

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

            let surface_loader = Surface::new(&entry, &instance);

            let surface = ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .unwrap();

            let phy_device_list = instance
                .enumerate_physical_devices()
                .expect("ERR_NO_PHY_DEVICE");

            let (phy_device, queue_family_index) = phy_device_list
                .iter()
                .find_map(|phy_device| {
                    instance
                        .get_physical_device_queue_family_properties(*phy_device)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let graphic_surface_support =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                        .get_physical_device_surface_support(
                                            *phy_device,
                                            index as u32,
                                            surface,
                                        )
                                        .unwrap();
                            if graphic_surface_support {
                                Some((*phy_device, index))
                            } else {
                                None
                            }
                        })
                })
                .expect("NO_SUITABLE_PHY_DEVICE");

            let phy_device_prop = instance.get_physical_device_properties(phy_device);
            let phy_device_memory_prop = instance.get_physical_device_memory_properties(phy_device);
            let phy_device_feature = instance.get_physical_device_features(phy_device);

            let device_ext_list = [
                Swapchain::name().as_ptr(),
                DynamicRendering::name().as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios",))]
                KhrPortabilitySubsetFn::name().as_ptr(),
            ];

            let feature = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };

            let queue_family_index = queue_family_index as u32;
            let priority = [1.0];

            log::info!("Get QueueList ...");
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priority);

            let mut dynamic_rendering_feature = vk::PhysicalDeviceDynamicRenderingFeaturesKHR::builder().dynamic_rendering(true);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_ext_list)
                .enabled_features(&feature)
                .push_next(&mut dynamic_rendering_feature);

            let device: Device = instance
                .create_device(phy_device, &device_create_info, None, )
                .unwrap();

            let present_queue = device.get_device_queue(queue_family_index, 0);

            let (
                surface_format,
                surface_capa,
                img_count,
                surface_res,
                pre_transform,
                present_mode_list,
                present_mode,
            ) = Self::load_surface(&surface_loader, phy_device, surface, pref);

            let swapchain_loader = Swapchain::new(&instance, &device);
            let swapchain = Self::create_swapchain(
                surface,
                img_count,
                &surface_format,
                surface_res,
                pre_transform,
                present_mode,
                &swapchain_loader,
            );

            log::info!("Creating CommandPool ...");
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let pool = device.create_command_pool(&pool_create_info, None).unwrap();

            log::info!("Creating CommandBuffer ...");
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffer_list = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();

            let setup_command_buffer = command_buffer_list[0];
            let draw_command_buffer = command_buffer_list[1];

            let (present_img_list, present_img_view_list) =
                Self::load_present_img_list(&swapchain_loader, swapchain, &surface_format, &device);

            log::info!("Init Fence ...");
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_command_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");
            let setup_command_fence = device
                .create_fence(&fence_create_info, None)
                .expect("FENCE_CREATE_ERR");

            log::info!("Init Semaphore ...");
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();
            let rendering_complete_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap();

            log::info!("Interface finished ...");
            Interface {
                entry,
                instance,
                device,
                surface_loader,
                swapchain_loader,
                debug_util_loader,
                window,
                monitor_list,
                monitor,
                debug_call_back,

                phy_device,
                phy_device_prop,
                phy_device_memory_prop,
                phy_device_feature,

                queue_family_index,
                present_queue,

                surface_capa,
                pre_transform,

                surface,
                surface_format,
                surface_res,

                swapchain,
                img_count,
                present_mode_list,
                present_mode,

                present_img_list,
                present_img_view_list,

                pool,
                command_buffer_list,
                draw_command_buffer,
                setup_command_buffer,

                present_complete_semaphore,
                rendering_complete_semaphore,

                draw_command_fence,
                setup_command_fence,
            }
        }
    }

    /// Function for creating memory. Find suitable
    /// type index for memory req. Evaluate all available and then
    /// select suitable type and return index.

    pub fn find_memorytype_index(
        &self,
        memory_req: &vk::MemoryRequirements,
        flag: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        // Get all available
        self.phy_device_memory_prop.memory_types
            [..self.phy_device_memory_prop.memory_type_count as _]
            .iter()
            .enumerate()
            // Find suitable type
            .find(|(index, memory_type)| {
                (1 << index) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flag == flag
            })
            .map(|(index, _)| index as _)
    }

    pub fn wait_for_gpu(&self) -> Result<(), Box<dyn Error>> {
        unsafe { Ok(self.device.device_wait_idle().unwrap()) }
    }
}
