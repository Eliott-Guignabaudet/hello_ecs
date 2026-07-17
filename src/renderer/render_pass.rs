use std::error::Error;
use ash::{vk, Device, Instance};
use crate::renderer::image::ImageHandle;

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
    pub color_image: ImageHandle,
    pub depth: ImageHandle,
}

impl RenderPass {
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        swapchain_format: vk::Format,
        swapchain_extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<Self, Box<dyn Error>> {
        let depth_format = Self::get_depth_format( instance, physical_device)?;
        
        let render_pass = Self::create_render_pass(
            device,
            swapchain_format,
            depth_format,
            msaa_samples,
        )?;

        let mut color_image = ImageHandle::new(
            instance, 
            device, 
            physical_device,
            swapchain_extent.width,
            swapchain_extent.height,
            1,
            msaa_samples, 
            swapchain_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?; 
        
        color_image.create_image_view(
            device,
            swapchain_format,
            vk::ImageAspectFlags::COLOR,
            1,
        )?;
        
        let mut depth = ImageHandle::new(
            instance,
            device,
            physical_device,
            swapchain_extent.width,
            swapchain_extent.height,
            1,
            msaa_samples,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        
        depth.create_image_view(
            device,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
            1)?;

        Ok(Self {
            render_pass,
            color_image,
            depth
        })
    }
    
    fn create_render_pass(
        device: &Device,
        swapchain_format: vk::Format,
        depth_format: vk::Format,
        msaa_sample: vk::SampleCountFlags,
    ) -> Result<vk::RenderPass, Box<dyn Error>> {

        // Attachments

        let color_attachment = vk::AttachmentDescription::default()
            .format(swapchain_format)
            .samples(msaa_sample)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment = vk::AttachmentDescription::default()
            .format(depth_format)
            .samples(msaa_sample)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_resolve_attachment = vk::AttachmentDescription::default()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        // Subpasses

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_resolve_attachment_ref = vk::AttachmentReference::default()
            .attachment(2)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];
        let resolve_attachments = &[color_resolve_attachment_ref];
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            .depth_stencil_attachment(&depth_stencil_attachment_ref)
            .resolve_attachments(resolve_attachments);


        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

        // Create

        let attachments = &[
            color_attachment,
            depth_stencil_attachment,
            color_resolve_attachment,
        ];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::default()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);

        let render_pass = unsafe { device.create_render_pass(&info, None) }?;
        
        Ok(render_pass)
    }

    fn get_depth_format(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    )-> anyhow::Result<vk::Format> {
        let candidates = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        Self::get_supported_format(
            instance,
            physical_device,
            candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn get_supported_format(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> anyhow::Result<vk::Format> {

        unsafe {
            candidates
                .iter()
                .cloned()
                .find(|f| {
                    let properties = instance.get_physical_device_format_properties(
                        physical_device,
                        *f,
                    );
                    match tiling {
                        vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                        vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                        _ => false,
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("Failed to find supported format!"))

        }
    }
}