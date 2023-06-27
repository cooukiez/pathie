use std::{ffi::c_void};

use ash::{util::Align, vk};

use crate::interface::interface::Interface;

#[derive(Clone)]
pub struct BufferSet {
    pub buffer: vk::Buffer,
    pub buffer_mem: vk::DeviceMemory,

    pub alignment: u64,
    pub size: u64,

    pub usage: vk::BufferUsageFlags,
}

impl BufferSet {
    /// Create new buffer set object with alignment, size in storage,
    /// usage, sharing mode and the actual buffer data.
    /// To finish, return the new buffer set object.

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
            let buffer_ptr: *mut c_void = interface
                .device
                .map_memory(buffer_mem, 0, memory_req.size, vk::MemoryMapFlags::empty())
                .unwrap();

            // Align memory
            let mut aligned_slice = Align::new(buffer_ptr, alignment, memory_req.size);

            // Copy and finish Memory
            aligned_slice.copy_from_slice(&data);
            interface.device.unmap_memory(buffer_mem);

            interface
                .device
                .bind_buffer_memory(buffer, buffer_mem, 0)
                .unwrap();

            Self { buffer, buffer_mem }
        }
    }

    /// Update the data in buffer set object
    /// by remapping memory. Input data, alignment and size has to be
    /// known for this operation.

    pub fn update<Type: Copy>(
        &self,
        interface: &Interface,
        alignment: u64,
        size: u64,
        data: &[Type],
    ) {
        unsafe {
            let buffer_ptr = interface
                .device
                .map_memory(self.buffer_mem, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();

            // Align memory
            let mut aligned_slice = Align::new(buffer_ptr, alignment, size);

            aligned_slice.copy_from_slice(&data.clone()[..]);
            interface.device.unmap_memory(self.buffer_mem);
        }
    }

    /// This function will update the descriptor in the gpu. This is done by
    /// creating a descriptor buffer info and then a write info. After that it will write the
    /// descriptor set.

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
