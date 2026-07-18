use nalgebra::Matrix4;

pub struct Scene {
    pub transforms: Vec<Matrix4<f32>>,
    pub model_idxs: Vec<u32>,
    pub material_idxs: Vec<u32>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            transforms: vec![],
            model_idxs: vec![],
            material_idxs: vec![],
        }
    }
}