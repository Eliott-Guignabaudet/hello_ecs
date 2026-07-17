use std::error::Error;
use ash::{vk, Device};

pub fn create_descriptor_set(
    device: &Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    uniform_buffer: vk::Buffer,
    uniform_buffer_size: u64,
) -> Result<(vk::DescriptorSet), Box<dyn Error>>{
    let layouts = vec![descriptor_set_layout; 1];
    let info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_set = unsafe { device.allocate_descriptor_sets(&info) }?[0];
    
    let info = vk::DescriptorBufferInfo::default()
        .buffer(uniform_buffer)
        .offset(0)
        .range(uniform_buffer_size);

    let buffer_info = &[info];
    let ubo_write = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(buffer_info);
    unsafe {
        device.update_descriptor_sets(
            &[ubo_write],
            &[] as &[vk::CopyDescriptorSet]
        ); }

    Ok(descriptor_set)
}

pub fn update_descriptor_image(
    descriptor_set: vk::DescriptorSet,
    device: &Device,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
) -> Result<(), Box<dyn Error>> {
    let info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(texture_image_view)
        .sampler(texture_sampler);

    let image_info = &[info];
    let sampler_write = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_info);
    unsafe {
        device.update_descriptor_sets(
            &[sampler_write],
            &[] as &[vk::CopyDescriptorSet]
        ); }
    
    Ok(())
}