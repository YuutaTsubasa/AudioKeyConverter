use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command; // Use tokio::process::Command for async operations
use tauri::{Listener, Emitter}; // Import Listener trait for listening to events

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

fn get_bundled_ffprobe_path() -> Result<PathBuf, String> {
    let mut exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable directory: {}", e))?;
    exe_dir.pop(); // Remove executable name
    
    #[cfg(target_os = "windows")]
    let ffprobe_name = "ffprobe.exe";
    #[cfg(not(target_os = "windows"))]
    let ffprobe_name = "ffprobe";
    
    let ffprobe_path = exe_dir.join(ffprobe_name);
    
    if !ffprobe_path.exists() {
        return Err("FFprobe binary not found in application directory".to_string());
    }
    
    Ok(ffprobe_path)
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

#[tauri::command]
async fn download_youtube_audio(
    url: String,
    output_dir: String,
) -> Result<serde_json::Value, String> {
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
            "--print", "after_move:filepath",
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
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    
    // The last line should contain the file path
    if let Some(file_path) = lines.last() {
        let file_path = file_path.trim();
        if !file_path.is_empty() && std::path::Path::new(file_path).exists() {
            // Get file info for the downloaded file
            match get_audio_info(file_path.to_string()).await {
                Ok(file_info) => {
                    return Ok(serde_json::json!({
                        "success": true,
                        "message": format!("Successfully downloaded: {}", url),
                        "file": file_info
                    }));
                }
                Err(_) => {
                    return Ok(serde_json::json!({
                        "success": true,
                        "message": format!("Successfully downloaded: {}", url),
                        "file": null
                    }));
                }
            }
        }
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Successfully downloaded: {}", url),
        "file": null
    }))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            process_audio_file,
            get_audio_info,
            download_youtube_audio
        ])
        .setup(|app| {
            // Set up file drop handling
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                
                // In Tauri v2, file drop events are handled differently through window events
                let window_clone = window.clone();
                window.listen("tauri://file-drop", move |event| {
                    if let Ok(paths) = serde_json::from_str::<Vec<String>>(event.payload()) {
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
                        
                        let _ = window_clone.emit("files-dropped", audio_files);
                    }
                });

                let window_clone = window.clone();
                window.listen("tauri://file-drop-hover", move |event| {
                    if let Ok(paths) = serde_json::from_str::<Vec<String>>(event.payload()) {
                        println!("Files hovered: {:?}", paths);
                        let _ = window_clone.emit("files-hovered", paths);
                    }
                });

                let window_clone = window.clone();
                window.listen("tauri://file-drop-cancelled", move |_event| {
                    println!("File drop cancelled");
                    let _ = window_clone.emit("files-drop-cancelled", ());
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
