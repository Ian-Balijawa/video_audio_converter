use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::io::{BufRead, BufReader};
use std::path::Path;
use regex::Regex;

#[derive(Debug)]
pub struct ConversionProgress {
    pub duration_seconds: f64,
    pub processed_seconds: f64,
    pub percentage: f64,
    pub speed: f64,
    pub bitrate: String,
}

#[derive(Debug)]
pub enum ConversionError {
    FileNotFound,
    InvalidFormat,
    FFmpegError(String),
    IOError(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConversionError::FileNotFound => write!(f, "Input file not found"),
            ConversionError::InvalidFormat => write!(f, "Invalid video format"),
            ConversionError::FFmpegError(msg) => write!(f, "FFmpeg error: {}", msg),
            ConversionError::IOError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ConversionError {}

pub struct VideoToAudioConverter {
    ffmpeg_path: String,
    progress: Arc<Mutex<ConversionProgress>>,
}

impl VideoToAudioConverter {
    pub fn new() -> Result<Self, ConversionError> {
        let ffmpeg_path = Self::find_ffmpeg()?;
        Ok(Self {
            ffmpeg_path,
            progress: Arc::new(Mutex::new(ConversionProgress {
                duration_seconds: 0.0,
                processed_seconds: 0.0,
                percentage: 0.0,
                speed: 0.0,
                bitrate: String::new(),
            })),
        })
    }

    fn find_ffmpeg() -> Result<String, ConversionError> {
        let paths = vec!["ffmpeg", "/usr/bin/ffmpeg", "/usr/local/bin/ffmpeg"];
        
        for path in paths {
            if Command::new(path).arg("-version").output().is_ok() {
                return Ok(path.to_string());
            }
        }
        
        Err(ConversionError::FFmpegError("FFmpeg not found in PATH".to_string()))
    }

    pub fn get_video_duration(&self, input_path: &str) -> Result<f64, ConversionError> {
        let output = Command::new(&self.ffmpeg_path)
            .args(&["-i", input_path, "-f", "null", "-"])
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| ConversionError::IOError(e.to_string()))?
            .wait_with_output()
            .map_err(|e| ConversionError::IOError(e.to_string()))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let duration_regex = Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();
        
        if let Some(captures) = duration_regex.captures(&stderr) {
            let hours: f64 = captures[1].parse().unwrap_or(0.0);
            let minutes: f64 = captures[2].parse().unwrap_or(0.0);
            let seconds: f64 = captures[3].parse().unwrap_or(0.0);
            let centiseconds: f64 = captures[4].parse().unwrap_or(0.0);
            
            Ok(hours * 3600.0 + minutes * 60.0 + seconds + centiseconds / 100.0)
        } else {
            Err(ConversionError::InvalidFormat)
        }
    }

    pub fn convert<F>(&self, input_path: &str, output_path: &str, mut progress_callback: F) -> Result<(), ConversionError>
    where
        F: FnMut(&ConversionProgress) + Send + 'static,
    {
        if !Path::new(input_path).exists() {
            return Err(ConversionError::FileNotFound);
        }

        let duration = self.get_video_duration(input_path)?;
        {
            let mut progress = self.progress.lock().unwrap();
            progress.duration_seconds = duration;
        }

        let mut child = Command::new(&self.ffmpeg_path)
            .args(&[
                "-i", input_path,
                "-vn",                    // No video
                "-acodec", "libmp3lame",  // MP3 codec
                "-ab", "192k",            // Audio bitrate
                "-ar", "44100",           // Sample rate
                "-y",                     // Overwrite output file
                "-progress", "pipe:2",    // Progress to stderr
                output_path
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| ConversionError::IOError(e.to_string()))?;

        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        let progress_arc = Arc::clone(&self.progress);

        let progress_thread = thread::spawn(move || {
            let time_regex = Regex::new(r"out_time_ms=(\d+)").unwrap();
            let speed_regex = Regex::new(r"speed=([0-9.]+)x").unwrap();
            let bitrate_regex = Regex::new(r"bitrate=([0-9.]+kbits/s)").unwrap();

            for line in reader.lines() {
                if let Ok(line) = line {
                    let mut progress = progress_arc.lock().unwrap();
                    
                    if let Some(captures) = time_regex.captures(&line) {
                        if let Ok(microseconds) = captures[1].parse::<u64>() {
                            progress.processed_seconds = microseconds as f64 / 1_000_000.0;
                            if progress.duration_seconds > 0.0 {
                                progress.percentage = (progress.processed_seconds / progress.duration_seconds) * 100.0;
                            }
                        }
                    }
                    
                    if let Some(captures) = speed_regex.captures(&line) {
                        if let Ok(speed) = captures[1].parse::<f64>() {
                            progress.speed = speed;
                        }
                    }
                    
                    if let Some(captures) = bitrate_regex.captures(&line) {
                        progress.bitrate = captures[1].to_string();
                    }

                    progress_callback(&progress);
                }
            }
        });

        let status = child.wait().map_err(|e| ConversionError::IOError(e.to_string()))?;
        progress_thread.join().unwrap();

        if !status.success() {
            return Err(ConversionError::FFmpegError("Conversion failed".to_string()));
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        println!("Usage: {} <input_video> <output_audio.mp3>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Starting conversion: {} -> {}", input_path, output_path);
    
    let converter = VideoToAudioConverter::new()?;
    
    let start_time = std::time::Instant::now();
    
    converter.convert(input_path, output_path, {
        let mut last_update = std::time::Instant::now();
        move |progress| {
            let now = std::time::Instant::now();
            if now.duration_since(last_update) >= Duration::from_millis(250) {
                // Create progress bar
                let progress_width = 50;
                let filled = (progress.percentage / 100.0 * progress_width as f64) as usize;
                let empty = progress_width - filled;
                let bar = format!("{}{}",
                    "█".repeat(filled),
                    "░".repeat(empty)
                );
                
                // Calculate ETA
                let eta = if progress.speed > 0.0 {
                    let remaining_seconds = (progress.duration_seconds - progress.processed_seconds) / progress.speed;
                    if remaining_seconds < 60.0 {
                        format!("{:.0}s", remaining_seconds)
                    } else {
                        format!("{:.0}m {:.0}s", remaining_seconds / 60.0, remaining_seconds % 60.0)
                    }
                } else {
                    "calculating...".to_string()
                };

                print!("\r🎵 [{}] {:.1}% | Speed: {:.2}x | ETA: {} | {:.1}s/{:.1}s ",
                    bar,
                    progress.percentage.min(100.0),
                    progress.speed,
                    eta,
                    progress.processed_seconds,
                    progress.duration_seconds
                );
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                last_update = now;
            }
        }
    })?;

    let elapsed = start_time.elapsed();
    println!("\n✅ Conversion completed in {:.2}s", elapsed.as_secs_f64());
    println!("Output file: {}", output_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let result = VideoToAudioConverter::new();
        // This test will pass if FFmpeg is installed
        match result {
            Ok(_) => println!("FFmpeg found"),
            Err(e) => println!("FFmpeg not available: {}", e),
        }
    }

    #[test]
    fn test_invalid_file() {
        if let Ok(converter) = VideoToAudioConverter::new() {
            let result = converter.convert("nonexistent.mp4", "output.mp3", |_| {});
            assert!(matches!(result, Err(ConversionError::FileNotFound)));
        }
    }
}
