use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use glam::Vec3;

pub struct Buffer<'a> {
    pub(crate) inner: ffi::IPLAudioBuffer,

    context: &'a Context,
}

impl<'a> Buffer<'a> {
    pub fn new(context: &'a Context, channels: u16, samples: u32) -> Result<Self, Error> {
        let mut buffer = unsafe { std::mem::zeroed() };

        unsafe {
            check(
                ffi::iplAudioBufferAllocate(
                    context.inner,
                    channels as i32,
                    samples as i32,
                    &mut buffer,
                ),
                (),
            )?;
        }

        Ok(Self {
            inner: buffer,
            context,
        })
    }

    pub fn channels(&mut self) -> u16 {
        self.inner.numChannels as u16
    }

    pub fn samples(&mut self) -> u32 {
        self.inner.numSamples as u32
    }

    pub fn interleave(&mut self, out: &mut Vec<f32>) {
        unsafe { ffi::iplAudioBufferInterleave(self.context.inner, &self.inner, out.as_mut_ptr()) }
    }

    pub fn deinterleave(&mut self, in_: &[f32]) {
        unsafe {
            ffi::iplAudioBufferDeinterleave(self.context.inner, in_.as_ptr(), &mut self.inner);
        }
    }
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAudioBufferFree(self.context.inner, &mut self.inner);
        }
    }
}

pub enum SpeakerLayout {
    Mono,
    Stereo,
    Quadraphonic,
    Surround5_1,
    Surround7_1,
    Custom(Vec<Vec3>),
}

impl Into<ffi::IPLSpeakerLayout> for SpeakerLayout {
    fn into(self) -> ffi::IPLSpeakerLayout {
        match self {
            SpeakerLayout::Mono => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_MONO,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Stereo => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_STEREO,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Quadraphonic => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_QUADRAPHONIC,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Surround5_1 => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_SURROUND_5_1,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Surround7_1 => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_SURROUND_7_1,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Custom(speakers) => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_CUSTOM,
                numSpeakers: speakers.len() as i32,
                speakers: speakers
                    .iter()
                    .map(|speaker| speaker.into())
                    .collect::<Vec<_>>()
                    .as_ptr(),
            },
        }
    }
}
