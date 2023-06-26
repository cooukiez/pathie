use std::ffi::CStr;

use ash::{extensions::khr::Surface, vk, Instance};

use super::surface::SurfaceGroup;

#[derive(Clone)]
pub struct PhyDeviceGroup {
    pub device_list: Vec<vk::PhysicalDevice>,

    pub device: vk::PhysicalDevice,
    pub queue_family_index: u32,

    pub device_prop: vk::PhysicalDeviceProperties,
    pub mem_prop: vk::PhysicalDeviceMemoryProperties,
    pub feature: vk::PhysicalDeviceFeatures, 
}

impl PhyDeviceGroup {
    /// Get available physical device list,
    /// for example your dedicated GPU or integrated GPU.
    /// Then set the constructor in the PhyDevice object.

    pub fn get_phy_device_list(&self, instance: &Instance) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Getting available physical device list ...");
            result.device_list = instance
                .enumerate_physical_devices()
                .expect("ERR_NO_PHY_DEVICE");

            result
        }
    }

    /// Check if physical device is suitable.
    /// Return None if not suitable or return device and index,
    /// if suitable.
    ///
    /// This function primarily checks if there is any graphic support
    /// in the available queue family.
    ///
    /// *Add Other criteria for device selection here*

    pub fn is_device_suitable(
        &self,
        info: &vk::QueueFamilyProperties,
        surface: &SurfaceGroup,
        device: &vk::PhysicalDevice,
        index: usize,
    ) -> Option<(vk::PhysicalDevice, u32)> {
        unsafe {
            // Check for graphic queue support
            let supported = info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && surface.loader
                    .get_physical_device_surface_support(*device, index as u32, surface.surface)
                    .unwrap();

            // Return device and index if suitable
            if supported {
                // Convert because will later be used different
                Some((*device, index as u32))
            } else {
                None
            }
        }
    }

    /// This function will set the physical device in the
    /// physical device group object.
    ///
    /// We first go thru the entire physical device list. After
    /// that we check for the queue family support for graphic.
    ///
    /// If not suitable device is found, we throw an exception,
    /// because then the application won't be able to run.

    pub fn get_suitable_phy_device(
        &self,
        instance: &Instance,
        surface: &SurfaceGroup,
    ) -> Self {
        unsafe {
            let mut result = self.clone();

            log::info!("Trying to find suitable physical device ...");

            (result.device, result.queue_family_index) = result
                // Iterate over physical device list
                .device_list
                .iter()
                .find_map(|device| {
                    instance
                        // Get queue family prop
                        .get_physical_device_queue_family_properties(*device)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            // Check if device is suitable
                            self.is_device_suitable(
                                info,
                                surface,
                                device,
                                index,
                            )
                        })
                })
                .expect("NO_SUITABLE_PHY_DEVICE");

            result
        }
    }

    /// This function will set the physical device prop,
    /// physical device memory prop and physical device feature
    /// attrib. in the physical device group object.

    pub fn get_phy_device_prop(&self, instance: &Instance) -> Self {
        unsafe {
            let mut result = self.clone();

            result.device_prop = instance.get_physical_device_properties(result.device);
            result.mem_prop =
                instance.get_physical_device_memory_properties(result.device);
                result.feature = instance.get_physical_device_features(result.device);

            log::info!(
                "Selected physical device -> {}",
                &CStr::from_ptr(result.device_prop.device_name.as_ptr())
                    .to_str()
                    .unwrap()
            );

            result
        }
    }
}

impl Default for PhyDeviceGroup {
    fn default() -> Self {
        Self {
            device_list: Default::default(),

            device: Default::default(),
            queue_family_index: 0,

            device_prop: Default::default(),
            mem_prop: Default::default(),
            feature: Default::default(),
        }
    }
}
