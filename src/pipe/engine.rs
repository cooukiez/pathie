use std::{
    error::Error,
    ffi::CString,
    io::Cursor,
    mem::{self, align_of},
};

use ash::{util::read_spv, vk};

use crate::{
    interface::interface::Interface,
    pipe::{
        descriptor::DescriptorPool,
        pipe::{Pipe, Vertex},
    },
    tree::octree::Octree,
    uniform::Uniform,
    Pref, DEFAULT_STORAGE_BUFFER_SIZE, offset_of,
};

use super::{buffer::BufferSet, image::ImageTarget};

#[derive(Clone)]
pub struct Engine {
    pub image_target_list: Vec<ImageTarget>,

    pub index_data: Vec<u32>,

    pub index_buffer: BufferSet,
    pub vertex_buffer: BufferSet,

    pub uniform_buffer: BufferSet,
    pub octree_buffer: BufferSet,

    pub pool_comp: DescriptorPool,
    pub pipe_comp: Pipe,
    pub vk_pipe_comp: vk::Pipeline,

    pub pool_graphic: DescriptorPool,
    pub pipe_graphic: Pipe,
    pub vk_pipe_graphic: vk::Pipeline,
}

impl Engine {
    pub fn create_base(interface: &Interface, uniform: &Uniform, octree: &Octree) -> Self {
        let mut result = Self::default();

        log::info!("Getting ImageTarget List ...");
        result.image_target_list = interface
            .swapchain
            .img_list
            .iter()
            .map(|_| ImageTarget::basic_img(interface, interface.surface.render_res))
            .collect();

        log::info!("Creating IndexBuffer ...");
        let index_data = vec![0u32, 1, 2, 2, 3, 0];
        result.index_buffer = BufferSet::new(
            mem::size_of_val(&index_data) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<u32>() as u64,
            mem::size_of_val(&index_data) as u64,
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
        result.vertex_buffer = BufferSet::new(
            mem::size_of_val(&vertex_data) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<Vertex>() as u64,
            mem::size_of_val(&vertex_data) as u64,
            &index_data,
        );

        log::info!("Creating UniformBuffer ...");
        result.uniform_buffer = BufferSet::new(
            mem::size_of_val(&uniform) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<Uniform>() as u64,
            mem::size_of_val(&uniform) as u64,
            &[uniform],
        );

        log::info!("Creating OctreeBuffer ...");
        result.octree_buffer = BufferSet::new(
            DEFAULT_STORAGE_BUFFER_SIZE,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<u32>() as u64,
            DEFAULT_STORAGE_BUFFER_SIZE,
            &octree.octant_data,
        );

        result
    }

    pub fn create_compute(
        &self,
        interface: &Interface,
        uniform: &Uniform,
        octree: &Octree,
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Creating descriptor set layout list ...");
            result.pool_comp = DescriptorPool::default()
                // ImageTarget
                .create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_IMAGE,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    &interface.device,
                )
                // Uniform Set
                .create_descriptor_set_layout(
                    vk::DescriptorType::UNIFORM_BUFFER,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    &interface.device,
                )
                // Octree Set
                .create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::COMPUTE,
                    &interface.device,
                );

            result.pool_comp = result
                .pool_comp
                .create_descriptor_pool(&interface.device)
                .write_descriptor_pool(&interface.device);

            log::info!("Writing descriptor list ...");
            result.pool_comp.write_buffer_desc(
                &self.uniform_buffer,
                vk::WHOLE_SIZE,
                1,
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                &interface.device,
            );

            result.pool_comp.write_buffer_desc(
                &self.octree_buffer,
                vk::WHOLE_SIZE,
                2,
                0,
                vk::DescriptorType::STORAGE_BUFFER,
                &interface.device,
            );

            log::info!("Getting ShaderCode ...");
            let mut spv = Cursor::new(&include_bytes!("../../shader/comp.spv")[..]);

            let code = read_spv(&mut spv).expect("ERR_READ_VERTEX_SPV");
            let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

