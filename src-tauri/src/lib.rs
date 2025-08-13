use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub duration: Option<f64>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub semitones: i32, // Positive for up, negative for down
    pub output_format: String, // mp3, wav, etc.
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingProgress {
    pub percentage: f32,
    pub status: String,
    pub current_file: Option<String>,
}

// Get the path to bundled FFmpeg binary
fn get_ffmpeg_path() -> Result<PathBuf, String> {
    // In a real implementation, this would point to the bundled FFmpeg binary
    // For now, we'll try to find it in the system PATH
    if cfg!(target_os = "windows") {
        Ok(PathBuf::from("ffmpeg.exe"))
    } else {
        Ok(PathBuf::from("ffmpeg"))
    }
}

// Get the path to bundled yt-dlp binary
fn get_ytdlp_path() -> Result<PathBuf, String> {
    // In a real implementation, this would point to the bundled yt-dlp binary
    // For now, we'll try to find it in the system PATH
    if cfg!(target_os = "windows") {
        Ok(PathBuf::from("yt-dlp.exe"))
    } else {
        Ok(PathBuf::from("yt-dlp"))
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn process_audio_file(
    file_path: String,
    options: ConversionOptions,
) -> Result<String, String> {
    let input_path = Path::new(&file_path);
    
    if !input_path.exists() {
        return Err("Input file does not exist".to_string());
    }
    
    // Validate file extension
    let ext = input_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    if !["mp3", "wav", "flac", "m4a", "aac", "ogg"].contains(&ext.as_str()) {
        return Err("Unsupported audio format".to_string());
    }
    
    // For demonstration, we'll simulate the FFmpeg processing
    // In a real implementation, this would call FFmpeg with proper pitch shifting
    let pitch_shift_desc = if options.semitones > 0 {
        format!("raised by {} semitones", options.semitones)
    } else if options.semitones < 0 {
        format!("lowered by {} semitones", options.semitones.abs())
    } else {
        "unchanged (0 semitones)".to_string()
    };
    
    // Simulate processing time
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // TODO: Implement actual FFmpeg processing
    /*
    let ffmpeg_path = get_ffmpeg_path()?;
    let pitch_factor = 2.0_f64.powf(options.semitones as f64 / 12.0);
    
    let output = Command::new(ffmpeg_path)
        .args([
            "-i", &file_path,
            "-af", &format!("asetrate=44100*{},aresample=44100", pitch_factor),
            "-y", &options.output_path
        ])
        .output()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr)));
    }
    */
    
    Ok(format!(
        "Audio processing completed!\nFile: {}\nPitch: {}\nOutput format: {}\nOutput: {}",
        input_path.file_name().unwrap_or_default().to_string_lossy(),
        pitch_shift_desc,
        options.output_format.to_uppercase(),
        options.output_path
    ))
}

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
    
    let format = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_uppercase());
    
    // TODO: Use FFprobe to get actual duration
    // For now, estimate based on file size (very rough estimate)
    let estimated_duration = if metadata.len() > 0 {
        Some((metadata.len() as f64) / 128000.0) // Assume 128kbps average
    } else {
        None
    };
    
    Ok(AudioFile {
        name: file_name,
        path: file_path,
        size: metadata.len(),
        duration: estimated_duration,
        format,
    })
}

#[tauri::command]
async fn download_youtube_audio(
    url: String, 
    output_dir: String
) -> Result<String, String> {
    // Validate YouTube URL
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return Err("Invalid YouTube URL".to_string());
    }
    
    // Simulate download process
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // TODO: Implement actual yt-dlp integration
    /*
    let ytdlp_path = get_ytdlp_path()?;
    let output = Command::new(ytdlp_path)
        .args([
            "-x",
            "--audio-format", "mp3",
            "--audio-quality", "0",
            "-o", &format!("{}/%(title)s.%(ext)s", output_dir),
            &url
        ])
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("yt-dlp error: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    // Parse output to get downloaded file path
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Extract filename from yt-dlp output
    */
    
    Ok(format!(
        "YouTube download simulation completed!\nURL: {}\nOutput directory: {}\n\nNote: This is a demonstration. In the full implementation, yt-dlp would download the audio file automatically.",
        url, output_dir
    ))
}

#[tauri::command]
async fn get_system_info() -> Result<serde_json::Value, String> {
    let info = serde_json::json!({
        "platform": env::consts::OS,
        "arch": env::consts::ARCH,
        "ffmpeg_available": check_ffmpeg_available(),
        "ytdlp_available": check_ytdlp_available(),
    });
    
    Ok(info)
}

fn check_ffmpeg_available() -> bool {
    // Check if FFmpeg is available
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_ytdlp_available() -> bool {
    // Check if yt-dlp is available
    Command::new("yt-dlp")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            process_audio_file,
            get_audio_info,
            download_youtube_audio,
            get_system_info
        ])
        .setup(|app| {
            // Set up file drop handling
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                window.on_file_drop(|window, event| {
                    match event {
                        tauri::webview::FileDropEvent::Hovered { paths, position: _ } => {
                            println!("Files hovered: {:?}", paths);
                            let _ = window.emit("files-hovered", paths);
                        }
                        tauri::webview::FileDropEvent::Dropped { paths, position: _ } => {
                            println!("Files dropped: {:?}", paths);
                            // Filter for audio files only
                            let audio_files: Vec<String> = paths
                                .into_iter()
                                .filter(|path| {
                                    let ext = Path::new(path)
                                        .extension()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    ["mp3", "wav", "flac", "m4a", "aac", "ogg"].contains(&ext.as_str())
                                })
                                .collect();
                            
                            let _ = window.emit("files-dropped", audio_files);
                        }
                        tauri::webview::FileDropEvent::Cancelled => {
                            println!("File drop cancelled");
                            let _ = window.emit("files-drop-cancelled", ());
                        }
                        _ => {}
                    }
                    true
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
