use std::error::Error;
use ash::{khr, vk, Entry, Instance};
use ash::khr::surface;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct RenderSurface {
    handle: vk::SurfaceKHR,
    loader: surface::Instance,
}

impl RenderSurface {
    pub fn new(
        entry: &Entry, 
        instance: &Instance, 
        window: &winit::window::Window
    ) -> Result<Self, Box<dyn Error>> {

        let handle = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None)
        }.unwrap();

        let loader = khr::surface::Instance::new(&entry, &instance);
        Ok(Self{
            handle,
            loader,
        })
    }
}