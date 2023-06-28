use std::ffi::c_void;

use ash::{util::Align, vk, Device};

use crate::interface::{interface::Interface, phydev::PhyDeviceGroup};

#[derive(Clone)]
pub struct BufferSet {
    pub buffer: vk::Buffer,

    pub mem: vk::DeviceMemory,
    pub mem_req: vk::MemoryRequirements,

    pub usage: vk::BufferUsageFlags,
    pub sharing_mode: vk::SharingMode,
}

impl BufferSet {
    pub fn create_memory<Type: Copy>(
        &self,
        device: &Device,
        phy_device: &PhyDeviceGroup,
        alignment: u64,
        size: u64,
        data: &[Type],
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            // Get MemoryRequirement
            result.mem_req = device.get_buffer_memory_requirements(result.buffer);

            let mem_idx = phy_device
                .find_memorytype_index(
                    &result.mem_req,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .expect("ERR_INDEX_BUFFER_MEM_INDEX");

            // Prepare Allocation
            let allocate_info = vk::MemoryAllocateInfo {
                allocation_size: result.mem_req.size,
                memory_type_index: mem_idx,

                ..Default::default()
            };

            // Create MemoryObject
            result.mem = device.allocate_memory(&allocate_info, None).unwrap();

            // Prepare MemoryCopy
            let buffer_ptr: *mut c_void = device
                .map_memory(
                    result.mem,
                    0,
                    result.mem_req.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();

            // Align memory
            let mut aligned_slice = Align::new(buffer_ptr, alignment, size);

            // Copy and finish Memory
            aligned_slice.copy_from_slice(&data);
            device.unmap_memory(result.mem);

            device
                .bind_buffer_memory(result.buffer, result.mem, 0)
                .unwrap();

            result
        }
    }

    /// Create new buffer set object with alignment, size in storage,
    /// usage, sharing mode and the actual buffer data.
    /// To finish, return the new buffer set object.

    pub fn new(
        buffer_size: u64,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        device: &Device,
    ) -> Self {
        unsafe {
            let mut result = Self::default();

            result.usage = usage;
            result.sharing_mode = sharing_mode;

            // BufferInfo
            let buffer_info = vk::BufferCreateInfo {
                size: buffer_size,
                usage,
                sharing_mode,

                ..Default::default()
            };

            // Create BufferObject
            result.buffer = device.create_buffer(&buffer_info, None).unwrap();

            result
        }
    }
}

impl Default for BufferSet {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            mem: Default::default(),
            mem_req: Default::default(),
            usage: Default::default(),
            sharing_mode: Default::default(),
        }
    }
}
