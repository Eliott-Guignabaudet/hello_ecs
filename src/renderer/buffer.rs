use std::error::Error;
use ash::{vk, Device, Instance};



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