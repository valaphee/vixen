use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct Context {
    pub(crate) inner: ffi::IPLContext,
}

impl Default for Context {
    fn default() -> Self {
        let context_settings = ffi::IPLContextSettings {
            version: ffi::STEAMAUDIO_VERSION_MAJOR << 16
                | ffi::STEAMAUDIO_VERSION_MINOR << 8
                | ffi::STEAMAUDIO_VERSION_PATCH,
            logCallback: None,
            allocateCallback: None,
            freeCallback: None,
            simdLevel: ffi::IPLSIMDLevel_IPL_SIMDLEVEL_AVX2,
        };
        let mut context: ffi::IPLContext = unsafe { std::mem::zeroed() };

        unsafe {
            check(ffi::iplContextCreate(&context_settings, &mut context), ()).unwrap();
        }

        Self { inner: context }
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Clone for Context {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplContextRetain(self.inner);
        }

        Context {
            inner: self.inner,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::iplContextRelease(&mut self.inner);
        }
    }
}
