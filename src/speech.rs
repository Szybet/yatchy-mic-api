use log::debug;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const PATH_TO_MODEL: &str = "data/ggml-tiny.bin";

pub fn convert_wav_to_16bit_in_place(path: String) {
    debug!("sox repair before whisper");
    let temp_path = format!("{}tmp.wav", path);

    std::process::Command::new("sox")
        .args(&["--clobber", &path, "-b", "16", &temp_path])
        .status()
        .unwrap();

    std::fs::rename(&temp_path, path).unwrap();
}

pub fn convert_to_16khz_mono_wav_in_place(path: &str) {
    debug!("ffmpeg repair before whisper");
    let original_path = std::path::Path::new(path);
    
    // Create temp file path with .tmp.wav extension
    let temp_path = original_path.with_extension("tmp.wav");
    let temp_str = temp_path.to_str().expect("Invalid temporary path");

    // Convert to WAV using ffmpeg with required parameters
    std::process::Command::new("ffmpeg")
        .args(&[
            "-y",                 // Overwrite without prompting
            "-i", path,           // Input file
            "-ar", "16000",       // Set sample rate
            "-ac", "1",           // Mono audio
            "-c:a", "pcm_s16le",  // 16-bit PCM encoding
            "-hide_banner",       // Cleaner output
            "-loglevel", "error", // Only show errors
            temp_str
        ])
        .status()
        .expect("Failed to execute ffmpeg");

    // Replace original file with converted temp file
    std::fs::rename(temp_str, path)
        .expect("Failed to replace original file with converted version");
}

pub fn get_text_from_wav(path: String) -> String {
    convert_wav_to_16bit_in_place(path.clone());
    convert_to_16khz_mono_wav_in_place(&path.clone());

    let samples: Vec<i16> = hound::WavReader::open(path)
        .unwrap()
        .into_samples::<i16>()
        .map(|x| x.unwrap())
        .collect();

    // load a context and model
    let ctx = WhisperContext::new_with_params(PATH_TO_MODEL, WhisperContextParameters::default())
        .expect("failed to load model");

    let mut state = ctx.create_state().expect("failed to create state");

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // and set the language to translate to to english
    params.set_language(Some("en"));

    // we also explicitly disable anything that prints to stdout
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    // we must convert to 16KHz mono f32 samples for the model
    // some utilities exist for this
    // note that you don't need to use these, you can do it yourself or any other way you want
    // these are just provided for convenience
    // SIMD variants of these functions are also available, but only on nightly Rust: see the docs
    let mut inter_samples = vec![Default::default(); samples.len()];

    whisper_rs::convert_integer_to_float_audio(&samples, &mut inter_samples)
        .expect("failed to convert audio data");
    //let samples = whisper_rs::convert_stereo_to_mono_audio(&inter_samples)
    //    .expect("failed to convert audio data");

    // now we can run the model
    // note the key we use here is the one we created above
    state
        .full(params, &inter_samples[..])
        .expect("failed to run model");

    // fetch the results

    let mut str: String = String::new();

    let num_segments = state
        .full_n_segments()
        .expect("failed to get number of segments");
    for i in 0..num_segments {
        let segment = state
            .full_get_segment_text(i)
            .expect("failed to get segment");
        str.push_str(&segment);
        str.push(' ');
    }
    str.pop();
    str
}
