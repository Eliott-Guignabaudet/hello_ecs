use std::error::Error;
use ash::{vk, Device};

pub struct CommandPool {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
}

impl CommandPool {
    pub fn new(
        device: &Device,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Self, Box<dyn Error>> {
        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(flags);
        
        let command_pool = unsafe { device.create_command_pool(&create_info, None)?};
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info) }?[0];



        Ok(Self {
            command_pool,
            command_buffer,
        })
    }
    
    pub fn reset(&mut self, device: &Device) -> Result<(), Box<dyn Error>> {
        unsafe { device.reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())? }
        
        Ok(())
    }
}