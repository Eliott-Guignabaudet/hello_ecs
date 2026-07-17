use nalgebra::Vector4;
use crate::renderer::texture::Texture;

pub struct Material{
    pub base_color : Vector4<f32>,
    
    pub texture_index: Option<u32>
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MaterialUniformBufferObject {
    pub base_color : Vector4<f32>,
}

