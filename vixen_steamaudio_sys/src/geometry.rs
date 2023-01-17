use crate::ffi;
use glam::Vec3;

impl From<Vec3> for ffi::IPLVector3 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<&Vec3> for ffi::IPLVector3 {
    fn from(value: &Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

pub struct Orientation {
    pub right: Vec3,
    pub up: Vec3,
    pub ahead: Vec3,
    pub origin: Vec3,
}

impl From<Orientation> for ffi::IPLCoordinateSpace3 {
    fn from(value: Orientation) -> Self {
        Self {
            right: value.right.into(),
            up: value.up.into(),
            ahead: value.ahead.into(),
            origin: value.origin.into(),
        }
    }
}
