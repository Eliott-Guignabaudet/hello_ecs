use std::error::Error;
use std::sync::Arc;
use ash::{vk, Device, Instance};
use crate::renderer::command_pool::CommandPool;
use crate::renderer::descriptor::{create_descriptor_set};
use crate::renderer::uniform_buffer::{UniformBuffer, UniformBufferObject};

pub struct RenderFrameResource {
    pub framebuffer: vk::Framebuffer,
    pub image_in_flight: vk::Fence,
    pub graphics_command_pool: CommandPool,
    pub uniform_buffer: UniformBuffer,
    pub descriptor_set: vk::DescriptorSet,
    
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
        
        
        Ok(Self {
            framebuffer,
            graphics_command_pool,
            image_in_flight,
            descriptor_set,
            uniform_buffer,
            device,
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
    
}

impl Drop for RenderFrameResource{
    fn drop(&mut self) {
        self.graphics_command_pool.destroy();
        unsafe { self.device.destroy_buffer(self.uniform_buffer.buffer, None) }
        unsafe { self.device.free_memory(self.uniform_buffer.buffer_memory, None) }
        unsafe { self.device.destroy_framebuffer(self.framebuffer, None) }
    }
}