            let shader_module = interface
                .device
                .create_shader_module(&shader_info, None)
                .expect("ERR_VERTEX_MODULE");

            log::info!("Stage Creation ...");
            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage = vk::PipelineShaderStageCreateInfo {
                module: shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            };

            result.pipe_comp = result
                .pipe_comp
                .create_layout(&result.pool_comp, &interface.device);

            let compute_pipe_info = vk::ComputePipelineCreateInfo::builder()
                .stage(shader_stage)
                .layout(result.pipe_comp.pipe_layout)
                .build();

            result.vk_pipe_comp = interface
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &[compute_pipe_info], None)
                .expect("ERROR_CREATE_PIPELINE")[0];

            result
        }
    }

    pub fn create_graphic(
        &self,
        interface: &Interface,
        uniform: &Uniform,
        octree: &Octree,
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Creating descriptor set layout list ...");
            result.pool_graphic = DescriptorPool::default()
                // Uniform Set
                .create_descriptor_set_layout(
                    vk::DescriptorType::UNIFORM_BUFFER,
                    1,
                    vk::ShaderStageFlags::FRAGMENT,
                    &interface.device,
                )
                // Octree Set
                .create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::FRAGMENT,
                    &interface.device,
                );

            result.pool_graphic = result
                .pool_graphic
                .create_descriptor_pool(&interface.device)
                .write_descriptor_pool(&interface.device);

            log::info!("Writing descriptor list ...");
            result.pool_graphic.write_buffer_desc(
                &self.uniform_buffer,
                vk::WHOLE_SIZE,
                0,
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                &interface.device,
            );

            result.pool_graphic.write_buffer_desc(
                &self.octree_buffer,
                vk::WHOLE_SIZE,
                1,
                0,
                vk::DescriptorType::STORAGE_BUFFER,
                &interface.device,
            );

            log::info!("Getting ShaderCode ...");
            let mut vert_spv = Cursor::new(&include_bytes!("../../shader/vert.spv")[..]);
            let mut frag_spv = Cursor::new(&include_bytes!("../../shader/frag.spv")[..]);

            let vert_code = read_spv(&mut vert_spv).expect("ERR_READ_VERTEX_SPV");
            let frag_code = read_spv(&mut frag_spv).expect("ERR_READ_VERTEX_SPV");

            let vert_shader_info = vk::ShaderModuleCreateInfo::builder()
                .code(&vert_code)
                .build();
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder()
                .code(&frag_code)
                .build();

            let vert_shader_module = interface
                .device
                .create_shader_module(&vert_shader_info, None)
                .expect("ERR_VERTEX_MODULE");
            let frag_shader_module = interface
                .device
                .create_shader_module(&frag_shader_info, None)
                .expect("ERR_VERTEX_MODULE");

            log::info!("Stage Creation ...");
            let shader_entry_name = CString::new("main").unwrap();

            let shader_stage_list = vec![
                vk::PipelineShaderStageCreateInfo {
                    module: vert_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    module: frag_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ];

            let vertex_input_binding_description_list = [
                vk::VertexInputBindingDescription { binding: 0, stride: mem::size_of::<Vertex>() as u32, input_rate: vk::VertexInputRate::VERTEX, }
            ];

            let vertex_input_attribute_description_list = [
                vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32B32A32_SFLOAT, offset: offset_of!(Vertex, pos) as u32, },
                vk::VertexInputAttributeDescription { location: 1, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: offset_of!(Vertex, uv) as u32, },
            ];

            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(&vertex_input_attribute_description_list)
                .vertex_binding_descriptions(&vertex_input_binding_description_list);

            let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo { topology: vk::PrimitiveTopology::TRIANGLE_LIST, ..Default::default() };

            let color_blend_attachment_state_list = [
                vk::PipelineColorBlendAttachmentState {
                    blend_enable: 0,

                    src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                    dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,

                    color_blend_op: vk::BlendOp::ADD,
                    src_alpha_blend_factor: vk::BlendFactor::ZERO,
                    dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                    alpha_blend_op: vk::BlendOp::ADD,

                    color_write_mask: vk::ColorComponentFlags::RGBA,
                }
            ];

            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_state_list);


            result.pipe_graphic = result
                .pipe_graphic
                .create_layout(&result.pool_graphic, &interface.device)
                .create_vertex_stage()
                .create_viewport(&interface.surface)
                .create_rasterization()
                .create_multisampling()
                .create_color_blending()
                .create_dynamic_state()
                .create_rendering(&interface.surface);

            let graphic_pipe_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stage_list)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&result.pipe_graphic.viewport_state)
                .rasterization_state(&result.pipe_graphic.raster_state)
                .multisample_state(&result.pipe_graphic.multisample_state)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&result.pipe_graphic.dynamic_state)
                .layout(result.pipe_graphic.pipe_layout)
                .push_next(&mut result.pipe_graphic.rendering)
                .build();

            result.vk_pipe_graphic = interface
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipe_info], None)
                .expect("ERROR_CREATE_PIPELINE")[0];

            result
        }
    }

    /// Draw the next image onto the window
    /// Get swapchain image, begin draw, render with
    /// pipe onto image target and finally blit to swapchain
    /// image. Then end draw.

    pub fn draw_comp(
        &self,
        interface: &Interface,
        pref: &Pref,
        uniform: &Uniform,
    ) -> Result<bool, Box<dyn Error>> {
        unsafe {
            interface.swap_draw_next(|present_index| {
                self.pipe_comp.record_submit_cmd(
                    &interface.device,
                    interface.draw_cmd_fence,
                    interface.draw_cmd_buffer,
                    interface.present_complete,
                    interface.render_complete,
                    interface.present_queue,
                    |cmd_buffer| {
                        self.pool_comp.write_img_desc(
                            &self.image_target_list[present_index as usize],
                            vk::ImageLayout::GENERAL,
                            0,
                            0,
                            vk::DescriptorType::STORAGE_IMAGE,
                            &interface.device,
                        );
                        self.pool_comp.write_buffer_desc(
                            &self.uniform_buffer,
                            vk::WHOLE_SIZE,
                            1,
                            0,
                            vk::DescriptorType::UNIFORM_BUFFER,
                            &interface.device,
                        );

                        // Dispatch Compute Pipe
                        interface.device.cmd_bind_pipeline(
                            cmd_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            self.vk_pipe_comp,
                        );
                        interface.device.cmd_bind_descriptor_sets(
                            cmd_buffer,
                            vk::PipelineBindPoint::COMPUTE,
                            self.pipe_comp.pipe_layout,
                            0,
                            &self.pool_comp.set_list[..],
                            &[],
                        );
                        interface.device.cmd_dispatch(
                            cmd_buffer,
                            interface.surface.render_res.width / 16,
                            interface.surface.render_res.height / 16,
                            1,
                        );

                        // First Image Barrier
                        self.pipe_comp.first_img_barrier(
                            &self.image_target_list[present_index as usize],
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        // Copy image memory
                        self.pipe_comp.copy_image(
                            &interface.device,
                            cmd_buffer,
                            pref,
                            self.image_target_list[present_index as usize].img,
                            interface.swapchain.img_list[present_index as usize],
                            interface.surface.render_res,
                            interface.surface.surface_res,
                        );
                        self.pipe_comp.sec_img_barrier(
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                    },
                );
            })
        }
    }

    pub fn draw_graphic(
        &self,
        interface: &Interface,
        pref: &Pref,
        uniform: &Uniform,
    ) -> Result<bool, Box<dyn Error>> {
        unsafe {
            interface.swap_draw_next(|present_index| {
                self.pipe_comp.record_submit_cmd(
                    &interface.device,
                    interface.draw_cmd_fence,
                    interface.draw_cmd_buffer,
                    interface.present_complete,
                    interface.render_complete,
                    interface.present_queue,
                    |cmd_buffer| {
                        self.pool_graphic.write_buffer_desc(
                            &self.uniform_buffer,
                            vk::WHOLE_SIZE,
                            0,
                            0,
                            vk::DescriptorType::UNIFORM_BUFFER,
                            &interface.device,
                        );

                        let color_attachment_info = vk::RenderingAttachmentInfoKHR::builder()
                            .image_view(self.image_target_list[present_index as usize].view)
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .store_op(vk::AttachmentStoreOp::STORE)
                            .clear_value(vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [0.0, 0.0, 0.0, 0.0],
                                },
                            })
                            .build();

                        let color_attachment_list = [color_attachment_info];

                        let rendering_info = vk::RenderingInfoKHR::builder()
                            .render_area(vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: interface.surface.render_res,
                            })
                            .layer_count(1)
                            .color_attachments(&color_attachment_list)
                            .build();

                        // Dispatch Compute Pipe
                        interface
                            .device
                            .cmd_begin_rendering(cmd_buffer, &rendering_info);

                        interface.device.cmd_bind_descriptor_sets(
                            cmd_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.pipe_graphic.pipe_layout,
                            0,
                            &self.pool_graphic.set_list[..],
                            &[],
                        );

                        interface.device.cmd_bind_pipeline(
                            cmd_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.vk_pipe_graphic,
                        );
                        interface.device.cmd_set_viewport(
                            cmd_buffer,
                            0,
                            &self.pipe_graphic.viewport,
                        );

                        interface
                            .device
                            .cmd_set_scissor(cmd_buffer, 0, &self.pipe_graphic.scissor);

                        interface.device.cmd_bind_vertex_buffers(
                            cmd_buffer,
                            0,
                            &[self.vertex_buffer.buffer],
                            &[0],
                        );

                        interface.device.cmd_bind_index_buffer(
                            cmd_buffer,
                            self.index_buffer.buffer,
                            0,
                            vk::IndexType::UINT32,
                        );

                        interface.device.cmd_draw_indexed(
                            cmd_buffer,
                            self.index_data.len() as u32,
                            1,
                            0,
                            0,
                            1,
                        );

                        interface.device.cmd_end_rendering(cmd_buffer);

                        // First Image Barrier
                        self.pipe_comp.first_img_barrier(
                            &self.image_target_list[present_index as usize],
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        // Copy image memory
                        self.pipe_comp.copy_image(
                            &interface.device,
                            cmd_buffer,
                            pref,
                            self.image_target_list[present_index as usize].img,
                            interface.swapchain.img_list[present_index as usize],
                            interface.surface.render_res,
                            interface.surface.surface_res,
                        );
                        self.pipe_comp.sec_img_barrier(
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
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
        interface.wait_for_gpu().expect("DEVICE_LOST");

        log::info!("Recreating Swapchain ...");
        self.image_target_list.iter().for_each(|target| {
            target.destroy(&interface.device);
        });

        interface.swapchain.destroy(&interface.device);

        interface.surface =
            interface
                .surface
                .get_surface_info(&interface.phy_device, &interface.window, pref);

        uniform.apply_resolution(interface.surface.render_res);

        interface.swapchain = interface
            .swapchain
            .create_swapchain(&interface.surface)
            .get_present_img(&interface.surface, &interface.device);

        self.image_target_list = interface
            .swapchain
            .view_list
            .iter()
            .map(|_| ImageTarget::basic_img(interface, interface.surface.render_res))
            .collect();
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            image_target_list: Default::default(),
            index_data: Default::default(),
            index_buffer: Default::default(),
            vertex_buffer: Default::default(),
            uniform_buffer: Default::default(),
            octree_buffer: Default::default(),
            pool_comp: Default::default(),
            pipe_comp: Default::default(),
            vk_pipe_comp: Default::default(),
            pool_graphic: Default::default(),
            pipe_graphic: Default::default(),
            vk_pipe_graphic: Default::default(),
        }
    }
}
