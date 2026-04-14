pub mod llm;
pub mod local;
pub mod remote;

use crate::config::profile::Language;
use crate::error::Result;
use async_trait::async_trait;

pub struct Transcription {
    pub text: String,
    pub duration_ms: u64,
    pub backend_id: &'static str,
}

#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    async fn transcribe(
        &self,
        pcm_16k_mono: &[f32],
        language: Language,
        vocabulary: &[String],
    ) -> Result<Transcription>;

    fn id(&self) -> &'static str;

    async fn is_available(&self) -> bool {
        true
    }
}

pub fn pcm_to_wav_16k_mono(samples: &[f32]) -> Result<Vec<u8>> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut buf = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        let mut writer = WavWriter::new(cursor, spec)
            .map_err(|e| crate::error::AppError::Transcription(e.to_string()))?;
        for s in samples {
            let clamped = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(clamped)
                .map_err(|e| crate::error::AppError::Transcription(e.to_string()))?;
        }
        writer
            .finalize()
            .map_err(|e| crate::error::AppError::Transcription(e.to_string()))?;
    }
    Ok(buf)
}
