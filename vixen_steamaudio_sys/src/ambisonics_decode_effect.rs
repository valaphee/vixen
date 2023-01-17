use crate::error::check;
use crate::ffi;
use crate::geometry::Orientation;
use crate::prelude::*;
use glam::Vec3;

pub struct AmbisonicsDecodeEffect<'a> {
    pub(crate) inner: ffi::IPLAmbisonicsDecodeEffect,

    hrtf: &'a Hrtf,
}

impl<'a> AmbisonicsDecodeEffect<'a> {
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &'a Hrtf,
        maximum_order: u8,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLAmbisonicsDecodeEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsDecodeEffectCreate(
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

    pub fn apply(&self, orientation: Orientation, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsDecodeEffectParams {
            hrtf: self.hrtf.inner,
            orientation: orientation.into(),
            order: order as i32,
            binaural: 0,
        };

        unsafe {
            ffi::iplAmbisonicsDecodeEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }
}

unsafe impl<'a> Sync for AmbisonicsDecodeEffect<'a> {}
unsafe impl<'a> Send for AmbisonicsDecodeEffect<'a> {}

impl<'a> Clone for AmbisonicsDecodeEffect<'a> {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsDecodeEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            hrtf: self.hrtf,
        }
    }
}

impl<'a> Drop for AmbisonicsDecodeEffect<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsDecodeEffectRelease(&mut self.inner);
        }
    }
}
