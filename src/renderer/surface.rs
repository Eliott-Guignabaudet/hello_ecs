use std::error::Error;
use ash::{khr, vk, Entry, Instance};
use ash::khr::surface;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct RenderSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: surface::Instance,
}

impl RenderSurface {
    pub fn new(
        entry: &Entry, 
        instance: &Instance, 
        window: &winit::window::Window
    ) -> Result<Self, Box<dyn Error>> {

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None)
        }.unwrap();

        let surface_loader = khr::surface::Instance::new(&entry, &instance);
        Ok(Self{
            surface,
            surface_loader,
        })
    }
}