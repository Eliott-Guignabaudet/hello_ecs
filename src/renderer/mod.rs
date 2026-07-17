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
mod model;
mod command;
mod sync;
mod texture;

use std::error::Error;
use std::fmt;
use std::path::Path;
use std::ptr::copy_nonoverlapping;
use std::time::Instant;
use ash::vk;
use ash::vk::Handle;
use nalgebra::{Matrix4, Point3, Unit, UnitVector3, Vector2, Vector3};
use thiserror::__private18::AsDynError;
use winit::window::Window;
pub use app::RenderApp;


use instance::RenderInstance;
use surface::RenderSurface;
use device::RenderDevice;
use swapchain::RenderSwapchain;
use render_pass::RenderPass;
use graphic_pipeline::GraphicsPipeline;
use model::Model;
use vertex::Vertex;
use crate::renderer::command_pool::CommandPool;
use crate::renderer::frame_resources::RenderFrameResource;
use crate::renderer::sync::FrameSync;
use crate::renderer::texture::Texture;
use crate::renderer::uniform_buffer::UniformBufferObject;

const MAX_FRAMES_IN_FLIGHT: usize = 2;
#[derive(Debug)]
struct VulkanError(String);

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vulkan Error: {}", self.0)
    }
}
impl Error for VulkanError {}

