# Video to Audio Converter

A high-performance Rust application that extracts audio from video files and converts them to MP3 format using FFmpeg.

## Features

• **Multi-threaded processing** for optimal performance
• **Real-time progress tracking** with visual progress bar
• **ETA calculation** showing estimated time remaining
• **Industry-standard quality** using libmp3lame codec
• **Cross-platform support** (Windows, macOS, Linux)
• **Comprehensive error handling** with detailed feedback
• **Support for all major video formats** (MP4, AVI, MKV, MOV, etc.)

## Prerequisites

You need FFmpeg installed on your system:

**macOS:**
```bash
brew install ffmpeg
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install ffmpeg
```

**Windows:**
Download from [ffmpeg.org](https://ffmpeg.org/download.html) and add to PATH

## Installation

Clone the repository:
```bash
git clone <repository-url>
cd video_audio_converter
```

Build the project:
```bash
cargo build --release
```

## Usage

Convert a video file to MP3:
```bash
./target/release/video_audio_converter input_video.mp4 output_audio.mp3
```

### Command Line Arguments

- `input_video` - Path to your video file
- `output_audio.mp3` - Desired output path for MP3 file

### Example

```bash
./target/release/video_audio_converter movie.mkv soundtrack.mp3
```

## Progress Display

The converter shows real-time progress with:

```
🎵 [████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 64.2% | Speed: 2.3x | ETA: 1m 23s | 45.2s/70.5s
```

• **Visual progress bar** - Shows completion status
• **Percentage complete** - Exact progress percentage
• **Conversion speed** - Processing speed multiplier
• **Time estimates** - ETA and elapsed/total time

## Audio Quality Settings

The converter uses optimized settings for high-quality MP3 output:

• **Codec:** libmp3lame (industry standard)
• **Bitrate:** 192 kbps (high quality)
• **Sample rate:** 44.1 kHz (CD quality)
• **Channels:** Preserved from source

## Supported Input Formats

• MP4, AVI, MKV, MOV
• WMV, FLV, WebM
• 3GP, ASF, OGV
• Any format supported by FFmpeg

## Error Handling

The application handles common issues:

• **File not found** - Validates input file exists
• **Invalid format** - Checks video format compatibility
• **FFmpeg errors** - Reports conversion failures
• **IO errors** - Handles file system issues

## Performance

• **Multi-threaded architecture** maximizes CPU usage
• **Optimized FFmpeg parameters** for speed
• **Minimal memory footprint** during conversion
• **Progress updates** don't impact conversion speed

## Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
regex = "1.0"
```

## Project Structure

```
src/
├── main.rs              # Main application logic
├── lib.rs               # Library components (optional)
└── tests/               # Unit tests

Cargo.toml               # Project dependencies
README.md                # This file
```

## Development

Run in development mode:
```bash
cargo run input.mp4 output.mp3
```

Run tests:
```bash
cargo test
```

Build optimized release:
```bash
cargo build --release
```

## Examples

**Convert movie to audio:**
```bash
./video_audio_converter movie.mp4 movie_audio.mp3
```

**Extract podcast audio:**
```bash
./video_audio_converter podcast.mkv podcast.mp3
```

**Process multiple files:**
```bash
for file in *.mp4; do
    ./video_audio_converter "$file" "${file%.*}.mp3"
done
```

## Troubleshooting

**FFmpeg not found:**
- Install FFmpeg using your system package manager
- Ensure FFmpeg is in your PATH
- Try specifying full path to FFmpeg binary

**Conversion fails:**
- Check input file isn't corrupted
- Verify you have write permissions to output directory
- Ensure sufficient disk space available

**Slow conversion:**
- Use SSD storage for faster I/O
- Close other CPU-intensive applications
- Check available system memory

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

• **FFmpeg team** for the powerful multimedia framework
• **Rust community** for excellent tooling and libraries
• **libmp3lame** developers for high-quality MP3 encoding
