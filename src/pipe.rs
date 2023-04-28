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
struct Vertex {
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

    pub vertex_code: Vec<u32>,
    pub frag_code: Vec<u32>,
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub pipeline_layout: vk::PipelineLayout,

    pub viewport: Vec<vk::Viewport>,
    pub scissor: Vec<vk::Rect2D>,
    pub graphic_pipeline: vk::Pipeline,
}

impl Pipe {
    pub fn init(interface: &Interface, uniform: &mut Uniform, octree: &Octree) -> Self {
        unsafe {
            let render_res = interface.surface_res;
            uniform.apply_resolution(render_res);

            log::info!("Getting ImageTarget List ...");
            let image_target_list = interface
                .present_img_view_list
                .iter()
                .map(|_| ImageTarget::new(interface, render_res))
                .collect();

            log::info!("Creating IndexBuffer ...");
            let index_data = vec![0u32, 1, 2, 2, 3, 0];
            let index_buffer = BufferSet::new(
                interface,
                align_of::<u32>() as u64,
                std::mem::size_of_val(&index_data) as u64,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &index_data,
            );

            log::info!("Creating VertexBuffer ...");
            let vertex_data = [
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
            let octree_data = octree.data.clone();
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

            log::info!("Creating DescriptorPool ...");
            let descriptor_size_list = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                }, // Uniform
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                }, // Octree - NodeData
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                }, // Octree - LightData
            ];

            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&descriptor_size_list)
                .max_sets(descriptor_size_list.len() as u32);
            let descriptor_pool = interface
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .unwrap();

            log::info!("Creating UniformSet ...");
            let uniform_set_binding_list = [vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }];

            log::info!("Creating OctreeSet ...");
            let octree_set_binding_list = [vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }];

            log::info!("Creating LightingSet ...");
            let light_set_binding_list = [vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }];

            let uniform_desc_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&uniform_set_binding_list);
            let octree_desc_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&octree_set_binding_list);
            let light_desc_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&light_set_binding_list);

            log::info!("Allocating whole DescriptorPool ...");
            let desc_set_layout_list: Vec<vk::DescriptorSetLayout> = vec![
                interface
                    .device
                    .create_descriptor_set_layout(&uniform_desc_info, None)
                    .unwrap(),
                interface
                    .device
                    .create_descriptor_set_layout(&octree_desc_info, None)
                    .unwrap(),
                interface
                    .device
                    .create_descriptor_set_layout(&light_desc_info, None)
                    .unwrap(),
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

            log::info!("Writing whole DescriptorPool ...");
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

            let layout_create_info =
                vk::PipelineLayoutCreateInfo::builder().set_layouts(&desc_set_layout_list);

            log::info!("Creating PipelineLayout ...");
            let pipeline_layout = interface
                .device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();

            log::info!("Stage Creation ...");
            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage_info_list = [
                vk::PipelineShaderStageCreateInfo {
                    module: vertex_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    module: fragment_shader_module,
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
                width: render_res.width as f32,
                height: render_res.height as f32,
                max_depth: 1.0,

                ..Default::default()
            }];

            let scissor = vec![render_res.into()];
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
            let mut pipeline_rendering = vk::PipelineRenderingCreateInfoKHR::builder()
                .color_attachment_formats(&format_list);

            log::info!("Pipe incoming ...");
            let graphic_pipeline_info_list = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stage_info_list)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .push_next(&mut pipeline_rendering)
                .build();

            let graphic_pipeline_list = interface
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info_list],
                    None,
                )
                .unwrap();

            log::info!("Rendering initialisation finished ...");
            Pipe {
                render_res,
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
                vertex_code,
                frag_code,
                vertex_shader_module,
                fragment_shader_module,
                pipeline_layout,
                viewport,
                scissor,
                graphic_pipeline: graphic_pipeline_list[0],
            }
        }
    }

    pub fn draw(&self, interface: &Interface, pref: &Pref) -> Result<bool, Box<dyn Error>> {
        unsafe {
            let next_image = interface.swapchain_loader.acquire_next_image(
                interface.swapchain,
                std::u64::MAX,
                interface.present_complete,
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
                .wait_for_fences(&[interface.draw_cmd_fence], true, std::u64::MAX)
                .expect("DEVICE_LOST");

            interface
                .device
                .reset_fences(&[interface.draw_cmd_fence])
                .expect("FENCE_RESET_FAILED");

            interface
                .device
                .reset_command_buffer(
                    interface.draw_cmd_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("ERR_RESET_CMD_BUFFER");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            interface
                .device
                .begin_command_buffer(interface.draw_cmd_buffer, &command_buffer_begin_info)
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
                .cmd_begin_rendering(interface.draw_cmd_buffer, &rendering_info);
            interface.device.cmd_bind_descriptor_sets(
                interface.draw_cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &self.descriptor_set_list[..],
                &[],
            );
            interface.device.cmd_bind_pipeline(
                interface.draw_cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphic_pipeline,
            );
            interface
                .device
                .cmd_set_viewport(interface.draw_cmd_buffer, 0, &self.viewport);
            interface
                .device
                .cmd_set_scissor(interface.draw_cmd_buffer, 0, &self.scissor);
            interface.device.cmd_bind_vertex_buffers(
                interface.draw_cmd_buffer,
                0,
                &[self.vertex_buffer.buffer],
                &[0],
            );
            interface.device.cmd_bind_index_buffer(
                interface.draw_cmd_buffer,
                self.index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );
            interface.device.cmd_draw_indexed(
                interface.draw_cmd_buffer,
                self.index_data.len() as u32,
                1,
                0,
                0,
                1,
            );
            interface
                .device
                .cmd_end_rendering(interface.draw_cmd_buffer);

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
                        x: self.render_res.width as i32,
                        y: self.render_res.height as i32,
                        z: 1,
                    },
                ],
                dst_subresource: dst,
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: interface.surface_res.width as i32,
                        y: interface.surface_res.height as i32,
                        z: 1,
                    },
                ],
            };

            interface.device.cmd_blit_image(
                interface.draw_cmd_buffer,
                self.image_target_list[present_index as usize].image_target,
                vk::ImageLayout::UNDEFINED,
                interface.present_img_list[present_index as usize],
                vk::ImageLayout::UNDEFINED,
                &[blit],
                pref.img_filter,
            );

            // Finish Draw
            interface
                .device
                .end_command_buffer(interface.draw_cmd_buffer)
                .expect("ERR_END_CMD_BUFFER");

            let command_buffer_list: Vec<vk::CommandBuffer> = vec![interface.draw_cmd_buffer];

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[interface.present_complete])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::BOTTOM_OF_PIPE])
                .command_buffers(&command_buffer_list)
                .signal_semaphores(&[interface.render_complete])
                .build();

            interface
                .device
                .queue_submit(
                    interface.present_queue,
                    &[submit_info],
                    interface.draw_cmd_fence,
                )
                .expect("QUEUE_SUBMIT_FAILED");

            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: 1,
                p_wait_semaphores: &interface.render_complete,
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

    pub fn recreate_swapchain(
        &mut self,
        interface: &mut Interface,
        uniform: &mut Uniform,
        pref: &Pref,
    ) {
        unsafe {
            interface.wait_for_gpu().expect("DEVICE_LOST");

            log::info!("Recreating Swapchain ...");
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

            // New SurfaceCapability
            let surface_capa = interface
                .surface_loader
                .get_physical_device_surface_capabilities(interface.phy_device, interface.surface)
                .unwrap();

            // Select new Dimension
            let dim = interface.window.inner_size();
            interface.surface_res = match surface_capa.current_extent.width {
                std::u32::MAX => vk::Extent2D {
                    width: dim.width,
                    height: dim.height,
                },
                _ => surface_capa.current_extent,
            };

            // Select new RenderResolution
            self.render_res = if pref.use_render_res && interface.window.fullscreen() != None {
                pref.render_res
            } else {
                vk::Extent2D {
                    width: (interface.surface_res.width as f32 / pref.img_scale) as u32,
                    height: (interface.surface_res.height as f32 / pref.img_scale) as u32,
                }
            };

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
