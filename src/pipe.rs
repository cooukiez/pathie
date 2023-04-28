use std::{
    error::Error,
    ffi::{c_void, CString},
    io::Cursor,
    mem::{self, align_of},
};

use ash::{
    util::{read_spv, Align},
    vk::{self, DescriptorSet, DescriptorSetLayout, ImageAspectFlags},
};

use crate::{
    interface::Interface,
    octree::{Light, Octree, TreeNode},
    offset_of,
    uniform::Uniform,
    Pref, DEFAULT_STORAGE_BUFFER_SIZE, DEFAULT_UNIFORM_BUFFER_SIZE,
};

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
}

pub struct ImageTarget {
    image_target: vk::Image,
    image_mem: vk::DeviceMemory,
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

    pub index_data: Vec<u32>,
    pub index_buffer: BufferSet,

    pub vertex_buffer: BufferSet,

    pub uniform_buffer: BufferSet,
    pub octree_buffer: BufferSet,
    pub light_buffer: BufferSet,

    pub descriptor_pool: vk::DescriptorPool,
    pub desc_set_layout_list: Vec<DescriptorSetLayout>,
    pub descriptor_set_list: Vec<DescriptorSet>,

    pub vert_shader: Shader,
    pub frag_shader: Shader,

    pub viewport: Vec<vk::Viewport>,
    pub scissor: Vec<vk::Rect2D>,

    pub pipe_layout_list: Vec<vk::PipelineLayout>,
    pub pipe_list: Vec<vk::Pipeline>,
}

impl Pipe {
    /// Desciptor describe some sort buffer like storage buffer.
    /// Descriptor set is group of descriptor.
    /// Specify the descriptor count for each storage type here.
    /// Uniform buffer count and storage buffer descriptor count.
    /// Max set is the max amount of set in the pool.

