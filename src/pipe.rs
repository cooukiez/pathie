use std::{ffi::{c_void, CString}, mem::{align_of, self}, io::Cursor, error::Error};

use ash::{vk::{self, DescriptorSetLayout, DescriptorSet, ImageAspectFlags}, util::{Align, read_spv}};
use rand::Rng;

use crate::{interface::Interface, offset_of, octree::{Octree, TreeNode}, uniform::Uniform, Pref, DEFAULT_STORAGE_BUFFER_SIZE, DEFAULT_UNIFORM_BUFFER_SIZE};

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
}

pub struct Pipe {
    pub scaled_extent: vk::Extent2D,
    pub image_target_list: Vec<vk::Image>,
    pub image_target_mem_list: Vec<vk::DeviceMemory>,
    pub image_target_view_list: Vec<vk::ImageView>,

    pub index_buffer_data: Vec<u32>,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub vertex_input_buffer: vk::Buffer,
    pub vertex_input_buffer_memory: vk::DeviceMemory,
    pub uniform_buffer: vk::Buffer,
    pub uniform_buffer_memory: vk::DeviceMemory,

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
    pub fn init(interface: &Interface, pref: &Pref, uniform: &mut Uniform, ) -> Pipe {
        unsafe {
            let scaled_extent = vk::Extent2D { 
                width: (interface.surface_resolution.width as f32 / pref.img_scale) as u32,
                height: (interface.surface_resolution.height as f32 / pref.img_scale) as u32
            };

            uniform.apply_resolution(scaled_extent);

            log::info!("Getting ImageTarget List ...");
            let image_target_list: Vec<vk::Image> = interface.present_img_view_list
                .iter()
                .map(| _ | {
                    // Create ImgInfo with Dimension -> Scaled Variant
                    let image_info = vk::ImageCreateInfo::builder()
                        .format(interface.surface_format.format)
                        .extent(vk::Extent3D { width: scaled_extent.width, height: scaled_extent.height, depth: 1 })
                        .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL).usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED)
                        .image_type(vk::ImageType::TYPE_2D)
                        .build();
                    // Create Image on Device
                    interface.device
                        .create_image(&image_info, None, )
                        .unwrap()
                })
                .collect();
            
            log::info!("Getting ImageTargetMemory List ...");
            let image_target_mem_list: Vec<vk::DeviceMemory> = image_target_list
                .iter()
                .map(| &image_target | {
                    // Get Memory Requirement for Image and the MemoryTypeIndex
                    let img_mem_requirement = interface.device.get_image_memory_requirements(image_target);
                    let img_mem_index = interface
                        .find_memorytype_index(&img_mem_requirement, vk::MemoryPropertyFlags::DEVICE_LOCAL, )
                        .expect("NO_SUITABLE_MEM_TYPE_INDEX");
                    // Prepare MemoryAllocation
                    let allocate_info = vk::MemoryAllocateInfo::builder()
                        .allocation_size(img_mem_requirement.size)
                        .memory_type_index(img_mem_index).build();
                    // Allocate Memory therefore create DeviceMemory
                    let image_mem = interface.device
                        .allocate_memory(&allocate_info, None, )
                        .unwrap();
                    // To Finish -> Bind Memory
                    interface.device
                        .bind_image_memory(image_target, image_mem, 0, )
                        .expect("UNABLE_TO_BIND_MEM");
                    image_mem
                })
                .collect();
            
