pub mod binaural_effect;
pub mod buffer;
pub mod context;
pub mod direct_effect;
pub mod error;
pub mod hrtf;

pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod prelude {
    pub use crate::binaural_effect::BinauralEffect;
    pub use crate::buffer::Buffer;
    pub use crate::context::Context;
    pub use crate::direct_effect::DirectEffect;
    pub use crate::error::Error;
    pub use crate::hrtf::{Hrtf, HrtfInterpolation, HrtfType};
}

impl From<glam::Vec3> for ffi::IPLVector3 {
    fn from(value: glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<&glam::Vec3> for ffi::IPLVector3 {
    fn from(value: &glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
