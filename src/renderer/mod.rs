mod constants;
mod app;
#[path = "core-renderer.rs"]
mod core_renderer;
mod instance;
mod surface;
mod device;
mod swapchain;
mod utils;
mod frame_resources;
mod command_pool;
mod buffer;
mod uniform_buffer;
mod descriptor;
mod render_pass;
mod image;

use std::error::Error;
pub use app::RenderApp;


use instance::RenderInstance;
use surface::RenderSurface;
use device::RenderDevice;
use swapchain::RenderSwapchain;

pub struct HelloRenderer {
    instance: RenderInstance,
    surface: RenderSurface,
    device: RenderDevice,
    swapchain: RenderSwapchain,
    
    
}

impl HelloRenderer {
    fn new(window: &winit::window::Window) -> Result<Self, Box<dyn Error>> {
        let instance = RenderInstance::new(window)?;
        let surface = RenderSurface::new(&instance.entry, &instance.instance, window)?;
        let device = RenderDevice::new(
            &instance.entry, 
            &instance.instance, 
            surface.surface, 
            &surface.surface_loader)?;
        
        let swapchain = RenderSwapchain::new(
            window, 
            &instance.instance, 
            &device.device,
            surface.surface,
            device.queue_family_indices,
            &device.swapchain_support,
        )?;
        
        Ok(Self { 
            instance, 
            surface,
            device,
            swapchain,
        })
    }
}