            log::info!("Getting ImageTargetView List ...");
            let image_target_view_list: Vec<vk::ImageView> = image_target_list
                .iter()
                .map(| &image_target | {
                    // Prepare ImageView Creation and bind Image
                    let image_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(interface.surface_format.format)
                        .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, })
                        .image(image_target)
                        .components(vk::ComponentMapping { r: vk::ComponentSwizzle::R, g: vk::ComponentSwizzle::G, b: vk::ComponentSwizzle::B, a: vk::ComponentSwizzle::A, })
                        .build();
                    // Build Image View
                    interface.device
                        .create_image_view(&image_view_info, None, )
                        .unwrap()
                })
                .collect();

            // Create Index Buffer
            log::info!("Creating IndexBuffer ...");
            let index_buffer_data: Vec<u32> = vec![0u32, 1, 2, 2, 3, 0];
            let index_buffer_info = vk::BufferCreateInfo { size: std::mem::size_of_val(&index_buffer_data) as u64, usage: vk::BufferUsageFlags::INDEX_BUFFER, sharing_mode: vk::SharingMode::EXCLUSIVE, ..Default::default() };

            let index_buffer = interface.device.create_buffer(&index_buffer_info, None).unwrap();
            let index_buffer_memory_req = interface.device.get_buffer_memory_requirements(index_buffer);
            let index_buffer_memory_index = 
                interface.find_memorytype_index(&index_buffer_memory_req, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, )
                .expect("ERR_INDEX_BUFFER_MEM_INDEX");
            
            let index_allocate_info = vk::MemoryAllocateInfo { allocation_size: index_buffer_memory_req.size, memory_type_index: index_buffer_memory_index, ..Default::default() };
            let index_buffer_memory = interface.device
                .allocate_memory(&index_allocate_info, None, )
                .unwrap();
            let index_ptr: *mut c_void = interface.device
                .map_memory(index_buffer_memory, 0, index_buffer_memory_req.size, vk::MemoryMapFlags::empty(), )
                .unwrap();
            let mut index_slice = Align::new(index_ptr, align_of::<u32>() as u64, index_buffer_memory_req.size, );

            index_slice.copy_from_slice(&index_buffer_data);
            interface.device.unmap_memory(index_buffer_memory);

            interface.device
                .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
                .unwrap();

            // Create Vertex Buffer
            log::info!("Creating VertexBuffer ...");
            let vertex_list = [
                Vertex { pos: [-1.0, -1.0, 0.0, 1.0], uv: [0.0, 0.0], },
                Vertex { pos: [-1.0, 1.0, 0.0, 1.0], uv: [0.0, 1.0], },
                Vertex { pos: [1.0, 1.0, 0.0, 1.0], uv: [1.0, 1.0], },
                Vertex { pos: [1.0, -1.0, 0.0, 1.0], uv: [1.0, 0.0], },
            ];

            let vertex_input_buffer_info = vk::BufferCreateInfo { size: std::mem::size_of_val(&vertex_list) as u64, usage: vk::BufferUsageFlags::VERTEX_BUFFER, sharing_mode: vk::SharingMode::EXCLUSIVE, ..Default::default() };
            let vertex_input_buffer = interface.device
                .create_buffer(&vertex_input_buffer_info, None, )
                .unwrap();
            let vertex_input_buffer_memory_req = interface.device
                .get_buffer_memory_requirements(vertex_input_buffer);
            let vertex_input_buffer_memory_index = 
                interface.find_memorytype_index(&vertex_input_buffer_memory_req, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, )
                .expect("ERR_VERTEX_MEM_INDEX");
            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo { allocation_size: vertex_input_buffer_memory_req.size, memory_type_index: vertex_input_buffer_memory_index, ..Default::default() };
            let vertex_input_buffer_memory = interface.device
                .allocate_memory(&vertex_buffer_allocate_info, None, )
                .unwrap();
            
            let vert_ptr = interface.device
                .map_memory(vertex_input_buffer_memory, 0, vertex_input_buffer_memory_req.size, vk::MemoryMapFlags::empty(), )
                .unwrap();
            let mut slice = Align::new(vert_ptr, align_of::<Vertex>() as u64, vertex_input_buffer_memory_req.size, );
            slice.copy_from_slice(&vertex_list);
            interface.device.unmap_memory(vertex_input_buffer_memory);
            interface.device
                .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0, )
                .unwrap();
            
            // Create Uniform Buffer
            log::info!("Creating UniformBuffer ...");
            let uniform_buffer_data = uniform.clone();
            let uniform_buffer_info = vk::BufferCreateInfo { size: DEFAULT_UNIFORM_BUFFER_SIZE, usage: vk::BufferUsageFlags::UNIFORM_BUFFER, sharing_mode: vk::SharingMode::EXCLUSIVE, ..Default::default() };
            
            let uniform_buffer = interface.device
                .create_buffer(&uniform_buffer_info, None, )
                .unwrap();
            let uniform_buffer_memory_req = interface.device
                .get_buffer_memory_requirements(uniform_buffer);
            let uniform_buffer_memory_index = interface.find_memorytype_index(&uniform_buffer_memory_req, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, )
                .expect("ERR_UNIFORM_MEM_INDEX");
            let uniform_buffer_allocate_info = vk::MemoryAllocateInfo { allocation_size: uniform_buffer_memory_req.size, memory_type_index: uniform_buffer_memory_index, ..Default::default() };
            let uniform_buffer_memory = interface.device
                .allocate_memory(&uniform_buffer_allocate_info, None, )
                .unwrap();
            let uniform_ptr = interface.device
                .map_memory(uniform_buffer_memory, 0, uniform_buffer_memory_req.size, vk::MemoryMapFlags::empty(), )
                .unwrap();
            let mut uniform_aligned_slice = Align::new(uniform_ptr, align_of::<Uniform>() as u64, uniform_buffer_memory_req.size, );
            uniform_aligned_slice.copy_from_slice(&[uniform_buffer_data]);
            interface.device.unmap_memory(uniform_buffer_memory);
            interface.device
                .bind_buffer_memory(uniform_buffer, uniform_buffer_memory, 0, )
                .unwrap();

            // Create Octree Buffer
            log::info!("Creating OctreeBuffer ...");
            let mut octree = Octree::empty(&uniform);
            octree.collect_random(3);
            let octree_buffer_data = octree.data.clone();
            
            let octree_buffer_info = vk::BufferCreateInfo { size: DEFAULT_STORAGE_BUFFER_SIZE, usage: vk::BufferUsageFlags::STORAGE_BUFFER, sharing_mode: vk::SharingMode::EXCLUSIVE, ..Default::default() };
            
            let octree_buffer = interface.device
                .create_buffer(&octree_buffer_info, None, )
                .unwrap();
            let octree_buffer_memory_req = interface.device
                .get_buffer_memory_requirements(octree_buffer);
            let octree_buffer_memory_index = interface.find_memorytype_index(&octree_buffer_memory_req, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, )
                .expect("ERR_OCTREE_MEM_INDEX");
            let octree_buffer_allocate_info = vk::MemoryAllocateInfo { allocation_size: octree_buffer_memory_req.size, memory_type_index: octree_buffer_memory_index, ..Default::default() };
            let octree_buffer_memory = interface.device
                .allocate_memory(&octree_buffer_allocate_info, None, )
                .unwrap();
            let octree_buffer_ptr = interface.device
                .map_memory(octree_buffer_memory, 0, octree_buffer_memory_req.size, vk::MemoryMapFlags::empty(), )
                .unwrap();
            let mut octree_aligned_slice = Align::new(octree_buffer_ptr, align_of::<TreeNode>() as u64, octree_buffer_memory_req.size, );
            octree_aligned_slice.copy_from_slice(&octree_buffer_data[ .. ]);
            interface.device.unmap_memory(octree_buffer_memory);
            interface.device
                .bind_buffer_memory(octree_buffer, octree_buffer_memory, 0, )
                .unwrap();

            // Create DescriptorSet
            log::info!("Creating DescriptorPool ...");
            let descriptor_size_list = [
                vk::DescriptorPoolSize { ty: vk::DescriptorType::UNIFORM_BUFFER, descriptor_count: 1, },
                vk::DescriptorPoolSize { ty: vk::DescriptorType::STORAGE_BUFFER, descriptor_count: 1, },
            ];

            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&descriptor_size_list)
                .max_sets(descriptor_size_list.len() as u32);
            let descriptor_pool = interface.device
                .create_descriptor_pool(&descriptor_pool_info, None, )
                .unwrap();
            
            let uniform_set_binding_list = [
                vk::DescriptorSetLayoutBinding { descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, descriptor_count: 1, stage_flags: vk::ShaderStageFlags::FRAGMENT, ..Default::default() },
            ];

            let octree_set_binding_list = [
                vk::DescriptorSetLayoutBinding { descriptor_type: vk::DescriptorType::STORAGE_BUFFER, descriptor_count: 1, stage_flags: vk::ShaderStageFlags::FRAGMENT, ..Default::default() },
            ];

            let uniform_desc_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&uniform_set_binding_list);
            let octree_dec_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&octree_set_binding_list);

            let desc_set_layout_list: Vec<vk::DescriptorSetLayout> = vec![
                interface.device
                    .create_descriptor_set_layout(&uniform_desc_info, None, )
                    .unwrap(),
                interface.device
                    .create_descriptor_set_layout(&octree_dec_info, None, )
                    .unwrap(),
            ];

            let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&desc_set_layout_list);
            let descriptor_set_list = interface.device
                .allocate_descriptor_sets(&desc_alloc_info)
                .unwrap();

            let uniform_buffer_descriptor = vk::DescriptorBufferInfo { buffer: uniform_buffer, offset: 0, range: mem::size_of_val(&uniform_buffer_data) as u64, };
            let octree_buffer_descriptor = vk::DescriptorBufferInfo { buffer: octree_buffer, offset: 0, range: mem::size_of_val(&octree_buffer_data) as u64, };

            let write_desc_set_list = [
                vk::WriteDescriptorSet { dst_set: descriptor_set_list[0], descriptor_count: 1, descriptor_type: vk::DescriptorType::UNIFORM_BUFFER, p_buffer_info: &uniform_buffer_descriptor, ..Default::default() },
                vk::WriteDescriptorSet { dst_set: descriptor_set_list[1], descriptor_count: 1, descriptor_type: vk::DescriptorType::STORAGE_BUFFER, p_buffer_info: &octree_buffer_descriptor, ..Default::default() },
            ];

            interface.device.update_descriptor_sets(&write_desc_set_list, &[], );

            log::info!("Getting ShaderCode ...");
            let mut vertex_spv_file = Cursor::new(&include_bytes!("../shader/new/vert.spv")[..]);
            let mut frag_spv_file = Cursor::new(&include_bytes!("../shader/new/frag.spv")[..]);

            let vertex_code = read_spv(&mut vertex_spv_file).expect("ERR_READ_VERTEX_SPV");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            let frag_code = read_spv(&mut frag_spv_file).expect("ERR_READ_FRAG_SPV");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            let vertex_shader_module = interface.device
                .create_shader_module(&vertex_shader_info, None, )
                .expect("ERR_VERTEX_MODULE");

            let fragment_shader_module = interface.device
                .create_shader_module(&frag_shader_info, None, )
                .expect("ERR_FRAG_MODULE");

            let layout_create_info =
                vk::PipelineLayoutCreateInfo::builder().set_layouts(&desc_set_layout_list);

            log::info!("Creating PipelineLayout ...");
            let pipeline_layout = interface.device
                .create_pipeline_layout(&layout_create_info, None, )
                .unwrap();

            log::info!("Stage Creation ...");
            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage_info_list = [
                vk::PipelineShaderStageCreateInfo { module: vertex_shader_module, p_name: shader_entry_name.as_ptr(), stage: vk::ShaderStageFlags::VERTEX, ..Default::default() },
                vk::PipelineShaderStageCreateInfo { module: fragment_shader_module, p_name: shader_entry_name.as_ptr(), stage: vk::ShaderStageFlags::FRAGMENT, ..Default::default() },
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

            log::info!("Viewport and Scissor ...");
            let viewport = vec![
                vk::Viewport { 
                    x: 0.0, y: 0.0,
                    width: scaled_extent.width as f32,
                    height: scaled_extent.height as f32,
                    min_depth: 0.0, max_depth: 1.0,
                }
            ];

            let scissor: Vec<vk::Rect2D> = vec![scaled_extent.into()];
            let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
                .scissors(&scissor)
                .viewports(&viewport);

            log::info!("Rasterization ...");
            let rasterization_info = vk::PipelineRasterizationStateCreateInfo { front_face: vk::FrontFace::COUNTER_CLOCKWISE, line_width: 1.0, polygon_mode: vk::PolygonMode::FILL, ..Default::default() };
            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            
            log::info!("Blending ...");
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

            let graphic_pipeline_list = interface.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info_list], None, )
                .unwrap();

            log::info!("Rendering initialisation finished ...");
            Pipe {
                scaled_extent,
                image_target_list,
                image_target_mem_list,
                image_target_view_list,
                index_buffer_data,
                index_buffer,
                index_buffer_memory,
                vertex_input_buffer,
                vertex_input_buffer_memory,
                uniform_buffer,
                uniform_buffer_memory,
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

    pub fn draw(&self, interface: &Interface, pref: &Pref, ) -> Result<bool, Box<dyn Error>> {
        unsafe {
            let next_image = interface.swapchain_loader
                .acquire_next_image(interface.swapchain, std::u64::MAX, interface.present_complete_semaphore, vk::Fence::null(), );

            let present_index = 
                match next_image {
                    Ok((present_index, _, )) => present_index,
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return Ok(true); },
                    Err(error) => panic!("ERROR_AQUIRE_IMAGE -> {}", error, ),
                };

            let clear_value = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0], }, }];

            interface.device
                .wait_for_fences(&[interface.draw_command_fence], true, std::u64::MAX)
                .expect("DEVICE_LOST");

            interface.device
                .reset_fences(&[interface.draw_command_fence])
                .expect("FENCE_RESET_FAILED");
    
            interface.device
                .reset_command_buffer(interface.draw_command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES, )
                .expect("ERR_RESET_CMD_BUFFER");
    
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    
            interface.device
                .begin_command_buffer(interface.draw_command_buffer, &command_buffer_begin_info)
                .expect("ERR_BEGIN_CMD_BUFFER");

            let color_attachment_info = vk::RenderingAttachmentInfoKHR::builder()
                .image_view(self.image_target_view_list[present_index as usize])
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value[0])
                .build();

            let color_attachment_list = [color_attachment_info];

            // Begin Draw
            let rendering_info = vk::RenderingInfoKHR::builder()
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.scaled_extent })
                .layer_count(1)
                .color_attachments(&color_attachment_list);

            // Pipe Rendering Part
            interface.device
                .cmd_begin_rendering(interface.draw_command_buffer, &rendering_info);
            interface.device
                .cmd_bind_descriptor_sets(interface.draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &self.descriptor_set_list[..], &[], );
            interface.device
                .cmd_bind_pipeline(interface.draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphic_pipeline, );
            interface.device
                .cmd_set_viewport(interface.draw_command_buffer, 0, &self.viewport, );
            interface.device
                .cmd_set_scissor(interface.draw_command_buffer, 0, &self.scissor, );
            interface.device
                .cmd_bind_vertex_buffers(interface.draw_command_buffer, 0, &[self.vertex_input_buffer], &[0], );
            interface.device
                .cmd_bind_index_buffer(interface.draw_command_buffer, self.index_buffer, 0, vk::IndexType::UINT32, );
            interface.device
                .cmd_draw_indexed(interface.draw_command_buffer, self.index_buffer_data.len() as u32, 1, 0, 0, 1, );
            interface.device
                .cmd_end_rendering(interface.draw_command_buffer);

            let src = vk::ImageSubresourceLayers { aspect_mask: ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
            let dst = vk::ImageSubresourceLayers { aspect_mask: ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
            
            let blit = vk::ImageBlit { 
                src_subresource: src, 
                src_offsets: [vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: self.scaled_extent.width as i32, y: self.scaled_extent.height as i32, z: 1 }], 
                dst_subresource: dst, 
                dst_offsets: [vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: interface.surface_resolution.width as i32, y: interface.surface_resolution.height as i32, z: 1 }] };
            
            interface.device
                .cmd_blit_image(interface.draw_command_buffer, self.image_target_list[present_index as usize], vk::ImageLayout::UNDEFINED, interface.present_img_list[present_index as usize], vk::ImageLayout::UNDEFINED, &[blit], pref.img_scale_filter, );
                
            // Finish Draw
            interface.device
                .end_command_buffer(interface.draw_command_buffer)
                .expect("ERR_END_CMD_BUFFER");
    
            let command_buffer_list: Vec<vk::CommandBuffer> = vec![interface.draw_command_buffer];
    
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[interface.present_complete_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::BOTTOM_OF_PIPE])
                .command_buffers(&command_buffer_list)
                .signal_semaphores(&[interface.rendering_complete_semaphore])
                .build();
    
            interface.device
                .queue_submit(interface.present_queue, &[submit_info], interface.draw_command_fence)
                .expect("QUEUE_SUBMIT_FAILED");

            let present_info = vk::PresentInfoKHR { wait_semaphore_count: 1, p_wait_semaphores: &interface.rendering_complete_semaphore, swapchain_count: 1, p_swapchains: &interface.swapchain, p_image_indices: &present_index, ..Default::default() };

            let present_result = interface.swapchain_loader
                .queue_present(interface.present_queue, &present_info);
            
            match present_result {
                Ok(is_suboptimal) if is_suboptimal => { return Ok(true); },
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return Ok(true); },
                Err(error) => panic!("ERROR_PRESENT_SWAP -> {}", error, ), _ => { },
            }
            
            Ok(false)
        }
    }

    pub fn recreate_swapchain(&mut self, interface: &mut Interface, uniform: &mut Uniform, pref: &Pref, ) {
        unsafe {
            interface.wait_for_gpu().expect("DEVICE_LOST");

            log::info!("Recreating Swapchain ...");

            // Destroy ImageTargetViewList
            self.image_target_view_list
            .iter()
            .for_each(| &image_target_view | {
                interface.device
                    .destroy_image_view(image_target_view, None, );
            });

            // Destroy ImageTargetList
            self.image_target_list
                .iter()
                .for_each(| &image_target | {
                    interface.device
                        .destroy_image(image_target, None, );
                });
            
            // Destroy PresentImgViewList
            interface.present_img_view_list
                .iter()
                .for_each(| view | interface.device.destroy_image_view(* view, None, ));
            // Destroy Swapchain
            interface.swapchain_loader
                .destroy_swapchain(interface.swapchain, None, );
            
            // Get new SurfaceCapability
            interface.surface_capability = interface.surface_loader
                .get_physical_device_surface_capabilities(interface.phy_device, interface.surface, )
                .unwrap();

            // Get new Dimension and check
            let dim = interface.window.inner_size();
            interface.surface_resolution = match interface.surface_capability.current_extent.width {
                std::u32::MAX => vk::Extent2D { width: dim.width, height: dim.height },
                _ => interface.surface_capability.current_extent,
            };

            // Select PresentMode -> PreferredPresentMode is selected in Pref
            let present_mode = interface.present_mode_list
                .iter()
                .cloned()
                .find(| &mode | mode == pref.pref_present_mode)
                .unwrap_or(vk::PresentModeKHR::FIFO);

            // Create Info for new Swapchain with different Surface
            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(interface.surface)
                .min_image_count(interface.desired_image_count)
                .image_color_space(interface.surface_format.color_space)
                .image_format(interface.surface_format.format)
                .image_extent(interface.surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(interface.pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            // Create Swapchain
            interface.swapchain = interface.swapchain_loader
                .create_swapchain(&swapchain_create_info, None, )
                .unwrap();

            // Get the new PresentImgList
            interface.present_img_list = interface.swapchain_loader
                .get_swapchain_images(interface.swapchain)
                .unwrap();
            // Create new PresentImgViewList for PresentImgList
            interface.present_img_view_list = interface.present_img_list
                .iter()
                .map(| &image | {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(interface.surface_format.format)
                        .components(vk::ComponentMapping { r: vk::ComponentSwizzle::R, g: vk::ComponentSwizzle::G, b: vk::ComponentSwizzle::B, a: vk::ComponentSwizzle::A, })
                        .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, })
                        .image(image);
                    interface.device
                        .create_image_view(&create_view_info, None, )
                        .unwrap()
                })
                .collect();

            // Apply Scale
            self.scaled_extent = vk::Extent2D { 
                width: (interface.surface_resolution.width as f32 / pref.img_scale) as u32,
                height: (interface.surface_resolution.height as f32 / pref.img_scale) as u32
            };

            uniform.apply_resolution(self.scaled_extent);

            self.image_target_list = interface.present_img_view_list
                .iter()
                .map(| _ | {
                    // Create ImgInfo with Dimension -> Scaled Variant
                    let image_info = vk::ImageCreateInfo::builder()
                        .format(interface.surface_format.format)
                        .extent(vk::Extent3D { width: self.scaled_extent.width, height: self.scaled_extent.height, depth: 1 })
                        .mip_levels(1).array_layers(1).samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL).usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED)
                        .image_type(vk::ImageType::TYPE_2D)
                        .build();
                    // Create Image on Device
                    interface.device
                        .create_image(&image_info, None, )
                        .unwrap()
                })
                .collect();
            
            self.image_target_mem_list = self.image_target_list
                .iter()
                .map(| &image_target | {
                    // Get Memory Requirement for Image and the MemoryTypeIndex
                    let img_mem_requirement = interface.device.get_image_memory_requirements(image_target);
                    let img_mem_index = interface
                        .find_memorytype_index(&img_mem_requirement, vk::MemoryPropertyFlags::DEVICE_LOCAL, )
                        .expect("NO_SUITABLE_MEM_TYPE_INDEX");
                    // Prepare MemoryAllocation
                    let allocate_info = vk::MemoryAllocateInfo::builder()
                        .allocation_size(img_mem_requirement.size)
                        .memory_type_index(img_mem_index).build();
                    // Allocate Memory therefore create DeviceMemory
                    let image_mem = interface.device
                        .allocate_memory(&allocate_info, None, )
                        .unwrap();
                    // To Finish -> Bind Memory
                    interface.device
                        .bind_image_memory(image_target, image_mem, 0, )
                        .expect("UNABLE_TO_BIND_MEM");
                    image_mem
                })
                .collect();
            
            self.image_target_view_list = self.image_target_list
                .iter()
                .map(| &image_target | {
                    // Prepare ImageView Creation and bind Image
                    let image_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(interface.surface_format.format)
                        .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1, })
                        .image(image_target)
                        .components(vk::ComponentMapping { r: vk::ComponentSwizzle::R, g: vk::ComponentSwizzle::G, b: vk::ComponentSwizzle::B, a: vk::ComponentSwizzle::A, })
                        .build();
                    // Build Image View
                    interface.device
                        .create_image_view(&image_view_info, None, )
                        .unwrap()
                })
                .collect();

            self.viewport = vec![
                vk::Viewport { 
                    x: 0.0, y: 0.0,
                    width: self.scaled_extent.width as f32,
                    height: self.scaled_extent.height as f32,
                    min_depth: 0.0, max_depth: 1.0,
                }
            ];
    
            self.scissor = vec![self.scaled_extent.into()];
        }
    }

    pub fn update_buffer<Type : Copy>(&mut self, interface: &Interface, buffer_mem: vk::DeviceMemory, data: &[Type]) {
        unsafe {
            let octree_buffer_ptr = interface.device
                    .map_memory(buffer_mem, 0, std::mem::size_of_val(data) as u64, vk::MemoryMapFlags::empty(), )
                    .unwrap();
            let mut octree_aligned_slice = 
                Align::new(octree_buffer_ptr, align_of::<Type>() as u64, std::mem::size_of_val(data) as u64, );
            octree_aligned_slice.copy_from_slice(&data.clone()[ .. ]);
            interface.device.unmap_memory(buffer_mem);
        }
    }
}