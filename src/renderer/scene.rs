use nalgebra::{Matrix4, Quaternion, UnitQuaternion, Vector3};

pub struct Scene {
    pub camera_data: CameraData,
    pub transforms: Vec<Matrix4<f32>>,
    pub model_idxs: Vec<u32>,
    pub material_idxs: Vec<u32>,
}



impl Default for Scene {
    fn default() -> Self {
        Self {
            camera_data: CameraData::default(),
            transforms: vec![],
            model_idxs: vec![],
            material_idxs: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraData{
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

impl Default for CameraData {
    fn default() -> Self {
        CameraData{position: Vector3::zeros(), rotation: UnitQuaternion::identity() }
    }
}