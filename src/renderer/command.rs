use std::error::Error;
use ash::{vk, Device};

pub fn begin_single_time_commands(
    device: &Device,
    command_pool: vk::CommandPool,
) -> Result<vk::CommandBuffer, Box<dyn Error>> {
    let info = vk::CommandBufferAllocateInfo::default()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info) }?[0];

    let info = vk::CommandBufferBeginInfo::default()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &info)?; }

    Ok(command_buffer)
}

pub fn end_single_time_commands(
    device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
) -> Result<(), Box<dyn Error>> {
    unsafe { device.end_command_buffer(command_buffer)?; }

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::default()
        .command_buffers(command_buffers);

    unsafe { device.queue_submit(queue, &[info], vk::Fence::null())?; }
    unsafe { device.queue_wait_idle(queue)?; }

    unsafe { device.free_command_buffers(command_pool, &[command_buffer]); }

    Ok(())
} 