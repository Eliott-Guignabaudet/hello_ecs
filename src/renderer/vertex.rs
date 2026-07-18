use ash::vk;
use nalgebra::{Vector2, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

impl Vertex {
    pub const fn new(pos: Vector3<f32>, normal: Vector3<f32>, color: Vector3<f32>, tex_coord: Vector2<f32>) -> Self {
        Self { pos, normal, color, tex_coord }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {

        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        let pos = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0);
        let normal_coord = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vector3<f32>>() as u32);
        let color = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset((size_of::<Vector3<f32>>()  + size_of::<Vector3<f32>>() )as u32);
        let tex_coord = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(3)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vector3<f32>>() + size_of::<Vector3<f32>>() + size_of::<Vector3<f32>>()) as u32);
        [pos, normal_coord, color, tex_coord]
    }
}