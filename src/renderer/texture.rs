use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::ptr::copy_nonoverlapping;
use ash::{vk, Device, Instance};
use crate::renderer::buffer::{copy_buffer_to_image, create_buffer};
use crate::renderer::command::{begin_single_time_commands, end_single_time_commands};
use crate::renderer::image::ImageHandle;
use crate::renderer::VulkanError;

pub struct Texture{
    pub texture: ImageHandle,
    pub sampler: vk::Sampler,
    pub mip_levels: u32,
}

impl Texture {
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        resource_path: &dyn AsRef<Path>,
        transfer_queue: vk::Queue,
        transfer_command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        graphics_command_pool: vk::CommandPool,

    ) -> Result<Self, Box<dyn Error>>{
        let (mut texture, mip_levels) = Self::create_texture_image(
            instance,
            device,
            physical_device,
            resource_path,
            transfer_queue,
            transfer_command_pool,
            graphics_queue,
            graphics_command_pool,
        )?;
        texture.create_image_view(
            device,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            mip_levels
        )?;

        let info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(mip_levels as f32)
            .mip_lod_bias(0.0);

        let sampler = unsafe { device.create_sampler(&info, None) }?;



        Ok(Self { texture, sampler, mip_levels})
    }

    fn create_texture_image(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        resource_path: &dyn AsRef<Path>,
        transfer_queue: vk::Queue,
        transfer_command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        graphics_command_pool: vk::CommandPool,
    ) -> Result<(ImageHandle, u32), Box<dyn Error>> {

        let image = File::open(resource_path)?;

        let decoder = png::Decoder::new(image);
        let mut reader = decoder.read_info()?;

        let mut pixels = vec![0;  reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;

        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();

        if width != 1024 || height != 1024 || reader.info().color_type != png::ColorType::Rgba {
            panic!("Invalid texture image.");
        }

        let mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory = unsafe {
            device.map_memory(
                staging_buffer_memory,
                0,
                size,
                vk::MemoryMapFlags::empty(),
            )
        }?;

        unsafe { copy_nonoverlapping(pixels.as_ptr(), memory.cast(), pixels.len()); }

        unsafe { device.unmap_memory(staging_buffer_memory); }


        let image_handle = ImageHandle::new(
            instance,
            device,
            physical_device,
            width,
            height,
            mip_levels,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        
        transition_image_layout(
            device,
            image_handle.image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            mip_levels,
            transfer_queue,
            transfer_command_pool,
        )?;

        copy_buffer_to_image(
            device,
            staging_buffer,
            image_handle.image,
            width,
            height,
            transfer_queue,
            transfer_command_pool,
        )?;


        unsafe { device.destroy_buffer(staging_buffer, None); }
        unsafe { device.free_memory(staging_buffer_memory, None); }

        Self::generate_mipmaps(
            instance,
            device,
            physical_device,
            image_handle.image,
            vk::Format::R8G8B8A8_SRGB,
            width,
            height,
            mip_levels,
            graphics_queue,
            graphics_command_pool,
        )?;


        Ok((image_handle, mip_levels))
    }

    fn generate_mipmaps(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        image: vk::Image,
        format: vk::Format,
        width: u32,
        height: u32,
        mip_levels: u32,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
    ) -> Result<(), Box<dyn Error>> {
        unsafe {
            if !instance
                .get_physical_device_format_properties(physical_device, format)
                .optimal_tiling_features
                .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
            {
                return Err(Box::new(VulkanError("Texture image format does not support linear blitting!".to_string())));
            }
        }

        let command_buffer = begin_single_time_commands(device, command_pool)?;

        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);

        let mut barrier = vk::ImageMemoryBarrier::default()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(subresource);

        let mut mip_width = width;
        let mut mip_height = height;

        for i in 1..mip_levels {
            barrier.subresource_range.base_mip_level = i - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[] as &[vk::MemoryBarrier],
                    &[] as &[vk::BufferMemoryBarrier],
                    &[barrier],
                );
            }

            let src_subresource = vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i - 1)
                .base_array_layer(0)
                .layer_count(1);

            let dst_subresource = vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i)
                .base_array_layer(0)
                .layer_count(1);

            let blit = vk::ImageBlit::default()
                .src_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width as i32,
                        y: mip_height as i32,
                        z: 1,
                    },
                ])
                .src_subresource(src_subresource)
                .dst_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: (if mip_width > 1 { mip_width / 2 } else { 1 }) as i32,
                        y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
                        z: 1,
                    },
                ])
                .dst_subresource(dst_subresource);

            unsafe {
                device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit],
                    vk::Filter::LINEAR,
                );
            }

            barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[] as &[vk::MemoryBarrier],
                    &[] as &[vk::BufferMemoryBarrier],
                    &[barrier],
                );
            }

            if mip_width > 1 {
                mip_width /= 2;
            }

            if mip_height > 1 {
                mip_height /= 2;
            }
        }

        barrier.subresource_range.base_mip_level = mip_levels - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );
        }

        end_single_time_commands(device, queue, command_pool, command_buffer)?;

        Ok(())
    }
}


fn transition_image_layout(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> Result<(), Box<dyn Error>> {

    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        match format {
            vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT =>
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            _ => vk::ImageAspectFlags::DEPTH
        }
    } else {
        vk::ImageAspectFlags::COLOR
    };


    let (
        src_access_mask,
        dst_access_mask,
        src_stage_mask,
        dst_stage_mask
    ) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        _ => return Err(Box::new(VulkanError("Unsupported image layout transition!".to_string()))),
    };

    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let subresource = vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    }
    end_single_time_commands(device, queue, command_pool, command_buffer)?;

    Ok(())
}