    pub fn create_descriptor_pool(
        uniform_desc_count: u32,
        storage_desc_count: u32,
        max_set: u32,
        interface: &Interface,
    ) -> vk::DescriptorPool {
        unsafe {
            // Specify descriptor count for each storage type
            let descriptor_size_list = [
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
    ) -> DescriptorSetLayout {
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

    /// Basic function to load all shader.
    /// Currently only load vertex & fragment shader, add
    /// other shader to be loaded here.

    pub fn load_shader(interface: &Interface) -> (Shader, Shader) {
        unsafe {
            log::info!("Getting ShaderCode ...");
            let mut vertex_spv_file = Cursor::new(&include_bytes!("../shader/vert.spv")[..]);
            let mut frag_spv_file = Cursor::new(&include_bytes!("../shader/frag.spv")[..]);

            let vertex_code = read_spv(&mut vertex_spv_file).expect("ERR_READ_VERTEX_SPV");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            let frag_code = read_spv(&mut frag_spv_file).expect("ERR_READ_FRAG_SPV");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            let vertex_shader_module = interface
                .device
                .create_shader_module(&vertex_shader_info, None)
                .expect("ERR_VERTEX_MODULE");

            let fragment_shader_module = interface
                .device
                .create_shader_module(&frag_shader_info, None)
                .expect("ERR_FRAG_MODULE");

            (
                Shader {
                    code: vertex_code,
                    module: vertex_shader_module,
                },
                Shader {
                    code: frag_code,
                    module: fragment_shader_module,
                },
            )
        }
    }

    /// Main method for creating necessary pipe list.
    /// This is the main rendering part. The interface is only for
    /// vulkan and API stuff.

    pub fn init_render(interface: &Interface, uniform: &mut Uniform, octree: &Octree) -> Self {
        unsafe {
            // Create list of target image for rendering and
            // copying to swapchain image later on.

            log::info!("Getting ImageTarget List ...");
            let image_target_list = interface
                .present_img_view_list
                .iter()
                .map(|_| ImageTarget::new(interface, interface.surface_res))
                .collect();

            // Basic index data for screen quad
            let index_data = vec![0u32, 1, 2, 2, 3, 0];
            let index_buffer = BufferSet::new(
                interface,
                align_of::<u32>() as u64,
                std::mem::size_of_val(&index_data) as u64,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &index_data,
            );

            // Basic vertex data for screen quad
            let vertex_data = vec![
                Vertex {
                    pos: [-1.0, -1.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    pos: [-1.0, 1.0, 0.0, 1.0],
                    uv: [0.0, 1.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0, 1.0],
                    uv: [1.0, 1.0],
                },
                Vertex {
                    pos: [1.0, -1.0, 0.0, 1.0],
                    uv: [1.0, 0.0],
                },
            ];

            let vertex_buffer = BufferSet::new(
                interface,
                align_of::<Vertex>() as u64,
                std::mem::size_of_val(&vertex_data) as u64,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &vertex_data,
            );

            let uniform_data: Uniform = uniform.clone();
            let uniform_buffer = BufferSet::new(
                interface,
                align_of::<Uniform>() as u64,
                mem::size_of_val(&uniform_data) as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &[uniform_data],
            );


            let octree_data = octree.data.clone();
            let octree_buffer = BufferSet::new(
                interface,
                align_of::<TreeNode>() as u64,
                DEFAULT_STORAGE_BUFFER_SIZE,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &octree_data,
            );

            let light_data = octree.light_data.clone();
            let light_buffer = BufferSet::new(
                interface,
                align_of::<Light>() as u64,
                DEFAULT_UNIFORM_BUFFER_SIZE,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &light_data,
            );;

            let descriptor_pool = Self::create_descriptor_pool(1, 2, 3, interface);

            log::info!("Creating descriptor set layout list ...");
            let desc_set_layout_list: Vec<vk::DescriptorSetLayout> = vec![
                // Uniform Set
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::UNIFORM_BUFFER,
                    1,
                    vk::ShaderStageFlags::FRAGMENT,
                    interface,
                ),
                // Octree Set
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::FRAGMENT,
                    interface,
                ),
                // Light Set
                Self::create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::FRAGMENT,
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

            let uniform_buffer_descriptor = vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer,
                offset: 0,
                range: mem::size_of_val(&uniform_data) as u64,
            };
            let octree_buffer_descriptor = vk::DescriptorBufferInfo {
                buffer: octree_buffer.buffer,
                offset: 0,
                range: (mem::size_of::<TreeNode>() * octree_data.len()) as u64,
            };
            let light_buffer_descriptor = vk::DescriptorBufferInfo {
                buffer: light_buffer.buffer,
                offset: 0,
                range: (mem::size_of::<Light>() * light_data.len()) as u64,
            };

            log::info!("Writing descriptor list ...");
            let write_desc_set_list = [
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set_list[0],
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    p_buffer_info: &uniform_buffer_descriptor,
                    ..Default::default()
                },
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set_list[1],
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                    p_buffer_info: &octree_buffer_descriptor,
                    ..Default::default()
                },
                vk::WriteDescriptorSet {
                    dst_set: descriptor_set_list[2],
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                    p_buffer_info: &light_buffer_descriptor,
                    ..Default::default()
                },
            ];

            interface
                .device
                .update_descriptor_sets(&write_desc_set_list, &[]);

            let (vert_shader, frag_shader) = Self::load_shader(interface);

            // Create graphic pipe
            let layout_create_info =
                vk::PipelineLayoutCreateInfo::builder().set_layouts(&desc_set_layout_list);

            log::info!("Creating PipelineLayout ...");
            let pipe_layout = interface
                .device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();

            log::info!("Stage Creation ...");
            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage_info_list = [
                vk::PipelineShaderStageCreateInfo {
                    module: vert_shader.module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    module: frag_shader.module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ];

            let vertex_input_binding_description_list = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];

            let vertex_input_attribute_description_list = [
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(Vertex, pos) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, uv) as u32,
                },
            ];

            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(&vertex_input_attribute_description_list)
                .vertex_binding_descriptions(&vertex_input_binding_description_list);

