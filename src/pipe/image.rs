use ash::vk;

use crate::interface::interface::Interface;

#[derive(Clone)]
pub struct ImageTarget {
    pub img: vk::Image,
    pub view: vk::ImageView,

    pub mem: vk::DeviceMemory,
    pub sampler: vk::Sampler,
}

/// Basic object for creating two dimensional image to store
/// some sort of information. Not intended to be used as multi layer
/// texture to replace buffer, as said for simple picture material.

impl ImageTarget {
    /// This function will create a simple two dimensional image
    /// with tiling, sample count and more predefined for the
    /// intended usage. After that, we create the simple image on the device.

    pub fn create_img(
        interface: &Interface,
        extent: &vk::Extent2D,
        usage: vk::ImageUsageFlags,
        sharing_mode: vk::SharingMode,
        layout: vk::ImageLayout,
    ) -> vk::Image {
        unsafe {
            let extent = vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            };

            // Create ImgInfo with Dimension
            let image_info = vk::ImageCreateInfo::builder()
                .format(interface.surface.format.format)
                .extent(extent)
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(usage)
                .sharing_mode(sharing_mode)
                .initial_layout(layout)
                .image_type(vk::ImageType::TYPE_2D)
                .build();

            // Create Image on Device
            interface.device.create_image(&image_info, None).unwrap()
        }
    }

    /// This function will create the image view for a sepcified image.
    /// First we create the info with predefined subresource range,
    /// format and more for the intended usage. After that we create
    /// the image view on the device.

    pub fn create_view(interface: &Interface, img: vk::Image) -> vk::ImageView {
        unsafe {
            // Prepare image view for creation and bind image
            let image_view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(interface.surface.format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(img)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .build();

            // Build image view
            interface
                .device
                .create_image_view(&image_view_info, None)
                .unwrap()
        }
    }

    pub fn new(interface: &Interface, extent: vk::Extent2D) -> Self {
        unsafe {
            let img = Self::create_img(
                interface,
                &extent,
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
                vk::SharingMode::EXCLUSIVE,
                vk::ImageLayout::UNDEFINED,
            );

            // Get Memory Requirement for Image and the MemoryTypeIndex
            let mem_requirement = interface.device.get_image_memory_requirements(img);
            let mem_index = interface
                .find_memorytype_index(&mem_requirement, vk::MemoryPropertyFlags::DEVICE_LOCAL)
                .expect("NO_SUITABLE_MEM_TYPE_INDEX");

            // Prepare MemoryAllocation
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirement.size)
                .memory_type_index(mem_index)
                .build();

            // Allocate Memory therefore create DeviceMemory
            let mem = interface
                .device
                .allocate_memory(&allocate_info, None)
                .unwrap();

            // To Finish -> Bind Memory
            interface
                .device
                .bind_image_memory(img, mem, 0)
                .expect("UNABLE_TO_BIND_MEM");

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
                .mip_lod_bias(0.0)
                .max_anisotropy(1.0)
                .compare_op(vk::CompareOp::NEVER)
                .min_lod(0.0)
                .max_lod(1.0)
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

            let sampler = interface
                .device
                .create_sampler(&sampler_info, None)
                .unwrap();

            let view = Self::create_view(interface, img);

            Self {
                img,
                mem,
                sampler,
                view,
            }
        }
    }

    pub fn describe_in_gpu(
        &self,
        interface: &Interface,
        image_layout: vk::ImageLayout,
        dst_set: vk::DescriptorSet,
        dst_binding: u32,
        descriptor_type: vk::DescriptorType,
    ) {
        unsafe {
            let image_descriptor = vk::DescriptorImageInfo {
                image_view: self.view,
                image_layout,
                sampler: self.sampler,
            };

            let write_info = vk::WriteDescriptorSet {
                dst_set,
                dst_binding,
                descriptor_count: 1,
                descriptor_type,
                p_image_info: &image_descriptor,
                ..Default::default()
            };

            interface.device.update_descriptor_sets(&[write_info], &[]);
        }
    }

    pub fn destroy(&self, interface: &Interface) {
        unsafe {
            interface.device.destroy_image_view(self.view, None);
            interface.device.destroy_image(self.img, None);
        }
    }
}

impl Default for ImageTarget {
    fn default() -> Self {
        Self {
            img: Default::default(),
            view: Default::default(),
            mem: Default::default(),
            sampler: Default::default(),
        }
    }
}
