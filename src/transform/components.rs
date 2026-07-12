use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Position(pub nalgebra::Point3<f32>);
pub struct Rotation(pub nalgebra::Quaternion<f32>);
pub struct Scale(pub nalgebra::Vector3<f32>);



impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //print!("cc zoe c'est la meilleure soeur du monde hihi ");
        write!(f, "x: {}, y: {}, z: {}", self.0.x, self.0.y, self.0.z)
    }
}

impl Display for Rotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //print!("cc zoe c'est la meilleure soeur du monde hihi ");
        write!(f, "i: {}, j: {}, k: {}, w: {}", self.0.i, self.0.j, self.0.k, self.0.w)
    }
}

impl Display for Scale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //print!("cc zoe c'est la meilleure soeur du monde hihi ");
        write!(f, "x: {}, y: {}, z: {}", self.0.x, self.0.y, self.0.z)
    }
}