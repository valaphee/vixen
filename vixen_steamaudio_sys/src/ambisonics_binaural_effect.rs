use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use glam::Vec3;

pub struct AmbisonicsBinauralEffect<'a> {
    pub(crate) inner: ffi::IPLAmbisonicsBinauralEffect,

    hrtf: &'a Hrtf,
}

impl<'a> AmbisonicsBinauralEffect<'a> {
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        hrtf: &'a Hrtf,
        maximum_order: u8,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLAmbisonicsBinauralEffectSettings {
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsBinauralEffectCreate(
                    context.inner,
                    &audio_settings,
                    &effect_settings,
                    &mut effect,
                ),
                (),
            )?;
        }

        Ok(Self {
            inner: effect,
            hrtf,
        })
    }

    pub fn apply(&self, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsBinauralEffectParams {
            hrtf: self.hrtf.inner,
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsBinauralEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }
}

unsafe impl<'a> Sync for AmbisonicsBinauralEffect<'a> {}
unsafe impl<'a> Send for AmbisonicsBinauralEffect<'a> {}

impl<'a> Clone for AmbisonicsBinauralEffect<'a> {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            hrtf: self.hrtf,
        }
    }
}

impl<'a> Drop for AmbisonicsBinauralEffect<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectRelease(&mut self.inner);
        }
    }
}
