mod constants;
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
mod material;
mod scene;

use std::error::Error;
use std::fmt;
use std::ptr::copy_nonoverlapping;
use std::sync::Arc;
use std::time::Instant;
use ash::vk;
use ash::vk::Handle;
use itertools::multizip;
use nalgebra::{Matrix4, Point3, Vector2, Vector3};
use winit::window::Window;


use instance::RenderInstance;
use surface::RenderSurface;
use device::RenderDevice;
use swapchain::RenderSwapchain;
use render_pass::RenderPass;
use graphic_pipeline::GraphicsPipeline;
use model::Model;
use vertex::Vertex;
use crate::renderer::descriptor::{create_descriptor_set, update_descriptor_image};
use crate::renderer::frame_resources::RenderFrameResource;
use crate::renderer::sync::FrameSync;
use crate::renderer::texture::Texture;
use crate::renderer::uniform_buffer::{UniformBuffer, UniformBufferObject};

pub use crate::renderer::material::{Material};
pub use crate::renderer::scene::Scene;

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
    //resources descriptors
    materials_descriptor_pool: vk::DescriptorPool,
    materials_descriptor_sets: Vec<vk::DescriptorSet>,
    materials_uniform_buffers: Vec<UniformBuffer>,
    
    // resources
    materials: Vec<Material>,
    models: Vec<Model>,
    textures: Vec<Texture>,
    

    frame_syncs: Vec<FrameSync>,
    frame_resources: Vec<RenderFrameResource>,
    graphics_pipeline: GraphicsPipeline,
    render_pass: RenderPass,
    swapchain: RenderSwapchain,
    device: RenderDevice,
    surface: RenderSurface,
    instance: RenderInstance,

    frame: usize,
    resized: bool,
    start: Instant,

}

