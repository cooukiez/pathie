use std::{
    error::Error,
    ffi::{c_void, CString},
    io::Cursor,
    mem::{self, align_of},
};

use ash::{
    util::{read_spv, Align},
    vk
};

use crate::{
    interface::Interface,
    octree::{Light, Octree, TreeNode},
    uniform::Uniform,
    Pref, DEFAULT_STORAGE_BUFFER_SIZE, DEFAULT_UNIFORM_BUFFER_SIZE,
};

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
}

pub struct ImageTarget {
    image_target: vk::Image,
    image_mem: vk::DeviceMemory,
    sampler: vk::Sampler,
    image_view: vk::ImageView,
}

pub struct BufferSet {
    pub buffer: vk::Buffer,
    pub buffer_mem: vk::DeviceMemory,
}

pub struct Shader {
    code: Vec<u32>,
    module: vk::ShaderModule,
}

pub struct Pipe {
    pub render_res: vk::Extent2D,
    pub image_target_list: Vec<ImageTarget>,

    pub uniform_buffer: BufferSet,
    pub octree_buffer: BufferSet,
    pub light_buffer: BufferSet,

    pub descriptor_pool: vk::DescriptorPool,
    pub desc_set_layout_list: Vec<vk::DescriptorSetLayout>,
    pub descriptor_set_list: Vec<vk::DescriptorSet>,

    pub primary_ray_code: Vec<u32>,
    pub primary_ray_module: vk::ShaderModule,

    pub pipe_layout: vk::PipelineLayout,
    pub pipe: vk::Pipeline,
}

