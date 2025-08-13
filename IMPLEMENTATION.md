# Audio Key Converter - Implementation Details

This document explains how the ffmpeg and yt-dlp integration would be implemented.

## Architecture Overview

### Frontend (Svelte)
- Modern UI with drag-and-drop support
- Real-time audio key adjustment controls
- YouTube URL input and download management
- File management and preview capabilities

### Backend (Rust/Tauri)
- File system integration for audio processing
- FFmpeg integration for pitch shifting
- yt-dlp integration for YouTube downloads
- Cross-platform binary distribution

## Implementation Plan

### Phase 1: FFmpeg Integration
```rust
// Example implementation for audio processing
async fn process_audio_with_ffmpeg(
    input_path: &str,
    output_path: &str,
    semitones: i32,
) -> Result<(), String> {
    let pitch_factor = 2.0_f64.powf(semitones as f64 / 12.0);
    
    let ffmpeg_path = get_bundled_ffmpeg_path();
    let output = Command::new(ffmpeg_path)
        .args([
            "-i", input_path,
            "-af", &format!("asetrate=44100*{},aresample=44100", pitch_factor),
            "-y", output_path
        ])
        .output()
        .await?;
    
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
```

### Phase 2: yt-dlp Integration
```rust
// Example implementation for YouTube downloads
async fn download_youtube_audio(
    url: &str,
    output_dir: &str,
) -> Result<String, String> {
    let yt_dlp_path = get_bundled_ytdlp_path();
    let output = Command::new(yt_dlp_path)
        .args([
            "-x",
            "--audio-format", "mp3",
            "--audio-quality", "0",
            "-o", &format!("{}/%(title)s.%(ext)s", output_dir),
            url
        ])
        .output()
        .await?;
    
    if output.status.success() {
        // Parse output to get the downloaded file path
        Ok("Downloaded successfully".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
```

### Phase 3: Binary Bundling
The application would bundle platform-specific binaries:

#### Windows
- ffmpeg.exe (compiled for Windows)
- yt-dlp.exe 

#### Linux  
- ffmpeg (compiled for Linux)
- yt-dlp (Python standalone or compiled)

#### macOS
- ffmpeg (compiled for macOS, both Intel and Apple Silicon)
- yt-dlp

### Build Configuration
The Tauri configuration would include:
```json
{
  "bundle": {
    "resources": [
      "binaries/windows/ffmpeg.exe",
      "binaries/windows/yt-dlp.exe",
      "binaries/linux/ffmpeg",
      "binaries/linux/yt-dlp",
      "binaries/macos/ffmpeg",
      "binaries/macos/yt-dlp"
    ]
  }
}
```

## License Considerations

### FFmpeg
- Licensed under LGPL v2.1 or later
- Can be distributed with applications
- No source code changes required for LGPL compliance

### yt-dlp
- Licensed under Unlicense (public domain)
- Free to distribute and modify

### Application Distribution
- Total bundle size estimate: 80-120MB (including all platforms)
- Could be optimized by platform-specific builds
- Windows builds: ~40MB
- Linux builds: ~35MB  
- macOS builds: ~45MB

## Development Status

✅ **Completed:**
- Tauri V2 + Svelte project structure
- Complete UI implementation
- File drag-and-drop interface
- Audio processing controls
- YouTube download interface
- Cross-platform build pipeline (GitHub Actions)

🚧 **In Progress:**
- Backend Rust functions (placeholder implementations ready)
- File system integration

📋 **Todo:**
- FFmpeg binary integration
- yt-dlp binary integration  
- Real audio processing implementation
- Error handling and user feedback
- Performance optimization
- Bundle size optimization