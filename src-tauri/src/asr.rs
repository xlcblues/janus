//! Offline speech recognition for the M4 speaking module.
//!
//! Uses whisper.cpp through whisper-rs 0.16 with the bundled tiny.en model.
//! Heavy inference runs on a blocking thread pool, never on the UI thread.

use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Wraps the Whisper model context. `WhisperContext` is `Send + Sync`, so the
/// engine can be shared across async commands and cloned into blocking tasks.
pub struct AsrEngine {
    ctx: WhisperContext,
}

/// Result of one transcription pass.
pub struct Transcription {
    pub text: String,
    /// 0..1 confidence derived from whisper's no-speech probability
    /// (1 - avg(no_speech_prob), clamped). Lower means the recognizer is
    /// unsure; we never treat low-confidence output as a wrong answer.
    pub confidence: f32,
    pub low_confidence: bool,
}

impl AsrEngine {
    pub fn load(model_path: &Path) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|error| format!("语音识别模型加载失败：{error}"))?;
        Ok(Self { ctx })
    }

    /// Transcribe a 16 kHz mono 16-bit PCM WAV blob into English text.
    pub fn transcribe(&self, wav: &[u8]) -> Result<Transcription, String> {
        let samples = parse_pcm16_mono_wav(wav)?;
        if samples.is_empty() {
            return Err("录音内容为空，请重新录制。".to_string());
        }

        let mut inter = vec![0f32; samples.len()];
        whisper_rs::convert_integer_to_float_audio(&samples, &mut inter)
            .map_err(|error| format!("录音格式转换失败：{error}"))?;

        let mut state = self.ctx.create_state().map_err(|e| e.to_string())?;
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_n_threads(4);
        state
            .full(params, &inter)
            .map_err(|error| format!("语音识别失败：{error}"))?;

        let mut text_parts = Vec::new();
        let mut no_speech_sum = 0.0f32;
        let mut no_speech_count = 0u32;
        for segment in state.as_iter() {
            let text = segment
            .to_str_lossy()
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
            if !text.is_empty() {
                text_parts.push(text);
            }
            no_speech_sum += segment.no_speech_probability();
            no_speech_count += 1;
        }

        let text = text_parts.join(" ");
        let confidence = if no_speech_count > 0 {
            (1.0 - no_speech_sum / no_speech_count as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let low_confidence = text.trim().is_empty() || confidence < 0.5;
        Ok(Transcription {
            text,
            confidence,
            low_confidence,
        })
    }
}

/// Parse a 16-bit PCM mono WAV blob into samples by scanning RIFF chunks.
/// Tolerates non-standard 'fmt ' chunk sizes and chunk padding bytes.
fn parse_pcm16_mono_wav(wav: &[u8]) -> Result<Vec<i16>, String> {
    if wav.len() < 12 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Err("录音文件格式不正确（非 WAV）。".to_string());
    }
    let mut audio_format: u16 = 0;
    let mut channels: u16 = 0;
    let mut sample_rate: u32 = 0;
    let mut bits: u16 = 0;
    let mut data: Option<&[u8]> = None;

    let mut offset = 12usize;
    while offset + 8 <= wav.len() {
        let tag = &wav[offset..offset + 4];
        let size = u32::from_le_bytes(wav[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let body = &wav[offset + 8..];
        let payload = body.get(..size.min(body.len())).unwrap_or(body);
        match tag {
            b"fmt " => {
                if payload.len() >= 16 {
                    audio_format = u16::from_le_bytes([payload[0], payload[1]]);
                    channels = u16::from_le_bytes([payload[2], payload[3]]);
                    sample_rate = u32::from_le_bytes(payload[4..8].try_into().unwrap());
                    bits = u16::from_le_bytes([payload[14], payload[15]]);
                }
            }
            b"data" => {
                data = Some(payload);
            }
            _ => {}
        }
        offset += 8 + size + (size % 2);
    }

    if audio_format != 1 || channels != 1 || sample_rate != 16000 || bits != 16 {
        return Err("仅支持 16kHz 单声道 16bit PCM 录音。".to_string());
    }
    let payload = data.ok_or_else(|| "录音数据为空，请重新录制。".to_string())?;
    if payload.is_empty() {
        return Err("录音数据为空，请重新录制。".to_string());
    }
    let mut out = Vec::with_capacity(payload.len() / 2);
    for chunk in payload.chunks_exact(2) {
        out.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm16_wav(samples: &[i16]) -> Vec<u8> {
        let data_len = samples.len() * 2;
        let mut wav = Vec::with_capacity(44 + data_len);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&((36 + data_len) as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&16000u32.to_le_bytes());
        wav.extend_from_slice(&32000u32.to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_len as u32).to_le_bytes());
        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        wav
    }

    #[test]
    fn parses_valid_pcm16_mono_wav() {
        let wav = pcm16_wav(&[0i16, 100, -100, 32767, -32768]);
        let samples = parse_pcm16_mono_wav(&wav).unwrap();
        assert_eq!(samples, vec![0, 100, -100, 32767, -32768]);
    }

    #[test]
    fn rejects_unsupported_wav_formats() {
        let mut wav = pcm16_wav(&[0i16, 1, 2]);
        // Simulate 44.1 kHz by patching the sample rate field.
        wav[24..28].copy_from_slice(&44100u32.to_le_bytes());
        assert!(parse_pcm16_mono_wav(&wav).is_err());

        assert!(parse_pcm16_mono_wav(b"not a wav at all").is_err());
    }

    #[test]
    fn rejects_empty_audio() {
        let wav = pcm16_wav(&[]);
        assert!(parse_pcm16_mono_wav(&wav).is_err());
    }
}
