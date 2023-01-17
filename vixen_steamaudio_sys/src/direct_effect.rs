use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct DirectEffect {
    pub(crate) inner: ffi::IPLDirectEffect,
}

impl DirectEffect {
    pub fn new(context: &Context, sample_rate: u32, frame_length: u32, channels: u16) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLDirectEffectSettings {
            numChannels: channels as i32,
        };
        let mut effect: ffi::IPLDirectEffect = unsafe { std::mem::zeroed() };

        unsafe {
            check(ffi::iplDirectEffectCreate(context.inner, &audio_settings, &effect_settings, &mut effect), ())?;
        }

        Ok(DirectEffect {
            inner: effect,
        })
    }

    pub fn apply(&self, distance_attenuation: Option<f32>, air_absorption: Option<[f32; 3]>, directivity: Option<f32>, occlusion: Option<f32>, transmission: Option<(TransmissionType, [f32; 3])>, in_: &Buffer, out: &mut Buffer) {
        let mut flags = 0;
        if distance_attenuation.is_some() {
            flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYDISTANCEATTENUATION;
        }
        if air_absorption.is_some() {
            flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYAIRABSORPTION;
        }
        if directivity.is_some() {
            flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYDIRECTIVITY;
        }
        if occlusion.is_some() {
            flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYOCCLUSION;
        }
        if transmission.is_some() {
            flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYTRANSMISSION;
        }

        let (transmission_type, transmission) = transmission.unwrap_or_default();
        let params = ffi::IPLDirectEffectParams {
            flags,
            transmissionType: transmission_type.into(),
            distanceAttenuation: distance_attenuation.unwrap_or_default(),
            airAbsorption: air_absorption.unwrap_or_default(),
            directivity: directivity.unwrap_or_default(),
            occlusion: occlusion.unwrap_or_default(),
            transmission,
        };

        unsafe {
            ffi::iplDirectEffectApply(
                self.inner,
                &params,
                &in_.inner,
                &mut out.inner,
            );
        }
    }
}

unsafe impl Sync for DirectEffect {}
unsafe impl Send for DirectEffect {}

impl Clone for DirectEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplDirectEffectRetain(self.inner);
        }

        DirectEffect {
            inner: self.inner,
        }
    }
}

impl Drop for DirectEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplDirectEffectRelease(&mut self.inner);
        }
    }
}

pub enum TransmissionType {
    FrequencyIndependent,
    FrequencyDependent
}

impl Default for TransmissionType {
    fn default() -> Self {
        Self::FrequencyIndependent
    }
}

impl Into<ffi::IPLTransmissionType> for TransmissionType {
    fn into(self) -> ffi::IPLHRTFInterpolation {
        match self {
            Self::FrequencyIndependent => ffi::IPLTransmissionType_IPL_TRANSMISSIONTYPE_FREQINDEPENDENT,
            Self::FrequencyDependent => ffi::IPLTransmissionType_IPL_TRANSMISSIONTYPE_FREQDEPENDENT,
        }
    }
}
