use std::{
    error::Error,
    ffi::CString,
    io::Cursor,
    mem::{self, align_of},
};

use ash::{util::read_spv, vk};

use crate::{
    interface::interface::Interface,
    pipe::{descriptor::DescriptorPool, pipe::Pipe},
    tree::octree::Octree,
    uniform::Uniform,
    Pref, DEFAULT_STORAGE_BUFFER_SIZE,
};

use super::{buffer::BufferSet, image::ImageTarget};

#[derive(Clone)]
pub struct Engine {
    pub image_target_list: Vec<ImageTarget>,

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
                1,
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                &interface.device,
            );

            result.pool_graphic.write_buffer_desc(
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
                .stages(&shader_stage_info_list)
                .vertex_input_state(&result.pipe_graphic.vertex_state)
                .input_assembly_state(&result.pipe_graphic.vertex_assembly_state)
                .viewport_state(&result.pipe_graphic.viewport_state)
                .rasterization_state(&result.pipe_graphic.raster_state)
                .multisample_state(&result.pipe_graphic.multisample_state)
                .color_blend_state(&result.pipe_graphic.blend_state)
                .dynamic_state(&result.pipe_graphic.dynamic_state)
                .layout(result.pipe_graphic.pipe_layout)
                .push_next(&mut result.pipe_graphic.rendering)
                .build();

            result.vk_pipe_graphic = interface
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &[graphic_pipe_info], None)
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
