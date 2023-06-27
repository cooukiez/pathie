use ash::{vk, Device};

use crate::interface::{interface::Interface, phydev::PhyDeviceGroup};

#[derive(Clone)]
pub struct ImageTarget {
    pub img: vk::Image,
    pub view: vk::ImageView,

    pub mem: vk::DeviceMemory,
    pub mem_req: vk::MemoryRequirements,
    pub sampler: vk::Sampler,
}

/// Convert extent two dimensional to three dimensional
#[macro_export]
macro_rules! extent_conv {
    ($extent : expr) => {
        vk::Extent3D {
            width: $extent.width,
            height: $extent.height,
            depth: 1,
        }
    };
}

pub const SUBRES_RANGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: 1,
    base_array_layer: 0,
    layer_count: 1,
};

pub const COMP_MAP: vk::ComponentMapping = vk::ComponentMapping {
    r: vk::ComponentSwizzle::R,
    g: vk::ComponentSwizzle::G,
    b: vk::ComponentSwizzle::B,
    a: vk::ComponentSwizzle::A,
};

/// Basic object for creating two dimensional image to store
/// some sort of information. Not intended to be used as multi layer
/// texture to replace buffer, as said for simple picture material.

impl ImageTarget {
    /// This function will create image on device
    /// and will update the img attribute.

    pub fn create_img(
        &self,
        info: vk::ImageCreateInfo,
        device: &Device,
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            // Create Image on Device
            result.img = device.create_image(&info, None).unwrap();

            result
        }
    }

    pub fn create_img_memory(&self, device: &Device, phy_device: &PhyDeviceGroup) -> Self {
        unsafe {
            let mut result = self.clone();

            // Get Memory Requirement for Image and the MemoryTypeIndex
            result.mem_req = device.get_image_memory_requirements(result.img);
            let mem_index = phy_device
                .find_memorytype_index(&result.mem_req, vk::MemoryPropertyFlags::DEVICE_LOCAL)
                .expect("NO_SUITABLE_MEM_TYPE_INDEX");

            // Prepare MemoryAllocation
            let allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(result.mem_req.size)
                .memory_type_index(mem_index)
                .build();

            // Allocate Memory
            result.mem = device.allocate_memory(&allocate_info, None).unwrap();

            device
                .bind_image_memory(result.img, result.mem, 0)
                .expect("UNABLE_TO_BIND_MEM");

            result
        }
    }

    pub fn create_sampler(&self, info: vk::SamplerCreateInfo, device: &Device) -> Self {
        unsafe {
            let mut result = self.clone();

            result.sampler = device.create_sampler(&info, None).unwrap();

            result
        }
    }

    /// This function will create image on device
    /// and will update the img attribute.

    pub fn create_view(&self, info: vk::ImageViewCreateInfo, device: &Device) -> Self {
        unsafe {
            let mut result = self.clone();
            let mut info = info.clone();
            info.image = result.img;

            // Build image view
            result.view = device.create_image_view(&info, None).unwrap();

            result
        }
    }

    /// Create new image target with image, image view, image memory and
    /// image sampler. It is only intended to be used as two dimensional image.
    /// Will return new image target object.

    pub fn basic_img(interface: &Interface, extent: vk::Extent2D) -> Self {
        unsafe {
            let mut result = Self::default();

            let img_info = vk::ImageCreateInfo::builder()
                .format(interface.surface.format.format)
                .extent(extent_conv!(extent))
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .image_type(vk::ImageType::TYPE_2D)
                .build();

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
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
                .build();

            let view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(interface.surface.format.format)
                .subresource_range(SUBRES_RANGE)
                .components(COMP_MAP)
                .build();

            result = result
                .create_img(img_info, &interface.device)
                .create_img_memory(&interface.device, &interface.phy_device)
                .create_sampler(sampler_info, &interface.device)
                .create_view(view_info, &interface.device);

            result
        }
    }

    /// This function will update the descriptor in the gpu. This is done by
    /// creating a descriptor image info and then a write info. After that it will write the
    /// descriptor set.

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

    /// Destroy image and image view

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
            mem_req: Default::default(),
            sampler: Default::default(),
        }
    }
}
