# Next Steps for Complete Implementation

This document outlines the remaining steps to complete the Audio Key Converter application with fully functional ffmpeg and yt-dlp integration.

## Current Status ✅

The application currently has:
- ✅ Complete Tauri V2 + Svelte frontend with modern UI
- ✅ Rust backend structure with all necessary functions
- ✅ File drag-and-drop support
- ✅ Audio key adjustment controls (-12 to +12 semitones)
- ✅ YouTube URL input and validation
- ✅ Cross-platform GitHub Actions build pipeline
- ✅ Comprehensive error handling and user feedback
- ✅ Responsive design for all screen sizes

## Required Implementation Steps 🚧

### 1. Bundle FFmpeg and yt-dlp Binaries

**Download platform-specific binaries:**
```bash
# Create binaries directory
mkdir -p src-tauri/binaries/{windows,linux,macos}

# Download FFmpeg for each platform
# Windows: https://www.gyan.dev/ffmpeg/builds/
# Linux: https://johnvansickle.com/ffmpeg/
# macOS: https://evermeet.cx/ffmpeg/

# Download yt-dlp for each platform  
# All platforms: https://github.com/yt-dlp/yt-dlp/releases
```

**Update Tauri configuration:**
```json
{
  "bundle": {
    "resources": [
      "binaries/windows/*",
      "binaries/linux/*", 
      "binaries/macos/*"
    ]
  }
}
```

### 2. Implement Actual Audio Processing

**Update the `process_audio_file` function in `src-tauri/src/lib.rs`:**

```rust
#[tauri::command]
async fn process_audio_file(
    file_path: String,
    options: ConversionOptions,
) -> Result<String, String> {
    let input_path = Path::new(&file_path);
    
    if !input_path.exists() {
        return Err("Input file does not exist".to_string());
    }
    
    let ffmpeg_path = get_bundled_ffmpeg_path()?;
    let pitch_factor = 2.0_f64.powf(options.semitones as f64 / 12.0);
    
    let output = Command::new(ffmpeg_path)
        .args([
            "-i", &file_path,
            "-af", &format!("asetrate=44100*{},aresample=44100", pitch_factor),
            "-f", &options.output_format,
            "-y", &options.output_path
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(format!("Successfully processed {} with {} semitones shift", 
               input_path.file_name().unwrap_or_default().to_string_lossy(),
               options.semitones))
}

fn get_bundled_ffmpeg_path() -> Result<PathBuf, String> {
    let mut exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable directory: {}", e))?;
    exe_dir.pop(); // Remove executable name
    
    #[cfg(target_os = "windows")]
    let ffmpeg_name = "ffmpeg.exe";
    #[cfg(not(target_os = "windows"))]
    let ffmpeg_name = "ffmpeg";
    
    let ffmpeg_path = exe_dir.join(ffmpeg_name);
    
    if !ffmpeg_path.exists() {
        return Err("FFmpeg binary not found in application directory".to_string());
    }
    
    Ok(ffmpeg_path)
}
```

### 3. Implement YouTube Download

**Update the `download_youtube_audio` function:**

```rust
#[tauri::command]
async fn download_youtube_audio(
    url: String,
    output_dir: String,
) -> Result<String, String> {
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return Err("Invalid YouTube URL".to_string());
    }
    
    let ytdlp_path = get_bundled_ytdlp_path()?;
    let output_template = format!("{}/%(title)s.%(ext)s", output_dir);
    
    let output = Command::new(ytdlp_path)
        .args([
            "-x",
            "--audio-format", "mp3",
            "--audio-quality", "0",
            "-o", &output_template,
            &url
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("yt-dlp error: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Parse output to get downloaded file path
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    Ok(format!("Successfully downloaded: {}", url))
}

fn get_bundled_ytdlp_path() -> Result<PathBuf, String> {
    let mut exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable directory: {}", e))?;
    exe_dir.pop();
    
    #[cfg(target_os = "windows")]
    let ytdlp_name = "yt-dlp.exe";
    #[cfg(not(target_os = "windows"))]
    let ytdlp_name = "yt-dlp";
    
    let ytdlp_path = exe_dir.join(ytdlp_name);
    
    if !ytdlp_path.exists() {
        return Err("yt-dlp binary not found in application directory".to_string());
    }
    
    Ok(ytdlp_path)
}
```

### 4. Add Audio Duration Detection

**Implement FFprobe integration:**

```rust
#[tauri::command]
async fn get_audio_info(file_path: String) -> Result<AudioFile, String> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    
    let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Get duration using FFprobe
    let duration = get_audio_duration(&file_path).await.ok();
    
    let format = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_uppercase());
    
    Ok(AudioFile {
        name: file_name,
        path: file_path,
        size: metadata.len(),
        duration,
        format,
    })
}

async fn get_audio_duration(file_path: &str) -> Result<f64, String> {
    let ffprobe_path = get_bundled_ffprobe_path()?;
    
    let output = Command::new(ffprobe_path)
        .args([
            "-v", "quiet",
            "-show_entries", "format=duration",
            "-of", "csv=p=0",
            file_path
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute FFprobe: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to get audio duration".to_string());
    }
    
    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str.trim().parse::<f64>()
        .map_err(|e| format!("Failed to parse duration: {}", e))
}
```

### 5. Update Cargo Dependencies

**Add required dependencies to `src-tauri/Cargo.toml`:**

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
```

### 6. Testing and Optimization

**Create test files:**
```bash
# Create test audio files for development
mkdir -p test-files
# Add various format test files (mp3, wav, flac, etc.)
```

**Bundle size optimization:**
- Use UPX compression for binaries
- Strip debug symbols in release builds
- Consider platform-specific releases to reduce download size

### 7. Final Build Configuration

**Update `.github/workflows/build.yml` to include binary copying:**

```yaml
      - name: Copy binaries to resources
        run: |
          mkdir -p src-tauri/binaries
          # Copy platform-specific binaries
          # This would need to be customized based on binary sources
```

## Expected Bundle Sizes

- **Windows**: ~45MB (FFmpeg ~40MB + yt-dlp ~15MB + app ~5MB)
- **Linux**: ~35MB (FFmpeg ~30MB + yt-dlp ~10MB + app ~5MB)  
- **macOS**: ~50MB (FFmpeg ~45MB + yt-dlp ~15MB + app ~5MB)

## License Compliance

- ✅ **FFmpeg**: LGPL v2.1+ (can be distributed with applications)
- ✅ **yt-dlp**: Unlicense (public domain, no restrictions)
- ✅ **Application**: MIT License (compatible with all dependencies)

## Deployment Ready Checklist

- [ ] Download and verify FFmpeg binaries for all platforms
- [ ] Download and verify yt-dlp binaries for all platforms
- [ ] Implement actual audio processing functions
- [ ] Implement actual YouTube download functions
- [ ] Add progress reporting for long operations
- [ ] Test with various audio formats and YouTube URLs
- [ ] Optimize binary sizes and bundle compression
- [ ] Update GitHub Actions to include binary bundling
- [ ] Create installer packages for easy distribution
- [ ] Add comprehensive error handling and user guidance

Once these steps are completed, the Audio Key Converter will be a fully functional desktop application ready for distribution across Windows, Linux, and macOS platforms.