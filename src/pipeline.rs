use std::{ffi::{CString}, io::Cursor};

use ash::{vk::{self, ImageAspectFlags}, Device, util};

use crate::{vulkan::{ImageObj, BufferObj, PipelineData}, data::{Uniform}};

pub struct Render {
    pub image: ImageObj,

    pub buffer_list: Vec<BufferObj>,
    pub uniform_list: Vec<BufferObj>,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout_list: Vec<vk::DescriptorSetLayout>,
    pub descriptor_set_list: Vec<vk::DescriptorSet>,

    pub pipeline_layout: vk::PipelineLayout,
    pub compute_pipeline: vk::Pipeline,
}

impl Render {
    pub fn init_descriptor_pool(buffer_list: &Vec<BufferObj>, uniform_list: &Vec<BufferObj>, device: &Device, image: &ImageObj, ) -> (vk::DescriptorPool, Vec<vk::DescriptorSetLayout>, ) {
        log::info!("Init DescriptorPool ...");

        let image_pool_size = vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::STORAGE_IMAGE).descriptor_count(1).build();
        let buffer_pool_size = vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(buffer_list.len() as u32).build();
        let uniform_pool_size = vk::DescriptorPoolSize::builder().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(uniform_list.len() as u32).build();

        let pool_size_list = [image_pool_size, buffer_pool_size, uniform_pool_size];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder().max_sets(pool_size_list.len() as u32).pool_sizes(&pool_size_list).build();
        let descriptor_pool = unsafe { device.create_descriptor_pool(&descriptor_pool_info, None, ).unwrap() };   

        let image_set_layout = PipelineData::create_desc_layout(&vec![image], vk::DescriptorType::STORAGE_IMAGE, &device, );
        let buffer_set_layout = PipelineData::create_desc_layout(&buffer_list, vk::DescriptorType::STORAGE_BUFFER, &device, );
        let uniform_set_layout = PipelineData::create_desc_layout(&uniform_list, vk::DescriptorType::UNIFORM_BUFFER, &device, );

        let descriptor_set_layout_list: Vec<vk::DescriptorSetLayout> = vec![image_set_layout, buffer_set_layout, uniform_set_layout];

