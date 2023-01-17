use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct Buffer<'a> {
    pub(crate) inner: ffi::IPLAudioBuffer,

    context: &'a Context
}

impl<'a> Buffer<'a> {
    pub fn new(context: &'a Context, channels: u16, samples: u32) -> Result<Self, Error> {
        let mut buffer: ffi::IPLAudioBuffer = unsafe { std::mem::zeroed() };

        unsafe {
            check(ffi::iplAudioBufferAllocate(
                context.inner,
                channels as i32,
                samples as i32,
                &mut buffer,
            ), ())?;
        }

        Ok(Self { inner: buffer, context })
    }

    pub fn channels(&mut self) -> u16 {
        self.inner.numChannels as u16
    }

    pub fn samples(&mut self) -> u32 {
        self.inner.numSamples as u32
    }

    pub fn interleave(&mut self, out: &mut Vec<f32>) {
        unsafe {
            ffi::iplAudioBufferInterleave(self.context.inner, &self.inner, out.as_mut_ptr())
        }
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
