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
mod graphic_pipeline;
mod vertex;

use std::error::Error;
pub use app::RenderApp;


use instance::RenderInstance;
use surface::RenderSurface;
use device::RenderDevice;
use swapchain::RenderSwapchain;
use render_pass::RenderPass;
use crate::renderer::graphic_pipeline::GraphicsPipeline;

pub struct HelloRenderer {
    instance: RenderInstance,
    surface: RenderSurface,
    device: RenderDevice,
    swapchain: RenderSwapchain,
    render_pass: RenderPass,
    graphics_pipeline: GraphicsPipeline,
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
        
        let render_pass = RenderPass::new(
            &instance.instance,
            &device.device,
            device.physical_device,
            swapchain.format,
            swapchain.extent,
            device.msaa_samples,
        )?;
        
        let graphics_pipeline = GraphicsPipeline::new(
            &device.device,
            swapchain.extent,
            render_pass.render_pass,
            vertex::Vertex::binding_description(),
            vertex::Vertex::attribute_descriptions(),
            device.msaa_samples,
            swapchain.images.len() as u32,
        )?;
        
        Ok(Self { 
            instance, 
            surface,
            device,
            swapchain,
            render_pass,
            graphics_pipeline,
        })
    }
}