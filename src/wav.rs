use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Error, ErrorKind};

pub fn boost_wav(input_path: &str, gain_factor: f32) -> Result<(), Error> {
    let input_path = Path::new(input_path);
    
    // Create temporary file with .wav extension
    let temp_output = input_path.with_file_name(format!(
        "{}.tmp.wav",
        input_path.file_stem()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Invalid filename"))?
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Non-UTF8 filename"))?
    ));

    // Check ffmpeg exists
    let ffmpeg_check = Command::new("which").arg("ffmpeg").output()?;
    if !ffmpeg_check.status.success() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "ffmpeg not found. Install with: sudo apt install ffmpeg",
        ));
    }

    // Run FFmpeg with explicit WAV format
    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i", input_path.to_str().unwrap(),
            "-filter:a", &format!("volume={}", gain_factor),
            "-f", "wav",  // Force WAV format
            temp_output.to_str().unwrap()
        ])
        .status()?;

    if !status.success() {
        fs::remove_file(&temp_output).ok();
        return Err(Error::new(
            ErrorKind::Other,
            format!("FFmpeg failed with exit code: {}", status),
        ));
    }

    // Replace original file
    fs::remove_file(input_path)?;
    fs::rename(&temp_output, input_path)?;

    Ok(())
}