use super::{Transcription, TranscriptionBackend};
use crate::config::profile::Language;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct LocalWhisperBackend {
    ctx: Arc<WhisperContext>,
}

impl LocalWhisperBackend {
    pub fn new(model_path: PathBuf) -> Result<Self> {
        let path_str = model_path
            .to_str()
            .ok_or_else(|| AppError::Transcription("bad model path".into()))?;
        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| AppError::Transcription(e.to_string()))?;
        Ok(Self {
            ctx: Arc::new(ctx),
        })
    }

    fn lang_code(l: &Language) -> Option<&'static str> {
        match l {
            Language::De => Some("de"),
            Language::En => Some("en"),
            Language::Auto => None,
        }
    }
}

#[async_trait]
impl TranscriptionBackend for LocalWhisperBackend {
    fn id(&self) -> &'static str {
        "local-whisper"
    }

    async fn transcribe(
        &self,
        samples: &[f32],
        language: Language,
        vocabulary: &[String],
    ) -> Result<Transcription> {
        let samples_owned = samples.to_vec();
        let vocab_prompt = vocabulary.join(", ");
        let ctx = self.ctx.clone();
        let lang = Self::lang_code(&language);

        let (text, ms) =
            tokio::task::spawn_blocking(move || -> Result<(String, u64)> {
                let mut state = ctx
                    .create_state()
                    .map_err(|e| AppError::Transcription(e.to_string()))?;
                // BeamSearch ist robuster gegen Repeat-Loops als Greedy,
                // besonders bei kurzen Utterances (<2s), wo das small-Modell
                // auf CPU gerne in Wiederholungen verfällt.
                let mut params = FullParams::new(SamplingStrategy::BeamSearch {
                    beam_size: 5,
                    patience: -1.0,
                });
                if let Some(l) = lang {
                    params.set_language(Some(l));
                }
                if !vocab_prompt.is_empty() {
                    params.set_initial_prompt(&vocab_prompt);
                }
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_special(false);
                // Unterdrücke [Musik], [Zwischenruf], [Applaus] & Co. — Whisper
                // halluziniert diese Tokens bei Stille oder leiser Aufnahme.
                params.set_suppress_non_speech_tokens(true);
                params.set_suppress_blank(true);
                // Verhindere, dass Whisper den vorherigen Output in den nächsten
                // Segment-Context mitnimmt — genau das triggert die endlose
                // Wiederholung ("[Zwischenruf] [Zwischenruf] ...").
                params.set_no_context(true);
                params.set_temperature(0.0);
                // Aktiviere whisper.cpp's Fallback-Mechanismus: wenn ein Segment
                // niedrige Token-Entropie hat (= Repeat-Loop) oder niedrige
                // Confidence, wird es mit höherer Temperature neu dekodiert.
                // Ohne temperature_inc > 0 gibt es keinen Retry und der Greedy-
                // Loop bleibt im Output.
                params.set_temperature_inc(0.2);
                params.set_entropy_thold(3.0);
                params.set_logprob_thold(-0.5);
                // Aggressiveres no-speech-Gating.
                params.set_no_speech_thold(0.6);

                let start = Instant::now();
                state
                    .full(params, &samples_owned)
                    .map_err(|e| AppError::Transcription(e.to_string()))?;

                let num_segments = state
                    .full_n_segments()
                    .map_err(|e| AppError::Transcription(e.to_string()))?;
                let mut text = String::new();
                for i in 0..num_segments {
                    text.push_str(
                        &state
                            .full_get_segment_text(i)
                            .map_err(|e| AppError::Transcription(e.to_string()))?,
                    );
                }
                Ok((collapse_repetitions(text.trim()), start.elapsed().as_millis() as u64))
            })
            .await
            .map_err(|e| AppError::Transcription(e.to_string()))??;

        Ok(Transcription {
            text,
            duration_ms: ms,
            backend_id: "local-whisper",
        })
    }
}

/// Safety-Net gegen übrig gebliebene Repeat-Loops: wenn das Transkript
/// eine Phrase (>= 4 Wörter) unmittelbar mindestens dreimal in Folge
/// wiederholt, kappe auf ein einziges Vorkommen. Dämpft den Worst-Case,
/// in dem whisper.cpp's temperature-Fallback trotz `temperature_inc` und
/// `entropy_thold` nicht greift ("Die Bekleidung wird… Die Bekleidung
/// wird… Die Bekleidung wird…").
fn collapse_repetitions(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let n = words.len();
    // Suche die *kürzeste* Phrase (4..=n/3 Wörter), die direkt dreimal
    // hintereinander vorkommt. Kürzeste bevorzugen, damit wir die
    // atomare Wiederholungseinheit finden und nicht zwei Kopien davon
    // als "eine Phrase" behandeln.
    for phrase_len in 4..=n / 3 {
        for start in 0..=(n - phrase_len * 3) {
            let a = &words[start..start + phrase_len];
            let b = &words[start + phrase_len..start + phrase_len * 2];
            let c = &words[start + phrase_len * 2..start + phrase_len * 3];
            if a == b && b == c {
                let mut kept: Vec<&str> =
                    words[..start + phrase_len].to_vec();
                // Überspringe alle weiteren direkt aufeinanderfolgenden
                // Wiederholungen der gleichen Phrase.
                let mut cursor = start + phrase_len * 3;
                while cursor + phrase_len <= n
                    && &words[cursor..cursor + phrase_len] == a
                {
                    cursor += phrase_len;
                }
                kept.extend_from_slice(&words[cursor..]);
                return kept.join(" ");
            }
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::collapse_repetitions;

    #[test]
    fn passthrough_non_repeating() {
        let s = "OK dann Option A ist gut";
        assert_eq!(collapse_repetitions(s), s);
    }

    #[test]
    fn collapses_triple_phrase() {
        let s = "Die Bekleidung wird geschnitten. Die Bekleidung wird geschnitten. Die Bekleidung wird geschnitten.";
        let out = collapse_repetitions(s);
        assert_eq!(out, "Die Bekleidung wird geschnitten.");
    }

    #[test]
    fn collapses_many_repetitions() {
        let phrase = "ja ich habe das verstanden";
        let s = vec![phrase; 8].join(" ");
        assert_eq!(collapse_repetitions(&s), phrase);
    }

    #[test]
    fn keeps_short_duplicate_words() {
        // "ja ja" ist zwei Wörter — unter der 4-Wort-Phrase-Schwelle.
        let s = "ja ja ja ja ja";
        assert_eq!(collapse_repetitions(s), s);
    }
}
