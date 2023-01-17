use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use std::ffi::CString;

pub struct Hrtf {
    pub(crate) inner: ffi::IPLHRTF,
}

impl Hrtf {
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        hrtf_type: HrtfType,
    ) -> Result<Self, Error> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let mut hrtf = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplHRTFCreate(
                    context.inner,
                    &mut audio_settings,
                    &mut hrtf_type.into(),
                    &mut hrtf,
                ),
                (),
            )?;
        }

        Ok(Self { inner: hrtf })
    }
}

unsafe impl Sync for Hrtf {}
unsafe impl Send for Hrtf {}

impl Clone for Hrtf {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplHRTFRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for Hrtf {
    fn drop(&mut self) {
        unsafe {
            ffi::iplHRTFRelease(&mut self.inner);
        }
    }
}

pub enum HrtfType {
    Default,
    Sofa(String),
}

impl Default for HrtfType {
    fn default() -> Self {
        Self::Default
    }
}

impl Into<ffi::IPLHRTFSettings> for HrtfType {
    fn into(self) -> ffi::IPLHRTFSettings {
        match self {
            Self::Default => ffi::IPLHRTFSettings {
                type_: ffi::IPLHRTFType_IPL_HRTFTYPE_DEFAULT,
                sofaFileName: std::ptr::null_mut(),
            },
            Self::Sofa(path) => ffi::IPLHRTFSettings {
                type_: ffi::IPLHRTFType_IPL_HRTFTYPE_SOFA,
                sofaFileName: CString::new(path.clone()).unwrap().as_ptr(),
            },
        }
    }
}
