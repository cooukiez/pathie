use ash::{vk, Device};

#[derive(Clone)]
pub struct DescriptorPool {
    pub size_list: Vec<vk::DescriptorPoolSize>,
    pub layout_list: Vec<vk::DescriptorSetLayout>,

    pub set_list: Vec<vk::DescriptorSet>,

    pub pool: vk::DescriptorPool,
}

impl DescriptorPool {
    /// Desciptor describe some sort buffer like storage buffer.
    /// Descriptor set is group of descriptor.
    /// Specify the descriptor count for each storage type here.
    /// Uniform buffer count and storage buffer descriptor count.
    /// Max set is the max amount of sets in the pool.

    pub fn create_descriptor_pool(&self, device: &Device) -> Self {
        unsafe {
            let mut result = self.clone();

            // Specify descriptor count for each storage type
            let descriptor_size_list = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: result.image_desc_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: result.uniform_desc_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: result.storage_desc_count,
                },
            ];

            log::info!("Creating DescriptorPool ...");
            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&descriptor_size_list)
                .max_sets(result.set_list.len() as u32);

            result.pool = device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .unwrap();

            result
        }
    }

    /// Create descriptor set which is group of descriptor.
    /// Specify the type and count, could cause error if more used than
    /// expect in pool creation. Same goes for descriptor set. If set count
    /// is bigger than max set, it will throw an error.

    pub fn create_descriptor_set_layout(
        &self,
        descriptor_type: vk::DescriptorType,
        descriptor_count: u32,
        shader_stage: vk::ShaderStageFlags,
        device: &Device,
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            result.size_list.push(vk::DescriptorPoolSize {
                ty: descriptor_type,
                descriptor_count,
            });

            log::info!("Creating DescriptorSet ...");
            let set_binding_info = [vk::DescriptorSetLayoutBinding {
                descriptor_type,
                descriptor_count,
                stage_flags: shader_stage,
                ..Default::default()
            }];

            let desc_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&set_binding_info);

            result.layout_list.push(
                device
                    .create_descriptor_set_layout(&desc_info, None)
                    .unwrap(),
            );

            result
        }
    }
}

/*

let descriptor_pool = Self::create_descriptor_pool(1, 1, 2, 4, interface);

log::info!("Creating descriptor set layout list ...");
let desc_set_layout_list: Vec<vk::DescriptorSetLayout> = vec![
    // ImageTarget
    Self::create_descriptor_set_layout(
        vk::DescriptorType::STORAGE_IMAGE,
        1,
        vk::ShaderStageFlags::COMPUTE,
        interface,
    ),
    // Uniform Set
    Self::create_descriptor_set_layout(
        vk::DescriptorType::UNIFORM_BUFFER,
        1,
        vk::ShaderStageFlags::COMPUTE,
        interface,
    ),
    // Octree Set
    Self::create_descriptor_set_layout(
        vk::DescriptorType::STORAGE_BUFFER,
        1,
        vk::ShaderStageFlags::COMPUTE,
        interface,
    ),
];

let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
    .descriptor_pool(descriptor_pool)
    .set_layouts(&desc_set_layout_list);

let descriptor_set_list = interface
    .device
    .allocate_descriptor_sets(&desc_alloc_info)
    .unwrap();

log::info!("Writing descriptor list ...");
uniform_buffer.describe_in_gpu(
    interface,
    mem::size_of_val(&uniform_data) as u64,
    descriptor_set_list[1],
    0,
    vk::DescriptorType::UNIFORM_BUFFER,
);
octree_buffer.describe_in_gpu(
    interface,
    (mem::size_of::<u32>() * octree_data.len()) as u64,
    descriptor_set_list[2],
    0,
    vk::DescriptorType::STORAGE_BUFFER,
);

*/
