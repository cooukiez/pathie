use ash::{extensions::khr::Surface, vk, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

use crate::Pref;

use super::phydev::PhyDeviceGroup;

#[derive(Clone)]
pub struct SurfaceGroup {
    pub loader: Surface,
    pub surface: vk::SurfaceKHR,

    pub format: vk::SurfaceFormatKHR,
    pub capa: vk::SurfaceCapabilitiesKHR,

    pub swap_img_count: u32,

    pub render_res: vk::Extent2D,
    pub surface_res: vk::Extent2D,

    pub pre_transform: vk::SurfaceTransformFlagsKHR,

    pub present_mode_list: Vec<vk::PresentModeKHR>,
    pub present_mode: vk::PresentModeKHR,
}

impl SurfaceGroup {
    /// Create new surface group object and immediatly create
    /// surface loader and surface khr.

    pub fn new(entry: &Entry, instance: &Instance, window: &Window) -> SurfaceGroup {
        unsafe {
            let loader = Surface::new(&entry, &instance);

            let surface = ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .unwrap();

            Self {
                loader,
                surface,

                format: Default::default(),
                capa: Default::default(),

                swap_img_count: 0,

                render_res: Default::default(),
                surface_res: Default::default(),

                pre_transform: Default::default(),

                present_mode_list: Default::default(),
                present_mode: Default::default(),
            }
        }
    }

    /// Set other param. of surface group. This function will
    /// gather information about surface format, surface capability,
    /// swapchain image count, surface resolution, surface pre transform and
    /// an availbable present mode list, from which it will select the
    /// preferred present mode if possible.

    pub fn get_surface_info(
        &self,
        phy_device: &PhyDeviceGroup,
        window: &Window,
        pref: &Pref,
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            result.format = result
                .loader
                .get_physical_device_surface_formats(phy_device.device, result.surface)
                .unwrap()[0];

            // Get surface Capability
            result.capa = result
                .loader
                .get_physical_device_surface_capabilities(phy_device.device, result.surface)
                .unwrap();

            result.swap_img_count = result.capa.min_image_count + 1;
            if result.capa.max_image_count > 0
                && result.swap_img_count > result.capa.max_image_count
            {
                result.swap_img_count = result.capa.max_image_count;
            }

            result = result.get_surface_res(&window, pref);

            result.pre_transform = if result
                .capa
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                result.capa.current_transform
            };

            result.present_mode_list = result
                .loader
                .get_physical_device_surface_present_modes(phy_device.device, result.surface)
                .unwrap();

            // Select preferred present mode if possible
            result.present_mode = result
                .present_mode_list
                .iter()
                .cloned()
                .find(|&mode| mode == pref.pref_present_mode)
                // Else use present mode fifo
                .unwrap_or(vk::PresentModeKHR::FIFO);

            result
        }
    }

    /// Function to get the resolution of the surface
    /// and the resolution at which to render.
    /// The resolution or scale factor can be changed in pref.

    pub fn get_surface_res(&self, window: &Window, pref: &Pref) -> Self {
        let mut result = self.clone();

        // Select new Dimension
        let dim = window.inner_size();
        result.surface_res = match result.capa.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: dim.width,
                height: dim.height,
            },
            _ => result.capa.current_extent,
        };

        // Select new RenderResolution
        result.render_res = if pref.use_render_res && window.fullscreen() != None {
            // Select render res only if fullscreen
            pref.render_res
        } else {
            // Else use scale factor defined in pref
            vk::Extent2D {
                width: (result.surface_res.width as f32 / pref.img_scale) as u32,
                height: (result.surface_res.height as f32 / pref.img_scale) as u32,
            }
        };

        result
    }
}
