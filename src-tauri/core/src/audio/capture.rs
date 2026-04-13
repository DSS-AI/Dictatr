use crate::audio::ringbuffer::RingBuffer;
use crate::error::{AppError, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use parking_lot::Mutex;
use std::sync::Arc;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

pub struct AudioCapture {
    stream: Option<Stream>,
    pub buffer: Arc<Mutex<RingBuffer>>,
    pub level: Arc<Mutex<f32>>,
}

impl AudioCapture {
    pub fn new(max_seconds: u32) -> Self {
        Self {
            stream: None,
            buffer: Arc::new(Mutex::new(RingBuffer::with_seconds(max_seconds, TARGET_SAMPLE_RATE))),
            level: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Construct an `AudioCapture` using pre-existing shared `Arc` handles.
    ///
    /// Used by `AudioController::spawn` to share the level meter and ring
    /// buffer with the controller before the audio thread is started, without
    /// ever moving an `AudioCapture` (which holds a `!Send` `Stream`) across
    /// thread boundaries.
    pub fn with_shared(level: Arc<Mutex<f32>>, buffer: Arc<Mutex<RingBuffer>>) -> Self {
        Self { stream: None, buffer, level }
    }

    pub fn list_input_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices = host.input_devices().map_err(|e| AppError::Audio(e.to_string()))?;
        Ok(devices.filter_map(|d| d.name().ok()).collect())
    }

    pub fn start(&mut self, device_name: Option<&str>) -> Result<()> {
        if self.stream.is_some() { return Ok(()); }

        let host = cpal::default_host();
        let device: Device = match device_name {
            Some(name) => host.input_devices()
                .map_err(|e| AppError::Audio(e.to_string()))?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| AppError::Audio(format!("device not found: {}", name)))?,
            None => host.default_input_device()
                .ok_or_else(|| AppError::Audio("no default input device".into()))?,
        };

        let config = device.default_input_config()
            .map_err(|e| AppError::Audio(e.to_string()))?;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.clone().into();
        let source_rate = stream_config.sample_rate.0;
        let channels = stream_config.channels as usize;

        let buf = self.buffer.clone();
        let level = self.level.clone();

        let err_cb = |err| eprintln!("audio stream error: {err}");

        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| Self::on_chunk(data, channels, source_rate, &buf, &level),
                err_cb, None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
                    Self::on_chunk(&f, channels, source_rate, &buf, &level);
                },
                err_cb, None,
            ),
            other => return Err(AppError::Audio(format!("unsupported sample format: {:?}", other))),
        }.map_err(|e| AppError::Audio(e.to_string()))?;

        stream.play().map_err(|e| AppError::Audio(e.to_string()))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None;
    }

    fn on_chunk(
        data: &[f32], channels: usize, source_rate: u32,
        buf: &Arc<Mutex<RingBuffer>>, level: &Arc<Mutex<f32>>,
    ) {
        let mono: Vec<f32> = if channels == 1 {
            data.to_vec()
        } else {
            data.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect()
        };

        let resampled = resample_linear(&mono, source_rate, TARGET_SAMPLE_RATE);

        let rms = (resampled.iter().map(|s| s * s).sum::<f32>() / resampled.len().max(1) as f32).sqrt();
        *level.lock() = rms;

        buf.lock().push_samples(&resampled);
    }
}

pub(crate) fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate { return input.to_vec(); }
    let ratio = to_rate as f32 / from_rate as f32;
    let out_len = (input.len() as f32 * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f32 / ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f32;
        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_passthrough() {
        let input = vec![1.0, 2.0, 3.0];
        let out = resample_linear(&input, 16_000, 16_000);
        assert_eq!(out, input);
    }

    #[test]
    fn resample_downsamples() {
        let input: Vec<f32> = (0..32).map(|i| i as f32).collect();
        let out = resample_linear(&input, 32_000, 16_000);
        assert_eq!(out.len(), 16);
    }
}
