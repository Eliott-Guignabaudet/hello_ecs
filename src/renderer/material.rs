use nalgebra::Vector4;
use crate::renderer::texture::Texture;

pub struct Material{
    pub base_color : Vector4<f32>,
    
    pub texture_index: Option<u32>
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            texture_index: None,
        }
    }
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