impl HelloRenderer {
    pub fn new(window: &winit::window::Window) -> Result<Self, Box<dyn Error>> {
        let instance = RenderInstance::new(window)?;
        let surface = RenderSurface::new(&instance.entry, &instance.instance, window)?;
        let device = RenderDevice::new(
            &instance.instance, 
            surface.surface, 
            &surface.surface_loader)?;
        
        let swapchain = RenderSwapchain::new(
            window, 
            &instance.instance, 
            Arc::clone(&device.device),
            surface.surface,
            device.queue_family_indices,
            &device.swapchain_support,
        )?;
        
        let render_pass = RenderPass::new(
            &instance.instance,
            Arc::clone(&device.device),
            device.physical_device,
            swapchain.format,
            swapchain.extent,
            device.msaa_samples,
        )?;
        
        let graphics_pipeline = GraphicsPipeline::new(
            Arc::clone(&device.device),
            swapchain.extent,
            render_pass.render_pass,
            Vertex::binding_description(),
            Vertex::attribute_descriptions(),
            device.msaa_samples,
            swapchain.images.len() as u32,
        )?;

        let frame_resources = swapchain.image_views
            .iter()
            .map(|i| {
                RenderFrameResource::new(
                    &instance.instance,
                    Arc::clone(&device.device),
                    device.physical_device,
                    *i,
                    render_pass.depth.image_view.unwrap(),
                    render_pass.color_image.image_view.unwrap(),
                    swapchain.extent,
                    render_pass.render_pass,
                    device.queue_family_indices.graphics,
                    vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                    graphics_pipeline.descriptor_set_layout,
                    graphics_pipeline.descriptor_pool,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut frame_syncs = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            frame_syncs.push(FrameSync::new(Arc::clone(&device.device))?);
        }
        
        let models = vec![];
        let textures = vec![];
        let materials = vec![];

        let materials_descriptor_pool = vk::DescriptorPool::null();
        let materials_uniform_buffers = vec![];
        let materials_descriptor_sets = vec![];
        
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
            textures,
            materials,
            materials_descriptor_pool,
            materials_uniform_buffers,
            materials_descriptor_sets,
            
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }
    
    pub fn load_material_resources(&mut self, materials: Vec<Material>, texture_paths: Vec<&str>) -> Result<(), Box<dyn Error>>{
        self.materials = materials;
        texture_paths.iter().for_each(|p| {
            let texture = Texture::new(
                &self.instance.instance,
                Arc::clone(&self.device.device),
                self.device.physical_device,
                p,
                self.device.transfer_queue,
                self.device.transfer_command_pool.command_pool,
                self.device.graphics_queue,
                self.device.graphics_command_pool.command_pool,
            ).unwrap();
            self.textures.push(texture);
        });
        let descriptor_count = self.materials.len() as u32;
        let ubo_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(descriptor_count);
        let sampler_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(descriptor_count);


        let pool_sizes = &[ubo_size, sampler_size];
        let info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(pool_sizes)
            .max_sets(descriptor_count);
        self.materials_descriptor_pool = unsafe { self.device.device.create_descriptor_pool(&info, None) }?;
        
        
        for i in 0..descriptor_count {
            let uniform_buffer_size = size_of::<UniformBufferObject>() as u64;
            let uniform_buffer = UniformBuffer::new(
                &self.instance.instance,
                &self.device.device,
                self.device.physical_device,
                uniform_buffer_size,
            )?;
            let descriptor_set = create_descriptor_set(
                &self.device.device,
                self.graphics_pipeline.descriptor_set_layout_material,
                self.materials_descriptor_pool,
                uniform_buffer.buffer,
                uniform_buffer_size,
            )?;
            
            let texture = &self
                .textures[self.materials[i as usize].texture_index.unwrap() as usize];
            update_descriptor_image(
                descriptor_set,
                &self.device.device,
                texture.texture.image_view.unwrap(),
                texture.sampler,
            )?;
            
            self.materials_uniform_buffers.push(uniform_buffer);
            self.materials_descriptor_sets.push(descriptor_set);
            
        }
        
        
        Ok(())
    }
    
    pub fn load_model_from_path(&mut self, path: &str, correction: Matrix4<f32>) -> Result<(), Box<dyn Error>>{
        let new_model = Model::new_from_path(
            &self.instance.instance,
            Arc::clone(&self.device.device),
            self.device.physical_device, 
            self.device.transfer_queue,
            self.device.transfer_command_pool.command_pool,
            path,
            correction
        )?;
        
        self.models.push(new_model);
        
        Ok(())
    }

    pub fn load_model_from_raw_data(&mut self,  vertices: Vec<Vertex>, indices: Vec<u32>) -> Result<(), Box<dyn Error>>{
        let new_model = Model::new_from_raw_data(
            &self.instance.instance,
            Arc::clone(&self.device.device),
            self.device.physical_device,
            self.device.transfer_queue,
            self.device.transfer_command_pool.command_pool,
            vertices,
            indices,
        )?;

        self.models.push(new_model);

        Ok(())
    }
    
    
    pub fn render(&mut self,  window: &winit::window::Window, scene: Scene) -> Result<(), Box<dyn Error>>{
        let _ = &self.frame_syncs[self.frame].wait_for_fence()?;

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
        self.update_command_buffer(image_index, scene)?;

        let wait_semaphores = &[self.frame_syncs[self.frame].image_available_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.frame_resources[image_index].graphics_command_pool.command_buffer];
        let signal_semaphores = &[self.frame_syncs[self.frame].render_finished_semaphore];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        let _ = &self.frame_syncs[self.frame].reset_fence()?;

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



        let ubo = UniformBufferObject { view, proj };

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
    
    fn update_command_buffer(&mut self, image_index: usize, scene: Scene) -> Result<(), Box<dyn Error>> {
        self.frame_resources[image_index].graphics_command_pool.reset(&self.device.device)?;
        record_draw_command_for_scene(
            &self.device,
            &self.swapchain,
            &self.render_pass,
            &self.graphics_pipeline,
            &self.frame_resources[image_index],
            &scene,
            &self.models,
            &self.materials_descriptor_sets,
        )?;
        
        Ok(())
    }

    fn recreate_swapchain(&self, _: &Window) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

impl Drop for HelloRenderer {
    fn drop(&mut self) {
        unsafe { self.device.device.device_wait_idle().unwrap() }

        unsafe { self.device.device.destroy_descriptor_pool(self.materials_descriptor_pool, None) }
        self.materials_uniform_buffers.iter().for_each(|u| {
            unsafe { self.device.device.destroy_buffer(u.buffer, None) }
            unsafe { self.device.device.free_memory(u.buffer_memory, None) }
        });
        
        self.textures.iter_mut().for_each(|t| {
            unsafe { self.device.device.destroy_sampler(t.sampler, None) }
            t.texture.destroy();
        })
        
    }
}

fn record_draw_command(
    device: &RenderDevice, 
    swapchain: &RenderSwapchain,
    render_pass: &RenderPass,
    render_pipeline: &GraphicsPipeline,
    frame_resources: &RenderFrameResource,
    model: &Model,
    transform: Matrix4<f32>,
    material: vk::DescriptorSet,
) -> Result<(), Box<dyn Error>> {
    let info = vk::CommandBufferBeginInfo::default();
    let command_buffer = frame_resources.graphics_command_pool.command_buffer;
    unsafe { device.device.begin_command_buffer(command_buffer, &info)?; }

    let render_area = vk::Rect2D::default()
        .offset(vk::Offset2D::default())
        .extent(swapchain.extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.2, 0.2, 0.2, 1.0],
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
    unsafe {
        device.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            render_pipeline.pipeline_layout,
            0,
            &[frame_resources.descriptor_set, ],
            &[],
        );
    }

    unsafe { device.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, render_pipeline.pipeline); }
    let model_bytes = unsafe {
        std::slice::from_raw_parts(
            &transform as *const Matrix4<f32> as *const u8,
            size_of::<Matrix4<f32>>()
        )
    };
    unsafe { device.device.cmd_push_constants(command_buffer, render_pipeline.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, model_bytes); }

    unsafe {
        device.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            render_pipeline.pipeline_layout,
            1,
            &[material],
            &[],
        );
    }
    unsafe { device.device.cmd_bind_vertex_buffers(command_buffer, 0, &[model.vertex_buffer], &[0]); }
    unsafe { device.device.cmd_bind_index_buffer(command_buffer, model.index_buffer, 0, vk::IndexType::UINT32); }
    
    unsafe { device.device.cmd_draw_indexed(command_buffer, model.index_count, 1, 0, 0, 0); }
    unsafe { device.device.cmd_end_render_pass(command_buffer); }

    unsafe { device.device.end_command_buffer(command_buffer)?; }
    
    
    Ok(())
}

fn record_draw_command_for_scene(
    device: &RenderDevice,
    swapchain: &RenderSwapchain,
    render_pass: &RenderPass,
    render_pipeline: &GraphicsPipeline,
    frame_resources: &RenderFrameResource,
    scene: &Scene,
    models: &Vec<Model>,
    material_descriptors: &Vec<vk::DescriptorSet>,
    
) -> Result<(), Box<dyn Error>> {
    let mut datas : Vec<Vec<(Matrix4<f32>, usize)>> = vec![];
    datas.resize(material_descriptors.len(), vec![]);
    let zip = multizip((scene.transforms.iter(), scene.model_idxs.iter(), scene.material_idxs.iter()));
    zip.for_each(|(t, mo, ma)| {
       datas[*ma as usize].push((*t, *mo as usize))
    });
    
    let info = vk::CommandBufferBeginInfo::default();
    let command_buffer = frame_resources.graphics_command_pool.command_buffer;
    unsafe { device.device.begin_command_buffer(command_buffer, &info)?; }

    let render_area = vk::Rect2D::default()
        .offset(vk::Offset2D::default())
        .extent(swapchain.extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.2, 0.2, 0.2, 1.0],
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
    unsafe {
        device.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            render_pipeline.pipeline_layout,
            0,
            &[frame_resources.descriptor_set, ],
            &[],
        );
    }

    unsafe { device.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, render_pipeline.pipeline); }
    
    for (i, data) in datas.iter().enumerate() {
        unsafe {
            device.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                render_pipeline.pipeline_layout,
                1,
                &[material_descriptors[i]],
                &[],
            );
        }

        unsafe {
            data.iter().for_each(|(t, m)| {
                let model_bytes = std::slice::from_raw_parts(
                    t as *const Matrix4<f32> as *const u8,
                    size_of::<Matrix4<f32>>()
                );
                device.device.cmd_push_constants(command_buffer, render_pipeline.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, model_bytes);

                device.device.cmd_bind_vertex_buffers(command_buffer, 0, &[models[*m].vertex_buffer], &[0]); 
                 device.device.cmd_bind_index_buffer(command_buffer, models[*m].index_buffer, 0, vk::IndexType::UINT32); 

                device.device.cmd_draw_indexed(command_buffer, models[*m].index_count, 1, 0, 0, 0); 
            })
        }
    }
    
    unsafe { device.device.cmd_end_render_pass(command_buffer); }

    unsafe { device.device.end_command_buffer(command_buffer)?; }


    Ok(())
}