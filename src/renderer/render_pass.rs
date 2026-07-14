use ash::vk;
use crate::renderer::image::ImageHandle;

pub struct RenderPass {
    render_pass: vk::RenderPass,
    depth: ImageHandle,
    color_image: ImageHandle,
}