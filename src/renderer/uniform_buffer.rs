use std::error::Error;
use ash::{vk, Device, Instance};
use nalgebra::Matrix4;
use crate::renderer::buffer::create_buffer;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

pub struct UniformBuffer {
    pub buffer: vk::Buffer,
    pub buffer_memory: vk::DeviceMemory,
    pub size: u64,
}

impl UniformBuffer {
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        size: u64,
    ) -> Result<Self, Box<dyn Error>> {

        let (buffer, buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        
        Ok(Self{buffer, buffer_memory, size})
    }
}