#[rustfmt::skip]
static VERTICES: [Vertex; 8] = [
    Vertex::new(Vector3::new(-0.5, -0.5, 0.0),Vector3::new(1.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(1.0, 0.0)),
    Vertex::new(Vector3::new(0.5, -0.5, 0.0), Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(0.0, 0.0)),
    Vertex::new(Vector3::new(0.5, 0.5, 0.0), Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(0.0, 1.0)),
    Vertex::new(Vector3::new(-0.5, 0.5, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(1.0, 1.0)),
    //
    Vertex::new(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(1.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(1.0, 0.0)),
    Vertex::new(Vector3::new(0.5, -0.5, -0.5), Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(0.0, 0.0)),
    Vertex::new(Vector3::new(0.5, 0.5, -0.5), Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(0.0, 1.0)),
    Vertex::new(Vector3::new(-0.5, 0.5, -0.5), Vector3::new(1.0, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0),Vector2::new(1.0, 1.0)),
];

#[rustfmt::skip]
const INDICES: &[u32] = &[
    0, 1, 2, 2, 3, 0,
    //
    4, 5, 6, 6, 7, 4
];


pub struct HelloRenderer {
    instance: RenderInstance,
    surface: RenderSurface,
    device: RenderDevice,
    swapchain: RenderSwapchain,
    render_pass: RenderPass,
    graphics_pipeline: GraphicsPipeline,
    frame_resources: Vec<RenderFrameResource>,
    frame_syncs: Vec<FrameSync>,
    // resources
    models: Vec<Model>,

    frame: usize,
    resized: bool,
    start: Instant,

}

impl HelloRenderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, Box<dyn Error>> {
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
        
        let texture = Texture::new(
            &instance.instance,
            &device.device,
            device.physical_device,
            &Path::new("resources/texture.png"),
            device.transfer_queue,
            device.transfer_command_pool.command_pool,
            device.graphics_queue,
            device.graphics_command_pool.command_pool,
        )?;

        let frame_resources = swapchain.image_views
            .iter()
            .map(|i| {
                RenderFrameResource::new(
                    &instance.instance,
                    &device.device,
                    device.physical_device,
                    *i,
                    render_pass.depth.image_view.unwrap(),
                    render_pass.color_image.image_view.unwrap(),
                    swapchain.extent,
                    render_pass.render_pass,
                    device.queue_family_indices.graphics,
                    vk::CommandPoolCreateFlags::TRANSIENT,
                    graphics_pipeline.descriptor_set_layout,
                    graphics_pipeline.descriptor_pool,
                    texture.texture.image_view.unwrap(),
                    texture.sampler,
                )
            })
            .collect::<anyhow::Result<Vec<_>, _>>()?;
        let mut frame_syncs = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            frame_syncs.push(FrameSync::new(&device.device)?);
        }
        
        let mut models = vec![];
        let model = Model::new_from_raw_data(
            &instance.instance,
            &device.device,
            device.physical_device,
            device.transfer_queue,
            device.transfer_command_pool.command_pool,
            VERTICES.to_vec(),
            INDICES.to_vec(),
        )?;
        models.push(model);
        
        frame_resources.iter().for_each(|f| {
            record_draw_command(
                &device,
                &swapchain,
                &render_pass,
                &graphics_pipeline,
                f,
                &models[0],
            ).unwrap()
        });
        
        Ok(Self { 
            instance, 
            surface,
            device,
            swapchain,
            render_pass,
            graphics_pipeline,
            frame_resources,
            frame_syncs,
            models,
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }
    
    pub fn load_model_from_path(&mut self, path: &str) -> Result<(), Box<dyn Error>>{
        let new_model = Model::new_from_path(
            &self.instance.instance,
            &self.device.device,
            self.device.physical_device, 
            self.device.transfer_queue,
            self.device.transfer_command_pool.command_pool,
            path,
        )?;
        
        self.models.push(new_model);
        
        Ok(())
    }

    pub fn load_model_from_raw_data(&mut self,  vertices: Vec<Vertex>, indices: Vec<u32>) -> Result<(), Box<dyn Error>>{
        let new_model = Model::new_from_raw_data(
            &self.instance.instance,
            &self.device.device,
            self.device.physical_device,
            self.device.transfer_queue,
            self.device.transfer_command_pool.command_pool,
            vertices,
            indices,
        )?;

        self.models.push(new_model);

        Ok(())
    }
    
    
    pub fn render(&mut self,  window: &winit::window::Window) -> Result<(), Box<dyn Error>>{
        let _ = &self.frame_syncs[self.frame].wait_for_fence(&self.device.device)?;

        let result = unsafe {
            self.swapchain.swapchain_loader
                .acquire_next_image(
                    self.swapchain.swapchain,
                    u64::MAX,
                    self.frame_syncs[self.frame].image_available_semaphore,
                    vk::Fence::null(),
                )
        };

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e ) =>  return Err(Box::new(VulkanError(format!("{}", e).into()))),
        };

        let image_in_flight = &self.frame_resources[image_index].image_in_flight;
        if !image_in_flight.is_null() {
            unsafe { self.device.device.wait_for_fences(&[*image_in_flight], true, u64::MAX)?; }
        }

        self.frame_resources[image_index].image_in_flight =  self.frame_syncs[self.frame].in_flight_fence;
       
        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.frame_syncs[self.frame].image_available_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.frame_resources[image_index].graphics_command_pool.command_buffer];
        let signal_semaphores = &[self.frame_syncs[self.frame].render_finished_semaphore];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        let _ = &self.frame_syncs[self.frame].reset_fence(&self.device.device)?;

        unsafe {
            self.device.device.queue_submit(
                self.device.graphics_queue, &[submit_info], self.frame_syncs[self.frame].in_flight_fence)?;
        }

        let swapchains = &[self.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = unsafe {
            self.swapchain.swapchain_loader.queue_present(self.device.present_queue, &present_info)
        };
        let changed = result == Ok(true) || result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(Box::new(VulkanError(format!("Fail to recreate swapchain{e}"))));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
    
    fn update_uniform_buffer(&mut self, image_index: usize) -> Result<(), Box<dyn Error>> {
        let time = self.start.elapsed().as_secs_f32();
        let rotation_angle : Unit<Vector3<f32>> = UnitVector3:: new_normalize(Vector3::new(0.0, 0.0, 1.0));
        
        let model = Matrix4::from_axis_angle(&rotation_angle,  90.0_f32.to_radians() * time);
        //let model = Matrix4::identity();
        let view = Matrix4::look_at_rh(
            &Point3::new(2.0, 2.0, 2.0),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 0.0, 1.0),
        );

        #[rustfmt::skip]
        let correction = Matrix4::new(
            1.0,  0.0,       0.0, 0.0,
            0.0, -1.0,       0.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 1.0,
        );
        
        let proj = correction * nalgebra::Perspective3::new(
            self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32,
            3.14 / 4.0,
            0.1,
            200.0,
        ).to_homogeneous();



        let ubo = UniformBufferObject { model, view, proj };

        // Copy

        let memory = unsafe {
            self.device.device.map_memory(
                self.frame_resources[image_index].uniform_buffer.buffer_memory,
                0,
                size_of::<UniformBufferObject>() as u64,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        unsafe { copy_nonoverlapping(&ubo, memory.cast(), 1); }

        unsafe { self.device.device.unmap_memory(self.frame_resources[image_index].uniform_buffer.buffer_memory); }
        Ok(())
    }

    fn recreate_swapchain(&self, window: &Window) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

fn record_draw_command(
    device: &RenderDevice, 
    swapchain: &RenderSwapchain,
    render_pass: &RenderPass,
    render_pipeline: &GraphicsPipeline,
    frame_resources: &RenderFrameResource,
    model: &Model
) -> Result<(), Box<dyn Error>> {
    let info = vk::CommandBufferBeginInfo::default();
    let command_buffer = frame_resources.graphics_command_pool.command_buffer;
    unsafe { device.device.begin_command_buffer(command_buffer, &info)?; }

    let render_area = vk::Rect2D::default()
        .offset(vk::Offset2D::default())
        .extent(swapchain.extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };

    let depth_clear_value = vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
    };

    let clear_values = &[color_clear_value, depth_clear_value];
    let info = vk::RenderPassBeginInfo::default()
        .render_pass(render_pass.render_pass)
        .framebuffer(frame_resources.framebuffer)
        .render_area(render_area)
        .clear_values(clear_values);

    unsafe { device.device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE); }
    unsafe { device.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, render_pipeline.pipeline); }
    unsafe { device.device.cmd_bind_vertex_buffers(command_buffer, 0, &[model.vertex_buffer], &[0]); }
    unsafe { device.device.cmd_bind_index_buffer(command_buffer, model.index_buffer, 0, vk::IndexType::UINT32); }
    unsafe {
        device.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            render_pipeline.pipeline_layout,
            0,
            &[frame_resources.descriptor_set],
            &[],
        );
    }
    unsafe { device.device.cmd_draw_indexed(command_buffer, INDICES.len() as u32, 1, 0, 0, 0); }
    unsafe { device.device.cmd_end_render_pass(command_buffer); }

    unsafe { device.device.end_command_buffer(command_buffer)?; }
    
    
    Ok(())
}