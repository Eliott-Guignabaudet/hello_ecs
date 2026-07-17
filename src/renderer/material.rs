use nalgebra::Vector4;
use crate::renderer::texture::Texture;

pub struct Material{
    pub base_color : Vector4<f32>,
    
    pub texture_index: Option<u32>
}

impl Material {
    pub fn to_uniform(&self) -> MaterialUniformBufferObject {
        
        MaterialUniformBufferObject {
            base_color: self.base_color,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MaterialUniformBufferObject {
    pub base_color : Vector4<f32>,
}

