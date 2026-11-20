use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn read_pcm16_mono_wav(path: &str) -> Vec<i16> {
    let data = std::fs::read(path).expect("read wav");
    // find "data" chunk
    let mut offset = 12usize;
    while offset + 8 <= data.len() {
        let tag = &data[offset..offset + 4];
        let size = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as usize;
        if tag == b"data" {
            let pcm = &data[offset + 8..offset + 8 + size];
            let mut out = Vec::with_capacity(pcm.len() / 2);
            for chunk in pcm.chunks_exact(2) {
                out.push(i16::from_le_bytes([chunk[0], chunk[1]]));
            }
            return out;
        }
        offset += 8 + size + (size % 2);
    }
    panic!("no data chunk");
}

fn main() {
    let model = std::env::args().nth(1).expect("model path");
    let wav = std::env::args().nth(2).expect("wav path");
    let samples = read_pcm16_mono_wav(&wav);
    let ctx = WhisperContext::new_with_params(&model, WhisperContextParameters::default()).expect("load model");
    let mut state = ctx.create_state().expect("create state");
    let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: -1.0 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_n_threads(4);
    let mut inter = vec![0f32; samples.len()];
    whisper_rs::convert_integer_to_float_audio(&samples, &mut inter).expect("convert");
    state.full(params, &inter).expect("run model");
    for segment in state.as_iter() {
        println!("[{} - {}ms]: {}", segment.start_timestamp() * 10, segment.end_timestamp() * 10, segment);
    }
}
