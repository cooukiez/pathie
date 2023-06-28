use std::{
    error::Error,
    ffi::{c_void, CString},
    io::Cursor,
    mem::{self, align_of},
};

use ash::{
    util::{read_spv, Align},
    vk,
};

use crate::{
    interface::interface::Interface, pipe::descriptor::DescriptorPool, tree::octree::Octree,
    uniform::Uniform, Pref, DEFAULT_STORAGE_BUFFER_SIZE,
};

use super::{buffer::BufferSet, image::ImageTarget};

impl Pipe {
    pub fn init(
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

            log::info!("Getting ShaderCode ...");
            let mut primary_ray_spv_file =
                Cursor::new(&include_bytes!("../../shader/comp.spv")[..]);

            let primary_ray_code =
                read_spv(&mut primary_ray_spv_file).expect("ERR_READ_VERTEX_SPV");
            let primary_ray_info = vk::ShaderModuleCreateInfo::builder().code(&primary_ray_code);

            let primary_ray_module = interface
                .device
                .create_shader_module(&primary_ray_info, None)
                .expect("ERR_VERTEX_MODULE");

            let layout_create_info =
                vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_pool.layout_list);

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
                render_res: interface.surface.render_res,
                image_target_list,
                uniform_buffer,
                octree_buffer,
                descriptor_pool,
                primary_ray_code,
                primary_ray_module,
                pipe_layout,
                pipe,
            }
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
                .image(image.img)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .subresource_range(basic_subresource_range.clone())
                .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
                .build();

            let comp_transfer = vk::ImageMemoryBarrier::builder()
                .image(image.img)
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
                        self.descriptor_pool
                            .write_img_desc(
                                &self.image_target_list[present_index as usize],
                                vk::ImageLayout::GENERAL,
                                0,
                                0,
                                vk::DescriptorType::STORAGE_IMAGE,
                                &interface.device,
                            );
                        self.descriptor_pool
                            .write_buffer_desc(
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
                            self.pipe_layout,
                            0,
                            &self.descriptor_pool.set_list[..],
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
                            interface.swapchain.img_list[present_index as usize],
                            interface,
                            cmd_buffer,
                        );
                        // Copy image memory
                        self.copy_image(
                            interface,
                            pref,
                            self.image_target_list[present_index as usize].img,
                            interface.swapchain.img_list[present_index as usize],
                            self.render_res,
                            interface.surface.surface_res,
                        );
                        self.sec_img_barrier(
                            interface.swapchain.img_list[present_index as usize],
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

            uniform.apply_resolution(self.render_res);

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
                .map(|_| ImageTarget::basic_img(interface, self.render_res))
                .collect();
        }
    }
}