impl Pipe {
    pub fn init(interface: &Interface, pref: &Pref, uniform: &mut Uniform, octree: &Octree) -> Self {
        unsafe {
            let surface_capa = interface.surface_loader
                .get_physical_device_surface_capabilities(interface.phy_device, interface.surface)
                .unwrap();

            let (_, render_res) = Interface::get_res(&interface.window, pref, &surface_capa);
            uniform.apply_resolution(render_res);

            log::info!("Getting ImageTarget List ...");
            let image_target_list = interface
                .present_img_view_list
                .iter()
                .map(|_| ImageTarget::new(interface, render_res))
                .collect();

            log::info!("Creating UniformBuffer ...");
            let uniform_data = uniform.clone();
            let uniform_buffer = BufferSet::new(
                interface,
                align_of::<Uniform>() as u64,
                mem::size_of_val(&uniform_data) as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &[uniform_data],
            );

            log::info!("Creating OctreeBuffer ...");
            let octree_data = octree.node_data.clone();
            let octree_buffer = BufferSet::new(
                interface,
                align_of::<TreeNode>() as u64,
                DEFAULT_STORAGE_BUFFER_SIZE,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &octree_data,
            );

            log::info!("Creating LightingBuffer ...");
            let light_data = octree.light_data.clone();
            let light_buffer = BufferSet::new(
                interface,
                align_of::<Light>() as u64,
                DEFAULT_UNIFORM_BUFFER_SIZE,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &light_data,
            );

            let descriptor_pool = Self::create_descriptor_pool(1, 1, 2, 4, interface);

            log::info!("Creating descriptor set layout list ...");
            let desc_set_layout_list: Vec<vk::DescriptorSetLayout> = vec![
                // ImageTarget
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_IMAGE,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    interface,
                ),
                // Uniform Set
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::UNIFORM_BUFFER,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    interface,
                ),
                // Octree Set
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    interface,
                ),
                // Light Set
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    interface,
                ),
            ];

            let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&desc_set_layout_list);

            let descriptor_set_list = interface
                .device
                .allocate_descriptor_sets(&desc_alloc_info)
                .unwrap();

            log::info!("Writing descriptor list ...");
            uniform_buffer.describe_in_gpu(
                interface,
                mem::size_of_val(&uniform_data) as u64,
                descriptor_set_list[1],
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
            );
            octree_buffer.describe_in_gpu(
                interface,
                (mem::size_of::<TreeNode>() * octree_data.len()) as u64,
                descriptor_set_list[2],
                0,
                vk::DescriptorType::STORAGE_BUFFER,
            );
            light_buffer.describe_in_gpu(
                interface,
                (mem::size_of::<Light>() * light_data.len()) as u64,
                descriptor_set_list[3],
                0,
                vk::DescriptorType::STORAGE_BUFFER,
            );

            log::info!("Getting ShaderCode ...");
            let mut primary_ray_spv_file = Cursor::new(&include_bytes!("../shader/comp.spv")[..]);

            let primary_ray_code =
                read_spv(&mut primary_ray_spv_file).expect("ERR_READ_VERTEX_SPV");
            let primary_ray_info = vk::ShaderModuleCreateInfo::builder().code(&primary_ray_code);

            let primary_ray_module = interface
                .device
                .create_shader_module(&primary_ray_info, None)
                .expect("ERR_VERTEX_MODULE");

            let layout_create_info =
                vk::PipelineLayoutCreateInfo::builder().set_layouts(&desc_set_layout_list);

            log::info!("Creating PipelineLayout ...");
            let pipe_layout = interface
                .device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();

            log::info!("Stage Creation ...");
            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage = vk::PipelineShaderStageCreateInfo {
                module: primary_ray_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            };

            let compute_pipe_info = vk::ComputePipelineCreateInfo::builder()
                .stage(shader_stage)
                .layout(pipe_layout)
                .build();

            let pipe = interface
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &[compute_pipe_info], None)
                .expect("ERROR_CREATE_PIPELINE")[0];

            log::info!("Rendering initialisation finished ...");
            Pipe {
                render_res,
                image_target_list,
                uniform_buffer,
                octree_buffer,
                light_buffer,
                descriptor_pool,
                desc_set_layout_list,
                descriptor_set_list,
                primary_ray_code,
                primary_ray_module,
                pipe_layout,
                pipe,
            }
        }
    }

    /// Desciptor describe some sort buffer like storage buffer.
    /// Descriptor set is group of descriptor.
    /// Specify the descriptor count for each storage type here.
    /// Uniform buffer count and storage buffer descriptor count.
    /// Max set is the max amount of set in the pool.

    pub fn create_descriptor_pool(
        image_desc_count: u32,
        uniform_desc_count: u32,
        storage_desc_count: u32,
        max_set: u32,
        interface: &Interface,
    ) -> vk::DescriptorPool {
        unsafe {
            // Specify descriptor count for each storage type
            let descriptor_size_list = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: image_desc_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: uniform_desc_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: storage_desc_count,
                },
            ];

            log::info!("Creating DescriptorPool ...");
            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&descriptor_size_list)
                .max_sets(max_set);

            interface
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .unwrap()
        }
    }

    /// Create descriptor set which is group of descriptor.
    /// Specify the type and count, could cause error if more used than
    /// expect in pool creation. Same goes for descriptor set. If set count
    /// is bigger than max set, it will throw an error.

    pub fn create_descriptor_set_layout(
        descriptor_type: vk::DescriptorType,
        descriptor_count: u32,
        shader_stage: vk::ShaderStageFlags,
        interface: &Interface,
    ) -> vk::DescriptorSetLayout {
        unsafe {
            log::info!("Creating DescriptorSet ...");
            let set_binding_info = [vk::DescriptorSetLayoutBinding {
                descriptor_type,
                descriptor_count,
                stage_flags: shader_stage,
                ..Default::default()
            }];

            let desc_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&set_binding_info);

            interface
                .device
                .create_descriptor_set_layout(&desc_info, None)
                .unwrap()
        }
    }

    /// Submit command buffer with
    /// sync setup. With draw command buffer and
    /// present queue.

    pub fn record_submit_cmd<Function: FnOnce(vk::CommandBuffer)>(
        &self,
        interface: &Interface,
        draw_cmd_fence: vk::Fence,
        draw_cmd_buffer: vk::CommandBuffer,
        present_complete: vk::Semaphore,
        render_complete: vk::Semaphore,
        function: Function,
    ) {
        unsafe {
            interface
                .device
                .wait_for_fences(&[draw_cmd_fence], true, std::u64::MAX)
                .expect("DEVICE_LOST");

            interface
                .device
                .reset_fences(&[draw_cmd_fence])
                .expect("FENCE_RESET_FAILED");

            interface
                .device
                .reset_command_buffer(
                    draw_cmd_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("ERR_RESET_CMD_BUFFER");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            interface
                .device
                .begin_command_buffer(draw_cmd_buffer, &command_buffer_begin_info)
                .expect("ERR_BEGIN_CMD_BUFFER");

            function(draw_cmd_buffer);

            interface
                .device
                .end_command_buffer(draw_cmd_buffer)
                .expect("ERR_END_CMD_BUFFER");

            let submit_info = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::BOTTOM_OF_PIPE])
                .wait_semaphores(&[present_complete])
                .command_buffers(&[draw_cmd_buffer])
                .signal_semaphores(&[render_complete])
                .build();

            interface
                .device
                .queue_submit(interface.present_queue, &[submit_info], draw_cmd_fence)
                .expect("QUEUE_SUBMIT_FAILED");
        }
    }

    pub fn first_img_barrier(
        &self,
        image: &ImageTarget,
        present_image: vk::Image,
        interface: &Interface,
        cmd_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            let basic_subresource_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };

            let comp_write = vk::ImageMemoryBarrier::builder()
                .image(image.image_target)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .subresource_range(basic_subresource_range.clone())
                .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
                .build();

            let comp_transfer = vk::ImageMemoryBarrier::builder()
                .image(image.image_target)
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .subresource_range(basic_subresource_range.clone())
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .build();

            let swap_transfer = vk::ImageMemoryBarrier::builder()
                .image(present_image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .subresource_range(basic_subresource_range.clone())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .build();

            interface.device.cmd_pipeline_barrier(
                cmd_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[comp_write, comp_transfer, swap_transfer],
            )
        }
    }

    /// Function for blitting one image to another image with possibile
    /// scaling implemented. This function is for fast usage
    /// and not for changing the copy setting.

    pub fn copy_image(
        &self,
        interface: &Interface,
        pref: &Pref,
        src_img: vk::Image,
        dst_img: vk::Image,
        src_res: vk::Extent2D,
        dst_res: vk::Extent2D,
    ) {
        unsafe {
            let src = vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            };
            let dst = vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            };

            let blit = vk::ImageBlit {
                src_subresource: src,
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: src_res.width as i32,
                        y: src_res.height as i32,
                        z: 1,
                    },
                ],
                dst_subresource: dst,
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: dst_res.width as i32,
                        y: dst_res.height as i32,
                        z: 1,
                    },
                ],
            };

            interface.device.cmd_blit_image(
                interface.draw_cmd_buffer,
                src_img,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst_img,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                pref.img_filter,
            );
        }
    }

    pub fn sec_img_barrier(
        &self,
        present_image: vk::Image,
        interface: &Interface,
        cmd_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            let basic_subresource_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };

            let swap_present = vk::ImageMemoryBarrier::builder()
                .image(present_image)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .subresource_range(basic_subresource_range.clone())
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .build();

            interface.device.cmd_pipeline_barrier(
                cmd_buffer,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[swap_present],
            )
        }
    }

    /// Draw the next image onto the window
    /// Get swapchain image, begin draw, render with
    /// pipe onto image target and finally blit to swapchain
    /// image. Then end draw.

    pub fn draw(
        &self,
        interface: &Interface,
        pref: &Pref,
        uniform: &Uniform,
    ) -> Result<bool, Box<dyn Error>> {
        unsafe {
            interface.swap_draw_next(|present_index| {
                self.record_submit_cmd(
                    interface,
                    interface.draw_cmd_fence,
                    interface.draw_cmd_buffer,
                    interface.present_complete,
                    interface.render_complete,
                    |cmd_buffer| {
                        self.image_target_list[present_index as usize].describe_in_gpu(
                            interface,
                            vk::ImageLayout::GENERAL,
                            self.descriptor_set_list[0],
                            0,
                            vk::DescriptorType::STORAGE_IMAGE,
                        );
                        self.uniform_buffer.describe_in_gpu(
                            interface,
                            mem::size_of_val(uniform) as u64,
                            self.descriptor_set_list[1],
                            0,
                            vk::DescriptorType::UNIFORM_BUFFER,
                        );

                        // Dispatch Compute Pipe
                        interface.device.cmd_bind_pipeline(
                            cmd_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            self.pipe,
                        );
                        interface.device.cmd_bind_descriptor_sets(
                            cmd_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            self.pipe_layout,
                            0,
                            &self.descriptor_set_list[..],
                            &[],
                        );
                        interface.device.cmd_dispatch(
                            cmd_buffer,
                            self.render_res.width / 16,
                            self.render_res.height / 16,
                            1,
                        );

                        // First Image Barrier
                        self.first_img_barrier(
                            &self.image_target_list[present_index as usize],
                            interface.present_img_list[present_index as usize],
                            interface,
                            cmd_buffer,
                        );
                        // Copy image memory
                        self.copy_image(
                            interface,
                            pref,
                            self.image_target_list[present_index as usize].image_target,
                            interface.present_img_list[present_index as usize],
                            self.render_res,
                            interface.surface_res,
                        );
                        self.sec_img_barrier(
                            interface.present_img_list[present_index as usize],
                            interface,
                            cmd_buffer,
                        );
                    },
                );
            })
        }
    }

    /// This function is called when the swapchain is outdated
    /// or has the wrong size basically whenever you change the window
    /// size or just minimize the window.

    pub fn recreate_swapchain(
        &mut self,
        interface: &mut Interface,
        uniform: &mut Uniform,
        pref: &Pref,
    ) {
        unsafe {
            interface.wait_for_gpu().expect("DEVICE_LOST");

            log::info!("Recreating Swapchain ...");

            // Destroy Image Target
            self.image_target_list.iter().for_each(|target| {
                target.destroy(interface);
            });

            // Destroy Swapchain and SwapchainImgList
            interface
                .present_img_view_list
                .iter()
                .for_each(|view| interface.device.destroy_image_view(*view, None));
            interface
                .swapchain_loader
                .destroy_swapchain(interface.swapchain, None);

            // New SurfaceCapability
            let surface_capa = interface
                .surface_loader
                .get_physical_device_surface_capabilities(interface.phy_device, interface.surface)
                .unwrap();

            (interface.surface_res, self.render_res) =
                Interface::get_res(&interface.window, pref, &surface_capa);

            uniform.apply_resolution(self.render_res);

            // Select PresentMode -> PreferredPresentMode is selected in Pref
            let present_mode = interface
                .present_mode_list
                .iter()
                .cloned()
                .find(|&mode| mode == pref.pref_present_mode)
                .unwrap_or(vk::PresentModeKHR::FIFO);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(interface.surface)
                .min_image_count(interface.swap_img_count)
                .image_color_space(interface.surface_format.color_space)
                .image_format(interface.surface_format.format)
                .image_extent(interface.surface_res)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(interface.pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);
            interface.swapchain = interface
                .swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();

            interface.present_img_list = interface
                .swapchain_loader
                .get_swapchain_images(interface.swapchain)
                .unwrap();

            interface.present_img_view_list = interface
                .present_img_list
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(interface.surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);
                    interface
                        .device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                })
                .collect();

            self.image_target_list = interface
                .present_img_view_list
                .iter()
                .map(|_| ImageTarget::new(interface, self.render_res))
                .collect();
        }
    }
}

