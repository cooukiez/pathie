use ash::{extensions::khr::Swapchain, vk, Device, Instance};

use crate::pipe::image::{COMP_MAP, SUBRES_RANGE};

use super::surface::SurfaceGroup;

#[derive(Clone)]
pub struct SwapchainGroup {
    pub loader: Swapchain,
    pub swapchain: vk::SwapchainKHR,

    // Present image list
    pub img_list: Vec<vk::Image>,
    // Present image view list
    pub view_list: Vec<vk::ImageView>,
}

impl SwapchainGroup {
    /// Create new swapchain group object and initialize
    /// swapchain loader for further use. The other attrib.
    /// are set to default.

    pub fn new(instance: &Instance, device: &Device) -> Self {
        let loader = Swapchain::new(instance, device);

        Self {
            loader,
            swapchain: Default::default(),
            img_list: Default::default(),
            view_list: Default::default(),
        }
    }

    /// Create swapchain with loader. Will use information sepcified in
    /// surface group object to set swapchain prop.

    pub fn create_swapchain(&self, surface: &SurfaceGroup) -> Self {
        unsafe {
            let mut result = self.clone();

            let create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface.surface)
                .min_image_count(surface.swap_img_count)
                .image_color_space(surface.format.color_space)
                .image_format(surface.format.format)
                .image_extent(surface.surface_res)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(surface.pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(surface.present_mode)
                .clipped(true)
                .image_array_layers(1);

            result.swapchain = result.loader.create_swapchain(&create_info, None).unwrap();

            result
        }
    }

    /// Get the swapchain present image list and create the image view
    /// list for the image list. Then set both the attrib. in the swapchain
    /// group object.

    pub fn get_present_img(&self, surface: &SurfaceGroup, device: &Device) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Getting swapchain present image material ...");
            result.img_list = result
                .loader
                .get_swapchain_images(result.swapchain)
                .unwrap();

            log::info!("Creating image view list for present image list ...");
            result.view_list = result
                .img_list
                .iter()
                .map(|img| {
                    device
                        .create_image_view(
                            &vk::ImageViewCreateInfo::builder()
                                .view_type(vk::ImageViewType::TYPE_2D)
                                .format(surface.format.format)
                                .components(COMP_MAP)
                                .subresource_range(SUBRES_RANGE)
                                .image(*img),
                            None,
                        )
                        .unwrap()
                })
                .collect();

            result
        }
    }

    /// Destroy Swapchain and SwapchainImgList

    pub fn destroy(&self, device: &Device) {
        unsafe {
            self.view_list
                .iter()
                .for_each(|view| device.destroy_image_view(*view, None));

            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
