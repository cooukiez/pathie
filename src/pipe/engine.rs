use std::{
    error::Error,
    mem::{self, align_of},
};

use ash::vk;
use cgmath::Vector3;

use crate::{
    interface::interface::Interface,
    pipe::{
        descriptor::DescriptorPool,
        pipe::{LocInfo, Pipe, Vertex},
    },
    tree::{
        octant::Octant,
        octree::{Octree, MAX_DEPTH},
        trace::{BranchInfo, PosInfo},
    },
    uniform::Uniform,
    vector::Vector,
    Pref, DEFAULT_STORAGE_BUFFER_SIZE, DEFAULT_UNIFORM_BUFFER_SIZE,
};

use super::{buffer::BufferSet, image::ImageTarget};

#[derive(Clone)]
pub struct Engine {
    pub image_target_list: Vec<ImageTarget>,
    pub depth_image: ImageTarget,

    pub index_data: Vec<u32>,

    pub index_buffer: BufferSet,
    pub vertex_buffer: BufferSet,

    pub uniform_buffer: BufferSet,
    pub octree_buffer: BufferSet,
    pub loc_info_buffer: BufferSet,

    pub pool_comp: DescriptorPool,
    pub pipe_comp: Pipe,
    pub vk_pipe_comp: vk::Pipeline,

    pub pool_graphic: DescriptorPool,
    pub pipe_graphic: Pipe,
}

impl Engine {
    pub fn create_base(interface: &Interface, uniform: &Uniform, octree: &Octree) -> Self {
        let mut result = Self::default();

        result.image_target_list = interface
            .swapchain
            .img_list
            .iter()
            .map(|_| ImageTarget::attachment_img(interface, interface.surface.render_res))
            .collect();

        result.depth_image = ImageTarget::depth_img(interface, interface.surface.render_res.into());

        let (vertex_data, index_data, loc_info) = Pipe::get_octree_vert_data(octree);

        log::info!("Creating IndexBuffer ...");
        result.index_data = index_data;
        result.index_buffer = BufferSet::new(
            mem::size_of_val(&result.index_data[..]) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<u32>() as u64,
            mem::size_of_val(&result.index_data[..]) as u64,
            &result.index_data,
        );

        log::info!("Creating VertexBuffer ...");
        result.vertex_buffer = BufferSet::new(
            mem::size_of_val(&vertex_data[..]) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<Vertex>() as u64,
            mem::size_of_val(&vertex_data[..]) as u64,
            &vertex_data,
        );

        log::info!("Creating UniformBuffer ...");
        result.uniform_buffer = BufferSet::new(
            mem::size_of::<Uniform>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<Uniform>() as u64,
            mem::size_of::<Uniform>() as u64,
            &[uniform.clone()],
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

        log::info!("Creating Location Info Buffer ...");
        result.loc_info_buffer = BufferSet::new(
            (mem::size_of::<LocInfo>() * loc_info.len()) as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::SharingMode::EXCLUSIVE,
            &interface.device,
        )
        .create_memory(
            &interface.device,
            &interface.phy_device,
            align_of::<LocInfo>() as u64,
            (mem::size_of::<LocInfo>() * loc_info.len()) as u64,
            &loc_info,
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

            result.pipe_comp = Pipe::create_comp_pipe(&interface.device, &result.pool_comp);

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
                    vk::ShaderStageFlags::ALL_GRAPHICS,
                    &interface.device,
                )
                // Octree Set
                .create_descriptor_set_layout(
                    vk::DescriptorType::STORAGE_BUFFER,
                    1,
                    vk::ShaderStageFlags::FRAGMENT,
                    &interface.device,
                )
                // location info set
                .create_descriptor_set_layout(
                    vk::DescriptorType::UNIFORM_BUFFER,
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

            result.pool_graphic.write_buffer_desc(
                &self.loc_info_buffer,
                vk::WHOLE_SIZE,
                2,
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                &interface.device,
            );

            result.pipe_graphic = Pipe::create_graphic_pipe(
                &interface.device,
                &interface.surface,
                &result.pool_graphic,
            );

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
                        /*
                        self.pipe_comp.first_img_barrier(
                            &self.image_target_list[present_index as usize],
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        */
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
                        /*
                        self.pipe_comp.sec_img_barrier(
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        */
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
                self.pipe_graphic.record_submit_cmd(
                    &interface.device,
                    interface.draw_cmd_fence,
                    interface.draw_cmd_buffer,
                    interface.present_complete,
                    interface.render_complete,
                    interface.present_queue,
                    |cmd_buffer| {
                        /*
                        self.pool_graphic.write_buffer_desc(
                            &self.uniform_buffer,
                            vk::WHOLE_SIZE,
                            0,
                            0,
                            vk::DescriptorType::UNIFORM_BUFFER,
                            &interface.device,
                        );
                        */

                        let color_attachment_info = vk::RenderingAttachmentInfoKHR::builder()
                            .image_view(self.image_target_list[present_index as usize].view)
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .store_op(vk::AttachmentStoreOp::STORE)
                            .clear_value(vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [1.0, 1.0, 1.0, 0.0],
                                },
                            })
                            .build();

                        let color_attachment_list = [color_attachment_info];

                        let depth_attachment_info = vk::RenderingAttachmentInfo::builder()
                            .image_view(self.depth_image.view)
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                            .resolve_image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                            .clear_value(vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            })
                            .build();

                        let rendering_info = vk::RenderingInfoKHR::builder()
                            .render_area(vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: interface.surface.render_res,
                            })
                            .layer_count(1)
                            .color_attachments(&color_attachment_list)
                            .depth_attachment(&depth_attachment_info)
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
                            self.pipe_graphic.pipe,
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
                        /*
                        self.pipe_comp.first_img_barrier(
                            &self.image_target_list[present_index as usize],
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        */
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
                        /*
                        self.pipe_comp.sec_img_barrier(
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        */
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

        self.depth_image.destroy(&interface.device);

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
            .img_list
            .iter()
            .map(|_| ImageTarget::attachment_img(interface, interface.surface.render_res))
            .collect();

        self.depth_image = ImageTarget::depth_img(interface, interface.surface.render_res.into());

        self.pipe_graphic.viewport = vec![vk::Viewport {
            width: interface.surface.render_res.width as f32,
            height: interface.surface.render_res.height as f32,
            max_depth: 1.0,

            ..Default::default()
        }];

        self.pipe_graphic.scissor = vec![interface.surface.render_res.into()];
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            image_target_list: Default::default(),
            depth_image: Default::default(),
            index_data: Default::default(),
            index_buffer: Default::default(),
            vertex_buffer: Default::default(),
            uniform_buffer: Default::default(),
            octree_buffer: Default::default(),
            loc_info_buffer: Default::default(),
            pool_comp: Default::default(),
            pipe_comp: Default::default(),
            vk_pipe_comp: Default::default(),
            pool_graphic: Default::default(),
            pipe_graphic: Default::default(),
        }
    }
}
