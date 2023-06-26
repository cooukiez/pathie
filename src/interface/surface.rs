use ash::{extensions::khr::Surface, vk, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

#[derive(Clone)]
pub struct SurfaceGroup {
    pub surface_loader: Surface,
    pub surface: vk::SurfaceKHR,
}

impl SurfaceGroup {
    pub fn new(entry: &Entry, instance: &Instance, window: &Window) -> SurfaceGroup {
        unsafe {
            let surface_loader = Surface::new(&entry, &instance);

            let surface = ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .unwrap();

            Self {
                surface_loader,
                surface,
            }
        }
    }
}