impl ImageTarget {
    pub fn new(interface: &Interface, extent: vk::Extent2D) -> Self {
        unsafe {
            // Create ImgInfo with Dimension -> Scaled Variant
            let image_info = vk::ImageCreateInfo::builder()
                .format(interface.surface_format.format)
                .extent(vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .image_type(vk::ImageType::TYPE_2D)
                .build();

            // Create Image on Device
            let image_target = interface.device.create_image(&image_info, None).unwrap();

            // Get Memory Requirement for Image and the MemoryTypeIndex
            let img_mem_requirement = interface.device.get_image_memory_requirements(image_target);
            let img_mem_index = interface
                .find_memorytype_index(&img_mem_requirement, vk::MemoryPropertyFlags::DEVICE_LOCAL)
                .expect("NO_SUITABLE_MEM_TYPE_INDEX");

            // Prepare MemoryAllocation
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(img_mem_requirement.size)
                .memory_type_index(img_mem_index)
                .build();

            // Allocate Memory therefore create DeviceMemory
            let image_mem = interface
                .device
                .allocate_memory(&allocate_info, None)
                .unwrap();

            // To Finish -> Bind Memory
            interface
                .device
                .bind_image_memory(image_target, image_mem, 0)
                .expect("UNABLE_TO_BIND_MEM");

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
                .mip_lod_bias(0.0)
                .max_anisotropy(1.0)
                .compare_op(vk::CompareOp::NEVER)
                .min_lod(0.0)
                .max_lod(1.0)
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

            let sampler = interface.device.create_sampler(&sampler_info, None).unwrap();

            // Prepare ImageView Creation and bind Image
            let image_view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(interface.surface_format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image_target)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .build();

            // Build Image View
            let image_view = interface
                .device
                .create_image_view(&image_view_info, None)
                .unwrap();

            Self {
                image_target,
                image_mem,
                sampler,
                image_view,
            }
        }
    }

    pub fn describe_in_gpu(
        &self,
        interface: &Interface,
        image_layout: vk::ImageLayout,
        dst_set: vk::DescriptorSet,
        dst_binding: u32,
        descriptor_type: vk::DescriptorType,
    ) {
        unsafe {
            let image_descriptor = vk::DescriptorImageInfo {
                image_view: self.image_view,
                image_layout,
                sampler: self.sampler,
            };

            let write_info = vk::WriteDescriptorSet {
                dst_set,
                dst_binding,
                descriptor_count: 1,
                descriptor_type,
                p_image_info: &image_descriptor,
                ..Default::default()
            };

            interface.device.update_descriptor_sets(&[write_info], &[]);
        }
    }

    pub fn destroy(&self, interface: &Interface) {
        unsafe {
            interface.device.destroy_image_view(self.image_view, None);
            interface.device.destroy_image(self.image_target, None);
        }
    }
}

impl BufferSet {
    pub fn new<Type: Copy>(
        interface: &Interface,
        alignment: u64,
        size: u64,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        data: &[Type],
    ) -> Self {
        unsafe {
            // BufferInfo
            let buffer_info = vk::BufferCreateInfo {
                size,
                usage,
                sharing_mode,

                ..Default::default()
            };

            // Create BufferObject
            let buffer = interface.device.create_buffer(&buffer_info, None).unwrap();

            // Get MemoryRequirement
            let memory_req = interface.device.get_buffer_memory_requirements(buffer);
            let memory_index = interface
                .find_memorytype_index(
                    &memory_req,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .expect("ERR_INDEX_BUFFER_MEM_INDEX");

            // Prepare Allocation
            let allocate_info = vk::MemoryAllocateInfo {
                allocation_size: memory_req.size,
                memory_type_index: memory_index,

                ..Default::default()
            };

            // Create MemoryObject
            let buffer_mem = interface
                .device
                .allocate_memory(&allocate_info, None)
                .unwrap();

            // Prepare MemoryCopy
            let index_ptr: *mut c_void = interface
                .device
                .map_memory(buffer_mem, 0, memory_req.size, vk::MemoryMapFlags::empty())
                .unwrap();
            let mut index_slice = Align::new(index_ptr, alignment, memory_req.size);

            // Copy and finish Memory
            index_slice.copy_from_slice(&data);
            interface.device.unmap_memory(buffer_mem);

            interface
                .device
                .bind_buffer_memory(buffer, buffer_mem, 0)
                .unwrap();

            Self { buffer, buffer_mem }
        }
    }

    pub fn update<Type: Copy>(&self, interface: &Interface, data: &[Type]) {
        unsafe {
            let buffer_ptr = interface
                .device
                .map_memory(
                    self.buffer_mem,
                    0,
                    std::mem::size_of_val(data) as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();

            let mut aligned_slice = Align::new(
                buffer_ptr,
                align_of::<Type>() as u64,
                std::mem::size_of_val(data) as u64,
            );

            aligned_slice.copy_from_slice(&data.clone()[..]);

            interface.device.unmap_memory(self.buffer_mem);
        }
    }

    pub fn describe_in_gpu(
        &self,
        interface: &Interface,
        range: u64,
        dst_set: vk::DescriptorSet,
        dst_binding: u32,
        descriptor_type: vk::DescriptorType,
    ) {
        unsafe {
            let buffer_descriptor = vk::DescriptorBufferInfo {
                buffer: self.buffer,
                offset: 0,
                range,
            };

            let write_info = vk::WriteDescriptorSet {
                dst_set,
                dst_binding,
                descriptor_count: 1,
                descriptor_type,
                p_buffer_info: &buffer_descriptor,
                ..Default::default()
            };

            interface.device.update_descriptor_sets(&[write_info], &[]);
        }
    }
}
