use std::error::Error;
use ash::{vk, Device, Instance};
use crate::renderer::command::{begin_single_time_commands, end_single_time_commands};

pub fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {

    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE );

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            physical_device,
            properties,
            requirements,
        )?);

    let buffer_memory = unsafe { device.allocate_memory(&memory_info, None) }?;

    unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0)?; }

    Ok((buffer, buffer_memory))
}

pub fn copy_buffer(
    device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> anyhow::Result<()> {

    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let regions = vk::BufferCopy::default().size(size);
    unsafe { device.cmd_copy_buffer(command_buffer, source, destination, &[regions]); }

    end_single_time_commands(device, queue, command_pool, command_buffer)?;

    Ok(())
}

pub fn copy_buffer_to_image(
    device: &Device,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> anyhow::Result<()> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let subresource = vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::default()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width, height, depth: 1 });

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    end_single_time_commands(device, queue, command_pool, command_buffer)?;

    Ok(())
}


pub fn get_memory_type_index(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> anyhow::Result<u32> {
    let memory = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to find suitable memory type."))
}