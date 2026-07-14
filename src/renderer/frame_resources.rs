use std::error::Error;
use ash::{vk, Device, Instance};
use crate::renderer::command_pool::CommandPool;
use crate::renderer::descriptor::create_descriptor_set;
use crate::renderer::uniform_buffer::{UniformBuffer, UniformBufferObject};

pub struct RenderFrameResource {
    framebuffer: vk::Framebuffer,
    image_in_flight: vk::Fence,
    descriptor_set: vk::DescriptorSet,
    graphics_command_pool: CommandPool,
    uniform_buffer: UniformBuffer,
}

impl RenderFrameResource {
    pub fn new(
        instance: &Instance,
        device: &Device,
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
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> Result<Self, Box<dyn Error>> {

        let framebuffer = Self::create_framebuffer(
            device,
            swapchain_image_view,
            depth_image_view,
            color_image_view,
            swapchain_extent,
            render_pass
        )?;
        
        let graphics_command_pool = CommandPool::new(
            device,
            queue_family_index,
            flags,
        )?;
        let image_in_flight = vk::Fence::null();
        let uniform_buffer = UniformBuffer::new(instance, device, physical_device)?;
        let descriptor_set = create_descriptor_set(
            device,
            descriptor_set_layout,
            descriptor_pool,
            uniform_buffer.buffer,
            size_of::<UniformBufferObject>() as u64,
            texture_image_view,
            texture_sampler,
        )?;
        
        
        Ok(Self {
            framebuffer,
            graphics_command_pool,
            image_in_flight,
            descriptor_set,
            uniform_buffer,
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