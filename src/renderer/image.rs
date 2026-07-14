use std::error::Error;
use ash::{vk, Device, Instance};
use crate::renderer::buffer::get_memory_type_index;

pub struct ImageHandle {
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: Option<vk::ImageView>,
}

impl ImageHandle {
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        width: u32,
        height: u32,
        mip_levels: u32,
        samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self, Box<dyn Error>> {
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(mip_levels)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(samples)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.create_image(&info, None) }?;

        let requirements = unsafe { device.get_image_memory_requirements(image) };

        let info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(get_memory_type_index(
                instance,
                physical_device,
                properties,
                requirements,
            )?);

        let image_memory = unsafe { device.allocate_memory(&info, None) }?;

        Ok(Self{
            image,
            image_memory,
            image_view: None,
        })
    }
    
    pub fn create_image_view(
        &mut self,
        device: &Device,
        format: vk::Format,
        aspects: vk::ImageAspectFlags,
        mip_levels: u32,
    ) ->  Result<(), Box<dyn Error>> {
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(aspects)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);

        let info = vk::ImageViewCreateInfo::default()
            .image(self.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(subresource_range);
        self.image_view = Some(unsafe {device.create_image_view(&info, None)? });
        Ok(())
    }
}