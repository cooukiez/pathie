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

    pub fn create_img(&self, info: vk::ImageCreateInfo, device: &Device) -> Self {
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
                .extent(extent.into())
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

    pub fn storage_texture(
        interface: &Interface,
        format: vk::Format,
        extent: vk::Extent3D,
        img_type: vk::ImageType,
        view_type: vk::ImageViewType,
        array_len: u32,
    ) -> Self {
        unsafe {
            let mut result = Self::default();

            let img_info = vk::ImageCreateInfo::builder()
                .format(format)
                .extent(extent)
                .mip_levels(1)
                .array_layers(array_len)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::STORAGE)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .image_type(img_type)
                .build();

            let sampler_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::NEAREST)
                .min_filter(vk::Filter::NEAREST)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
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
                .view_type(view_type)
                .format(img_info.format)
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

    pub fn attachment_img(interface: &Interface, extent: vk::Extent2D) -> Self {
        let mut result = Self::default();

        let img_info = vk::ImageCreateInfo::builder()
            .format(interface.surface.format.format)
            .extent(extent.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
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

    pub fn depth_img(interface: &Interface, extent: vk::Extent3D) -> Self {
        unsafe {
            let mut result = Self::default();

            let img_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::D16_UNORM)
                .extent(extent)
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();

            let view_info = vk::ImageViewCreateInfo::builder()
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .level_count(1)
                        .layer_count(1)
                        .build(),
                )
                .format(img_info.format)
                .view_type(vk::ImageViewType::TYPE_2D)
                .build();

            result = result
                .create_img(img_info, &interface.device)
                .create_img_memory(&interface.device, &interface.phy_device)
                .create_view(view_info, &interface.device);

            result
        }
    }

    /// Destroy image and image view

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.img, None);
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