            let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };

            log::info!("Viewport and Scissor ...");
            let viewport = vec![vk::Viewport {
                width: interface.surface_res.width as f32,
                height: interface.surface_res.height as f32,
                max_depth: 1.0,

                ..Default::default()
            }];

            let scissor = vec![interface.surface_res.into()];
            let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
                .scissors(&scissor)
                .viewports(&viewport);

            log::info!("Rasterization ...");
            let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
                front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                polygon_mode: vk::PolygonMode::FILL,
                ..Default::default()
            };

            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            log::info!("Blending ...");
            let color_blend_attachment_state_list = [vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,

                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,

                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,

                color_write_mask: vk::ColorComponentFlags::RGBA,
            }];

            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_state_list);

            log::info!("Creating DynamicState ...");
            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

            let format_list = [interface.surface_format.format];
            let mut pipe_rendering = vk::PipelineRenderingCreateInfoKHR::builder()
                .color_attachment_formats(&format_list);

            log::info!("Pipe incoming ...");
            let pipe_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stage_info_list)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipe_layout)
                .push_next(&mut pipe_rendering)
                .build();

            let pipe_list = interface
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipe_info],
                    None,
                )
                .unwrap();

            let pipe_layout_list = vec![pipe_layout];

            log::info!("Rendering initialisation finished ...");
            Pipe {
                render_res: interface.surface_res,
                image_target_list,
                index_data,
                index_buffer,
                vertex_buffer,
                uniform_buffer,
                octree_buffer,
                light_buffer,
                descriptor_pool,
                desc_set_layout_list,
                descriptor_set_list,
                vert_shader,
                frag_shader,
                pipe_layout_list,
                viewport,
                scissor,
                pipe_list,
            }
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
                aspect_mask: ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            };
            let dst = vk::ImageSubresourceLayers {
                aspect_mask: ImageAspectFlags::COLOR,
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
                interface.draw_command_buffer,
                src_img,
                vk::ImageLayout::UNDEFINED,
                dst_img,
                vk::ImageLayout::UNDEFINED,
                &[blit],
                pref.img_filter,
            );
        }
    }

    /// Draw the next image onto the window
    /// Get swapchain image, begin draw, render with
    /// pipe onto image target and finally blit to swapchain
    /// image. Then end draw.

    pub fn draw(&self, interface: &Interface, pref: &Pref) -> Result<bool, Box<dyn Error>> {
        unsafe {
            let next_image = interface.swapchain_loader.acquire_next_image(
                interface.swapchain,
                std::u64::MAX,
                interface.present_complete_semaphore,
                vk::Fence::null(),
            );

            let present_index = match next_image {
                Ok((present_index, _)) => present_index,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return Ok(true);
                }
                Err(error) => panic!("ERROR_AQUIRE_IMAGE -> {}", error,),
            };

            let clear_value = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            }];

            interface
                .device
                .wait_for_fences(&[interface.draw_command_fence], true, std::u64::MAX)
                .expect("DEVICE_LOST");

            interface
                .device
                .reset_fences(&[interface.draw_command_fence])
                .expect("FENCE_RESET_FAILED");

            interface
                .device
                .reset_command_buffer(
                    interface.draw_command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("ERR_RESET_CMD_BUFFER");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            interface
                .device
                .begin_command_buffer(interface.draw_command_buffer, &command_buffer_begin_info)
                .expect("ERR_BEGIN_CMD_BUFFER");

            let color_attachment_info = vk::RenderingAttachmentInfoKHR::builder()
                .image_view(self.image_target_list[present_index as usize].image_view)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value[0])
                .build();

            let color_attachment_list = [color_attachment_info];

            // Begin Draw
            let rendering_info = vk::RenderingInfoKHR::builder()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.render_res,
                })
                .layer_count(1)
                .color_attachments(&color_attachment_list);

            // Pipe Rendering Part
            interface
                .device
                .cmd_begin_rendering(interface.draw_command_buffer, &rendering_info);

            interface.device.cmd_bind_descriptor_sets(
                interface.draw_command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipe_layout_list[0],
                0,
                &self.descriptor_set_list[..],
                &[],
            );

            interface.device.cmd_bind_pipeline(
                interface.draw_command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipe_list[0],
            );

            interface
                .device
                .cmd_set_viewport(interface.draw_command_buffer, 0, &self.viewport);

            interface
                .device
                .cmd_set_scissor(interface.draw_command_buffer, 0, &self.scissor);

            interface.device.cmd_bind_vertex_buffers(
                interface.draw_command_buffer,
                0,
                &[self.vertex_buffer.buffer],
                &[0],
            );

            interface.device.cmd_bind_index_buffer(
                interface.draw_command_buffer,
                self.index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );

            interface.device.cmd_draw_indexed(
                interface.draw_command_buffer,
                self.index_data.len() as u32,
                1,
                0,
                0,
                1,
            );

            interface
                .device
                .cmd_end_rendering(interface.draw_command_buffer);

            self.copy_image(
                interface,
                pref,
                self.image_target_list[present_index as usize].image_target,
                interface.present_img_list[present_index as usize],
                self.render_res,
                interface.surface_res,
            );

            // Finish Draw
            interface
                .device
                .end_command_buffer(interface.draw_command_buffer)
                .expect("ERR_END_CMD_BUFFER");

            let command_buffer_list: Vec<vk::CommandBuffer> = vec![interface.draw_command_buffer];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[interface.present_complete_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::BOTTOM_OF_PIPE])
                .command_buffers(&command_buffer_list)
                .signal_semaphores(&[interface.rendering_complete_semaphore])
                .build();

            interface
                .device
                .queue_submit(
                    interface.present_queue,
                    &[submit_info],
                    interface.draw_command_fence,
                )
                .expect("QUEUE_SUBMIT_FAILED");

            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: 1,
                p_wait_semaphores: &interface.rendering_complete_semaphore,
                swapchain_count: 1,
                p_swapchains: &interface.swapchain,
                p_image_indices: &present_index,
                ..Default::default()
            };

            let present_result = interface
                .swapchain_loader
                .queue_present(interface.present_queue, &present_info);

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

    /// Function to get the resolution
    /// at which to render. The resolution or scale factor
    /// can be changed in pref.

    pub fn get_render_res(&self, pref: &Pref, interface: &Interface) -> vk::Extent2D {
        if pref.use_render_res && interface.window.fullscreen() != None {
            pref.render_res
        } else {
            vk::Extent2D {
                width: (interface.surface_res.width as f32 / pref.img_scale) as u32,
                height: (interface.surface_res.height as f32 / pref.img_scale) as u32,
            }
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
            log::info!("Recreating Swapchain ...");
            interface.wait_for_gpu().expect("DEVICE_LOST");

            // Destroy old swapchain an img list
            self.image_target_list.iter().for_each(|target| {
                target.destroy(interface);
            });

            interface
                .present_img_view_list
                .iter()
                .for_each(|view| interface.device.destroy_image_view(*view, None));
            interface
                .swapchain_loader
                .destroy_swapchain(interface.swapchain, None);

            // Get new surface info about format and more
            (
                _,
                interface.surface_capa,
                _,
                interface.surface_res,
                _,
                interface.present_mode_list,
                interface.present_mode,
            ) = Interface::load_surface(
                &interface.surface_loader,
                interface.phy_device,
                interface.surface,
                pref,
            );

            self.render_res = self.get_render_res(pref, interface);
            uniform.apply_resolution(self.render_res);

            interface.swapchain = Interface::create_swapchain(
                interface.surface,
                interface.img_count,
                &interface.surface_format,
                interface.surface_res,
                interface.pre_transform,
                interface.present_mode,
                &interface.swapchain_loader,
            );

            (interface.present_img_list, interface.present_img_view_list) =
                Interface::load_present_img_list(
                    &interface.swapchain_loader,
                    interface.swapchain,
                    &interface.surface_format,
                    &interface.device,
                );

            // Create new image target list because size of image has changed
            self.image_target_list = interface
                .present_img_view_list
                .iter()
                .map(|_| ImageTarget::new(interface, self.render_res))
                .collect();

            log::info!("Viewport and Scissor ...");
            self.viewport = vec![vk::Viewport {
                width: self.render_res.width as f32,
                height: self.render_res.height as f32,
                max_depth: 1.0,
                ..Default::default()
            }];

            self.scissor = vec![self.render_res.into()];
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
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
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
                image_view,
            }
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
}
