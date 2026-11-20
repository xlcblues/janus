// Auto-generated seed listening clips (Microsoft Zira en-US offline voice, 16 kHz mono WAV).
// Sentences mirror the sentences table in db.rs MIGRATION_1; written into the media cache at setup.

pub struct SeedClip {
    pub title: &'static str,
    pub text_zh: &'static str,
    pub wav: &'static [u8],
}

const SEED_WAV_01: &[u8] = include_bytes!("../resources/media/seed/01.wav");
const SEED_WAV_02: &[u8] = include_bytes!("../resources/media/seed/02.wav");
const SEED_WAV_03: &[u8] = include_bytes!("../resources/media/seed/03.wav");
const SEED_WAV_04: &[u8] = include_bytes!("../resources/media/seed/04.wav");
const SEED_WAV_05: &[u8] = include_bytes!("../resources/media/seed/05.wav");
const SEED_WAV_06: &[u8] = include_bytes!("../resources/media/seed/06.wav");
const SEED_WAV_07: &[u8] = include_bytes!("../resources/media/seed/07.wav");
const SEED_WAV_08: &[u8] = include_bytes!("../resources/media/seed/08.wav");
const SEED_WAV_09: &[u8] = include_bytes!("../resources/media/seed/09.wav");
const SEED_WAV_10: &[u8] = include_bytes!("../resources/media/seed/10.wav");
const SEED_WAV_11: &[u8] = include_bytes!("../resources/media/seed/11.wav");
const SEED_WAV_12: &[u8] = include_bytes!("../resources/media/seed/12.wav");
const SEED_WAV_13: &[u8] = include_bytes!("../resources/media/seed/13.wav");
const SEED_WAV_14: &[u8] = include_bytes!("../resources/media/seed/14.wav");
const SEED_WAV_15: &[u8] = include_bytes!("../resources/media/seed/15.wav");
const SEED_WAV_16: &[u8] = include_bytes!("../resources/media/seed/16.wav");
const SEED_WAV_17: &[u8] = include_bytes!("../resources/media/seed/17.wav");
const SEED_WAV_18: &[u8] = include_bytes!("../resources/media/seed/18.wav");
const SEED_WAV_19: &[u8] = include_bytes!("../resources/media/seed/19.wav");
const SEED_WAV_20: &[u8] = include_bytes!("../resources/media/seed/20.wav");

pub static SEED_CLIPS: &[SeedClip] = &[
    SeedClip { title: "It depends on how you look at it.", text_zh: "取决于你从哪个角度看。", wav: SEED_WAV_01 },
    SeedClip { title: "Could you say that again, please?", text_zh: "可以请你再说一遍吗？", wav: SEED_WAV_02 },
    SeedClip { title: "I am running out of time.", text_zh: "我的时间快不够了。", wav: SEED_WAV_03 },
    SeedClip { title: "Let us take a closer look.", text_zh: "我们再仔细看一下。", wav: SEED_WAV_04 },
    SeedClip { title: "That makes perfect sense.", text_zh: "这完全说得通。", wav: SEED_WAV_05 },
    SeedClip { title: "I will keep that in mind.", text_zh: "我会记住这一点。", wav: SEED_WAV_06 },
    SeedClip { title: "It is worth giving it a try.", text_zh: "值得试一试。", wav: SEED_WAV_07 },
    SeedClip { title: "We are on the same page.", text_zh: "我们看法一致。", wav: SEED_WAV_08 },
    SeedClip { title: "Could you be more specific?", text_zh: "你能说得更具体一点吗？", wav: SEED_WAV_09 },
    SeedClip { title: "I have a lot on my plate right now.", text_zh: "我现在手头有很多事。", wav: SEED_WAV_10 },
    SeedClip { title: "That is a great question.", text_zh: "这是个好问题。", wav: SEED_WAV_11 },
    SeedClip { title: "Let me walk you through it.", text_zh: "我来带你过一遍。", wav: SEED_WAV_12 },
    SeedClip { title: "It is not as simple as it sounds.", text_zh: "事情没有听起来那么简单。", wav: SEED_WAV_13 },
    SeedClip { title: "I would rather get it right than hurry.", text_zh: "我宁可做对，也不愿赶时间。", wav: SEED_WAV_14 },
    SeedClip { title: "The sooner we start, the better.", text_zh: "我们越早开始越好。", wav: SEED_WAV_15 },
    SeedClip { title: "Could you give me a hand?", text_zh: "你能帮我一下吗？", wav: SEED_WAV_16 },
    SeedClip { title: "I will get back to you tomorrow.", text_zh: "我明天再答复你。", wav: SEED_WAV_17 },
    SeedClip { title: "Practice makes progress, not perfection.", text_zh: "练习带来进步，而不是完美。", wav: SEED_WAV_18 },
    SeedClip { title: "Every small step counts.", text_zh: "每一步都很重要。", wav: SEED_WAV_19 },
    SeedClip { title: "You will get there with time.", text_zh: "假以时日，你会做到的。", wav: SEED_WAV_20 },
];

/// Parse the duration (ms) of a PCM WAV blob by scanning RIFF chunks.
/// Handles non-standard `fmt ` chunk sizes (e.g. 18-byte Microsoft headers).
pub fn wav_duration_ms(wav: &[u8]) -> i64 {
    if wav.len() < 12 || &wav[0..4] != b"RIFF" {
        return 0;
    }
    let mut offset = 12usize;
    let mut byte_rate = 0u32;
    let mut data_size = 0u32;
    while offset + 8 <= wav.len() {
        let chunk_id = &wav[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            wav[offset + 4],
            wav[offset + 5],
            wav[offset + 6],
            wav[offset + 7],
        ]) as usize;
        if chunk_id == b"fmt " && offset + 8 + 12 <= wav.len() {
            byte_rate = u32::from_le_bytes([
                wav[offset + 8 + 8],
                wav[offset + 8 + 9],
                wav[offset + 8 + 10],
                wav[offset + 8 + 11],
            ]);
        } else if chunk_id == b"data" {
            data_size = chunk_size as u32;
            break;
        }
        offset += 8 + chunk_size + (chunk_size % 2);
    }
    if byte_rate == 0 {
        0
    } else {
        data_size as i64 * 1000 / byte_rate as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_wavs_have_plausible_durations() {
        assert_eq!(SEED_CLIPS.len(), 20);
        for clip in SEED_CLIPS {
            let ms = wav_duration_ms(clip.wav);
            assert!(
                (1000..=6000).contains(&ms),
                "unexpected duration {ms} ms for: {}",
                clip.title
            );
        }
    }
}