        (descriptor_pool, descriptor_set_layout_list, )
    }

    pub fn update_descriptor_pool(descriptor_pool: vk::DescriptorPool, descriptor_set_layout_list: &Vec<vk::DescriptorSetLayout>, device: &Device, image_layout: vk::ImageLayout, image: &ImageObj, buffer_list: &Vec<BufferObj>, uniform_list: &Vec<BufferObj>, ) -> Vec<vk::DescriptorSet> {
        let descriptor_allocate_info = vk::DescriptorSetAllocateInfo::builder().descriptor_pool(descriptor_pool).set_layouts(&descriptor_set_layout_list).build();
        let descriptor_set_list = unsafe { device.allocate_descriptor_sets(&descriptor_allocate_info).unwrap() };

        let image_desc_info = vk::DescriptorImageInfo::builder().image_layout(image_layout).image_view(image.image_view).build();
        let mut write_list: Vec<vk::WriteDescriptorSet> = vec![vk::WriteDescriptorSet::builder().dst_set(descriptor_set_list[0]).dst_binding(0).descriptor_type(vk::DescriptorType::STORAGE_IMAGE).image_info(&[image_desc_info]).build()];
        
        for (index, _, ) in buffer_list.iter().enumerate() { let buffer_info = vk::DescriptorBufferInfo::builder().buffer(buffer_list[index].buffer).offset(0).range(vk::WHOLE_SIZE).build(); write_list.push(vk::WriteDescriptorSet::builder().dst_set(descriptor_set_list[1]).dst_binding(index as u32).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(&[buffer_info]).build()); }
        for (index, _, ) in uniform_list.iter().enumerate() { let buffer_info = vk::DescriptorBufferInfo::builder().buffer(uniform_list[index].buffer).offset(0).range(std::mem::size_of::<Uniform>() as u64).build(); write_list.push(vk::WriteDescriptorSet::builder().dst_set(descriptor_set_list[2]).dst_binding(index as u32).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(&[buffer_info]).build()); }

        unsafe { device.update_descriptor_sets(&write_list, &[]); }

        descriptor_set_list
    }

    pub fn init_compute_pipeline(device: &Device, descriptor_set_layout_list: &Vec<vk::DescriptorSetLayout> ) -> (vk::Pipeline, vk::PipelineLayout, ) {
        log::info!("Init ComputePipeline ...");
        let mut compute_spv_file = Cursor::new(&include_bytes!("../shader/raytracing/comp.spv")[..]);
        let compute_code = util::read_spv(&mut compute_spv_file).expect("ERROR_READ_SPV");

        let compute_shader_info = vk::ShaderModuleCreateInfo::builder().code(&compute_code).build();
        let compute_shader_module = unsafe { device.create_shader_module(&compute_shader_info, None, ).unwrap() };

        let main_function_name = CString::new("main").unwrap();
        let compute_stage = vk::PipelineShaderStageCreateInfo::builder().module(compute_shader_module).name(&main_function_name).stage(vk::ShaderStageFlags::COMPUTE).build();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_set_layout_list).build();
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None, ).unwrap() };
        let pipeline_info = [vk::ComputePipelineCreateInfo::builder().stage(compute_stage).layout(pipeline_layout).build()];
        let pipeline_list = unsafe { device.create_compute_pipelines(vk::PipelineCache::null(), &pipeline_info, None, ) }.unwrap();

        (pipeline_list[0], pipeline_layout, )   
    }

    pub fn set_first_img_mem_barrier(image: vk::Image, swapchain_image_list: &Vec<vk::Image>, image_index: usize, device: &Device, command_buffer: vk::CommandBuffer, ) {
        let mut compute_write = vk::ImageMemoryBarrier { src_queue_family_index: vk::QUEUE_FAMILY_IGNORED, dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED, image, subresource_range: vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, }, ..Default::default() };
            compute_write.old_layout = vk::ImageLayout::UNDEFINED;
            compute_write.new_layout = vk::ImageLayout::GENERAL;
            compute_write.src_access_mask = vk::AccessFlags::NONE;
            compute_write.dst_access_mask = vk::AccessFlags::SHADER_WRITE;

        let mut compute_transfer = vk::ImageMemoryBarrier { src_queue_family_index: vk::QUEUE_FAMILY_IGNORED, dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED, image, subresource_range: vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, }, ..Default::default() };
            compute_transfer.old_layout = vk::ImageLayout::GENERAL;
            compute_transfer.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            compute_transfer.src_access_mask = vk::AccessFlags::SHADER_WRITE;
            compute_transfer.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

        let mut swap_transfer = vk::ImageMemoryBarrier { src_queue_family_index: vk::QUEUE_FAMILY_IGNORED, dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED, image: swapchain_image_list[image_index], subresource_range: vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, }, ..Default::default() };
            swap_transfer.old_layout = vk::ImageLayout::UNDEFINED;
            swap_transfer.new_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            swap_transfer.src_access_mask = vk::AccessFlags::NONE;
            swap_transfer.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

        let barrier_list: Vec<vk::ImageMemoryBarrier> = vec![compute_write, compute_transfer, swap_transfer];
        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::COMPUTE_SHADER, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[], &[], &barrier_list, ); }
    }

    pub fn copy_image_mem(extent: &vk::Extent2D, device: &Device, command_buffer: vk::CommandBuffer, image: vk::Image, swapchain_image_list: &Vec<vk::Image>, image_index: usize, ) {
        let src = vk::ImageSubresourceLayers { aspect_mask: ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
        let dst = vk::ImageSubresourceLayers { aspect_mask: ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
        let copy = vk::ImageCopy { src_subresource: src, src_offset: vk::Offset3D { x: 0, y: 0, z: 0 }, dst_subresource: dst, dst_offset: vk::Offset3D { x: 0, y: 0, z: 0 }, extent: vk::Extent3D { width: extent.width, height: extent.height, depth: 1 } };
        unsafe { device.cmd_copy_image(command_buffer, image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, swapchain_image_list[image_index], vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy], ); }
    }

    pub fn set_second_img_mem_barrier(swapchain_image_list: &Vec<vk::Image>, image_index: usize, device: &Device, command_buffer: vk::CommandBuffer, ) {
        let mut swap_pres = vk::ImageMemoryBarrier { src_queue_family_index: vk::QUEUE_FAMILY_IGNORED, dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED, image: swapchain_image_list[image_index], subresource_range: vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, }, ..Default::default() };
            swap_pres.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            swap_pres.new_layout = vk::ImageLayout::PRESENT_SRC_KHR;
            swap_pres.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            swap_pres.dst_access_mask = vk::AccessFlags::MEMORY_READ;

        unsafe { device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::TOP_OF_PIPE, vk::DependencyFlags::empty(), &[], &[], &[swap_pres], ); }
    }

    pub fn record_command_pool(command_buffer_list: &Vec<vk::CommandBuffer>, device: &Device, compute_pipeline: vk::Pipeline, pipeline_layout: vk::PipelineLayout, descriptor_set_list: &Vec<vk::DescriptorSet>, extent: &vk::Extent2D, image: &ImageObj, swapchain_image_list: &Vec<vk::Image>, image_index: usize, ) {
        for command_buffer in command_buffer_list {
            let command_buffer = * command_buffer;
            let command_buffer_begin_info = vk::CommandBufferBeginInfo { flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, ..Default::default() };

            unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info, ).unwrap(); }
            unsafe { device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, compute_pipeline, ); }
            unsafe { device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline_layout, 0, &descriptor_set_list[..], &[], ); }
            unsafe { device.cmd_dispatch(command_buffer, extent.width / 16, extent.height / 16, 1, ); }

            Render::set_first_img_mem_barrier(image.image, swapchain_image_list, image_index, &device, command_buffer, );
            Render::copy_image_mem(extent, &device, command_buffer, image.image, swapchain_image_list, image_index, );
            Render::set_second_img_mem_barrier(swapchain_image_list, image_index, &device, command_buffer, );

            unsafe { device.end_command_buffer(command_buffer).unwrap(); }
        }
    }

    pub fn clean_up_swap_recreate(device: &Device, compute_pipeline: vk::Pipeline, pipeline_layout: vk::PipelineLayout, descriptor_set_layout_list: &Vec<vk::DescriptorSetLayout>, descriptor_pool: vk::DescriptorPool, image: &ImageObj, ) {
        unsafe { device.destroy_pipeline(compute_pipeline, None, ) }
        unsafe { device.destroy_pipeline_layout(pipeline_layout, None, ) }

        descriptor_set_layout_list.iter().for_each(| set_layout | unsafe { device.destroy_descriptor_set_layout(* set_layout, None, ) });
        unsafe { device.destroy_descriptor_pool(descriptor_pool, None, ) }

        unsafe { device.free_memory(image.image_mem, None, ) }
        unsafe { device.destroy_image_view(image.image_view, None, ) }
        unsafe { device.destroy_image(image.image, None, ) }
    }
}