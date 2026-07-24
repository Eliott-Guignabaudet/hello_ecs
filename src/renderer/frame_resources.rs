use std::error::Error;
use std::sync::Arc;
use ash::{vk, Device, Instance};
use crate::renderer::buffer::create_buffer;
use crate::renderer::command_pool::CommandPool;
use crate::renderer::descriptor::{create_descriptor_set};
use crate::renderer::uniform_buffer::{UniformBuffer, UniformBufferObject};

pub struct RenderFrameResource {
    pub framebuffer: vk::Framebuffer,
    pub image_in_flight: vk::Fence,
    pub render_finished_semaphore: vk::Semaphore,
    pub graphics_command_pool: CommandPool,
    pub uniform_buffer: UniformBuffer,
    pub descriptor_set: vk::DescriptorSet,
    
    instances_data_buffer: vk::Buffer,
    instances_data_buffer_memory: vk::DeviceMemory,
    instances_data_buffer_size: vk::DeviceSize,
    
    device: Arc<Device>
}

impl RenderFrameResource {
    pub fn new(
        instance: &Instance,
        device: Arc<Device>,
        physical_device: vk::PhysicalDevice,
        swapchain_image_view: vk::ImageView,
        depth_image_view: vk::ImageView,
        color_image_view: vk::ImageView,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
        descriptor_set_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
    ) -> Result<Self, Box<dyn Error>> {

        let framebuffer = Self::create_framebuffer(
            &device,
            swapchain_image_view,
            depth_image_view,
            color_image_view,
            swapchain_extent,
            render_pass
        )?;
        
        let graphics_command_pool = CommandPool::new(
            Arc::clone(&device),
            queue_family_index,
            flags,
        )?;
        let image_in_flight = vk::Fence::null();
        let uniform_buffer_size = size_of::<UniformBufferObject>() as u64;
        let uniform_buffer = UniformBuffer::new(instance, &device, physical_device, uniform_buffer_size)?;
        let descriptor_set = create_descriptor_set(
            &device,
            descriptor_set_layout,
            descriptor_pool,
            uniform_buffer.buffer,
            size_of::<UniformBufferObject>() as u64,
        )?;

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let render_finished_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }?;


        Ok(Self {
            framebuffer,
            graphics_command_pool,
            image_in_flight,
            descriptor_set,
            uniform_buffer,
            render_finished_semaphore,
            device,

            instances_data_buffer: vk::Buffer::null(),
            instances_data_buffer_memory: vk::DeviceMemory::null(),
            instances_data_buffer_size: 0,
        })
    }
    
    fn create_framebuffer(
        device: &Device,
        swapchain_image_view: vk::ImageView,
        depth_image_view: vk::ImageView,
        color_image_view: vk::ImageView,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Result<vk::Framebuffer, Box<dyn Error>> {
        let attachments = &[color_image_view,  depth_image_view, swapchain_image_view];
        let create_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1);
        let framebuffer = unsafe { device.create_framebuffer(&create_info, None) }?;

        Ok(framebuffer)

    }

    pub fn reset(
        &mut self,
        swapchain_image_view: vk::ImageView,
        depth_image_view: vk::ImageView,
        color_image_view: vk::ImageView,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Result<(), Box<dyn Error>> {
        unsafe { self.device.destroy_framebuffer(self.framebuffer, None) }
        
        self.framebuffer = Self::create_framebuffer(
            &self.device,
            swapchain_image_view,
            depth_image_view,
            color_image_view,
            swapchain_extent,
            render_pass
        )?;

        Ok(())
    }
    
    pub fn get_or_create_instances_data_buffer(
        &mut self,
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        size: vk::DeviceSize,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), Box<dyn Error>>{
        if self.instances_data_buffer_size + 1 < size { 
            if self.instances_data_buffer_size > 0 {
                unsafe { self.device.destroy_buffer(self.instances_data_buffer, None); }
                unsafe { self.device.free_memory(self.instances_data_buffer_memory, None); }
            }
            
            let (buffer, buffer_memory) = create_buffer(
                instance,
                &self.device,
                physical_device,
                size + 1,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE
            )?;
            self.instances_data_buffer = buffer;
            self.instances_data_buffer_memory = buffer_memory;
            self.instances_data_buffer_size = size + 1;
        }
        
        
        Ok((self.instances_data_buffer, self.instances_data_buffer_memory))
    }
}

impl Drop for RenderFrameResource{
    fn drop(&mut self) {
        if self.instances_data_buffer_size > 0 {
            unsafe { self.device.destroy_buffer(self.instances_data_buffer, None); }
            unsafe { self.device.free_memory(self.instances_data_buffer_memory, None); }
        }
        unsafe { self.device.destroy_semaphore(self.render_finished_semaphore, None) }
        self.graphics_command_pool.destroy();
        unsafe { self.device.destroy_buffer(self.uniform_buffer.buffer, None) }
        unsafe { self.device.free_memory(self.uniform_buffer.buffer_memory, None) }
        unsafe { self.device.destroy_framebuffer(self.framebuffer, None) }
    }
}