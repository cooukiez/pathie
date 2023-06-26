use std::{ffi::c_void, mem::align_of};

use ash::{vk, util::Align};

use crate::interface::interface::Interface;

#[derive(Clone)]
pub struct BufferSet {
    pub buffer: vk::Buffer,
    pub buffer_mem: vk::DeviceMemory,
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