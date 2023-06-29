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

pub struct Engine {
    pub image_target_list: Vec<ImageTarget>,

    pub uniform_buffer: BufferSet,
    pub octree_buffer: BufferSet,

    pub descriptor_pool: DescriptorPool,
    pub pipe_info: Pipe,
    pub pipe: vk::Pipeline,
}

impl Engine {
    pub fn create_compute(
        interface: &Interface,
        pref: &Pref,
        uniform: &mut Uniform,
        octree: &Octree,
    ) -> Self {
        unsafe {
            uniform.apply_resolution(interface.surface.render_res);

            log::info!("Getting ImageTarget List ...");
            let image_target_list = interface
                .swapchain
                .img_list
                .iter()
                .map(|_| ImageTarget::basic_img(interface, interface.surface.render_res))
                .collect();

            log::info!("Creating UniformBuffer ...");
            let uniform_data = uniform.clone();
            let uniform_buffer = BufferSet::new(
                mem::size_of_val(&uniform_data) as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::SharingMode::EXCLUSIVE,
                &interface.device,
            )
            .create_memory(
                &interface.device,
                &interface.phy_device,
                align_of::<Uniform>() as u64,
                mem::size_of_val(&uniform_data) as u64,
                &[uniform_data],
            );

            log::info!("Creating OctreeBuffer ...");
            let octree_data = octree.octant_data.clone();
            let octree_buffer = BufferSet::new(
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
                &octree_data,
            );

            log::info!("Creating descriptor set layout list ...");
            let mut descriptor_pool = DescriptorPool::default()
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

            descriptor_pool = descriptor_pool
                .create_descriptor_pool(&interface.device)
                .write_descriptor_pool(&interface.device);

            log::info!("Writing descriptor list ...");
            descriptor_pool.write_buffer_desc(
                &uniform_buffer,
                mem::size_of_val(&uniform_data) as u64,
                1,
                0,
                vk::DescriptorType::UNIFORM_BUFFER,
                &interface.device,
            );

            descriptor_pool.write_buffer_desc(
                &octree_buffer,
                (mem::size_of::<u32>() * octree_data.len()) as u64,
                2,
                0,
                vk::DescriptorType::STORAGE_BUFFER,
                &interface.device,
            );

            let mut pipe_info = Pipe::default();

            pipe_info = pipe_info.create_shader_module(
                &mut Cursor::new(&include_bytes!("../../shader/comp.spv")[..]),
                &interface.device,
                vk::ShaderStageFlags::COMPUTE,
            );

            pipe_info = pipe_info.create_layout(&descriptor_pool, &interface.device);

            let compute_pipe_info = vk::ComputePipelineCreateInfo::builder()
                .stage(pipe_info.shader_list[0].stage_info)
                .layout(pipe_info.pipe_layout)
                .build();

            let pipe = interface
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &[compute_pipe_info], None)
                .expect("ERROR_CREATE_PIPELINE")[0];

            log::info!("Rendering initialisation finished ...");
            Self {
                image_target_list,
                uniform_buffer,
                octree_buffer,
                descriptor_pool,
                pipe_info,
                pipe,
            }
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
                self.pipe_info.record_submit_cmd(
                    &interface.device,
                    interface.draw_cmd_fence,
                    interface.draw_cmd_buffer,
                    interface.present_complete,
                    interface.render_complete,
                    interface.present_queue,
                    |cmd_buffer| {
                        self.descriptor_pool.write_img_desc(
                            &self.image_target_list[present_index as usize],
                            vk::ImageLayout::GENERAL,
                            0,
                            0,
                            vk::DescriptorType::STORAGE_IMAGE,
                            &interface.device,
                        );
                        self.descriptor_pool.write_buffer_desc(
                            &self.uniform_buffer,
                            mem::size_of_val(uniform) as u64,
                            1,
                            0,
                            vk::DescriptorType::UNIFORM_BUFFER,
                            &interface.device,
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
                            self.pipe_info.pipe_layout,
                            0,
                            &self.descriptor_pool.set_list[..],
                            &[],
                        );
                        interface.device.cmd_dispatch(
                            cmd_buffer,
                            interface.surface.render_res.width / 16,
                            interface.surface.render_res.height / 16,
                            1,
                        );

                        // First Image Barrier
                        self.pipe_info.first_img_barrier(
                            &self.image_target_list[present_index as usize],
                            interface.swapchain.img_list[present_index as usize],
                            &interface.device,
                            cmd_buffer,
                        );
                        // Copy image memory
                        self.pipe_info.copy_image(
                            &interface.device,
                            cmd_buffer,
                            pref,
                            self.image_target_list[present_index as usize].img,
                            interface.swapchain.img_list[present_index as usize],
                            interface.surface.render_res,
                            interface.surface.surface_res,
                        );
                        self.pipe_info.sec_img_barrier(
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
        unsafe {
            interface.wait_for_gpu().expect("DEVICE_LOST");

            log::info!("Recreating Swapchain ...");

            // Destroy Image Target
            self.image_target_list.iter().for_each(|target| {
                target.destroy(&interface.device);
            });

            // Destroy Swapchain and SwapchainImgList
            interface
                .swapchain
                .view_list
                .iter()
                .for_each(|view| interface.device.destroy_image_view(*view, None));
            interface
                .swapchain
                .loader
                .destroy_swapchain(interface.swapchain.swapchain, None);

            interface.surface =
                interface
                    .surface
                    .get_surface_info(&interface.phy_device, &interface.window, pref);

            uniform.apply_resolution(interface.surface.render_res);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(interface.surface.surface)
                .min_image_count(interface.surface.swap_img_count)
                .image_color_space(interface.surface.format.color_space)
                .image_format(interface.surface.format.format)
                .image_extent(interface.surface.surface_res)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(interface.surface.pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(interface.surface.present_mode)
                .clipped(true)
                .image_array_layers(1);

            interface.swapchain.swapchain = interface
                .swapchain
                .loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();

            interface.swapchain.img_list = interface
                .swapchain
                .loader
                .get_swapchain_images(interface.swapchain.swapchain)
                .unwrap();

            interface.swapchain.view_list = interface
                .swapchain
                .img_list
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(interface.surface.format.format)
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
                .swapchain
                .view_list
                .iter()
                .map(|_| ImageTarget::basic_img(interface, interface.surface.render_res))
                .collect();
        }
    }
}
