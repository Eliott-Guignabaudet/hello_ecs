use bitflags::bitflags;
use nalgebra::Vector3;

bitflags! {
    pub struct MovementFlags: u32 {
        const NONE = 0;
        const FORWARD = 1;
        const BACKWARD = 1 << 2;
        const LEFT = 1 << 3;
        const RIGHT = 1 << 4;
    }
}
pub fn get_camera_movement(movement_flags: MovementFlags) -> Vector3<f32> {
    let mut movement = Vector3::zeros();

    if movement_flags.contains(MovementFlags::FORWARD) {
        movement += Vector3::new(0.0, 0.0, 1.0 );
    }
    if movement_flags.contains(MovementFlags::BACKWARD) {
        movement += Vector3::new(0.0, 0.0, -1.0 );
    }
    if movement_flags.contains(MovementFlags::LEFT) {
        movement += Vector3::new(1.0, 0.0, 0.0 );
    }
    if movement_flags.contains(MovementFlags::RIGHT) {
        movement += Vector3::new(-1.0, 0.0, 0.0 );
    }

    movement
}