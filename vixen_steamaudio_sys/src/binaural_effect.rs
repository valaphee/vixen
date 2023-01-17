use crate::ffi;
use crate::prelude::*;
use glam::Vec3;
use crate::error::check;

pub struct BinauralEffect<'a> {
    pub(crate) inner: ffi::IPLBinauralEffect,

    hrtf: &'a Hrtf,
}

impl<'a> BinauralEffect<'a> {
    pub fn new(context: &Context, sample_rate: u32, frame_length: u32, hrtf: &'a Hrtf) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLBinauralEffectSettings { hrtf: hrtf.inner };
        let mut effect: ffi::IPLBinauralEffect = unsafe { std::mem::zeroed() };

        unsafe {
            check(ffi::iplBinauralEffectCreate(
                context.inner,
                &audio_settings,
                &effect_settings,
                &mut effect,
            ), ())?;
        }

        Ok(Self { inner: effect, hrtf })
    }

    pub fn apply(&self, direction: Vec3, interpolation: HrtfInterpolation, spatial_bend: f32, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLBinauralEffectParams {
            direction: direction.into(),
            interpolation: interpolation.into(),
            spatialBlend: spatial_bend,
            hrtf: self.hrtf.inner,
            peakDelays: std::ptr::null_mut(),
        };

        unsafe {
            ffi::iplBinauralEffectApply(
                self.inner,
                &params,
                &in_.inner,
                &mut out.inner,
            );
        }
    }
}

unsafe impl<'a> Sync for BinauralEffect<'a> {}
unsafe impl<'a> Send for BinauralEffect<'a> {}

impl<'a> Clone for BinauralEffect<'a> {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplBinauralEffectRetain(self.inner);
        }

        BinauralEffect {
            inner: self.inner,
            hrtf: self.hrtf,
        }
    }
}

impl<'a> Drop for BinauralEffect<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::iplBinauralEffectRelease(&mut self.inner);
        }
    }
}
