use ash::vk;
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct InstanceData{
    pub matrix: Matrix4<f32>,
}

impl Vertex {
    pub const fn new(pos: Vector3<f32>, normal: Vector3<f32>, color: Vector3<f32>, tex_coord: Vector2<f32>) -> Self {
        Self { pos, normal, color, tex_coord }
    }

    pub fn binding_description() -> [vk::VertexInputBindingDescription; 2] {

        let vertex_binding = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);
        let instance_binding =         vk::VertexInputBindingDescription::default()
            .binding(1)
            .stride(size_of::<InstanceData>() as u32)
            .input_rate(vk::VertexInputRate::INSTANCE);
        [vertex_binding, instance_binding]
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 8] {
        // Per Vertex Data
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
        
        // Per Instance Data
        let row1 = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(4)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(0);
        let row2 = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(5)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(size_of::<Vector4<f32>>() as u32);
        let row3 = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(6)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset((size_of::<Vector4<f32>>() * 2) as u32);
        let row4 = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(7)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset((size_of::<Vector4<f32>>() * 3) as u32);
        
        [pos, normal_coord, color, tex_coord, row1, row2, row3, row4]
    }

}