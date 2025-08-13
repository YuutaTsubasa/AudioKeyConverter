use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub duration: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub semitones: i32, // Positive for up, negative for down
    pub output_format: String, // mp3, wav, etc.
    pub output_path: String,
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
    // This is a placeholder for audio processing
    // In a real implementation, we would use ffmpeg or similar
    
    let input_path = Path::new(&file_path);
    
    if !input_path.exists() {
        return Err("Input file does not exist".to_string());
    }
    
    // For now, return a placeholder message
    // TODO: Implement actual pitch shifting with ffmpeg
    Ok(format!("File would be processed successfully. Shifted by {} semitones to {}.", options.semitones, options.output_path))
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
    
    Ok(AudioFile {
        name: file_name,
        path: file_path,
        size: metadata.len(),
        duration: None, // TODO: Use ffprobe to get duration
    })
}

#[tauri::command]
async fn download_youtube_audio(url: String, output_dir: String) -> Result<String, String> {
    // This is a placeholder for YouTube download functionality
    // In a real implementation, we would use yt-dlp
    
    // For now, just return a placeholder response
    Ok(format!("YouTube download placeholder for URL: {} to directory: {}", url, output_dir))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
                window.on_file_drop(|window, event| {
                    match event {
                        tauri::webview::FileDropEvent::Hovered { paths, position: _ } => {
                            println!("Files hovered: {:?}", paths);
                        }
                        tauri::webview::FileDropEvent::Dropped { paths, position: _ } => {
                            println!("Files dropped: {:?}", paths);
                            // Emit event to frontend
                            let _ = window.emit("files-dropped", paths);
                        }
                        tauri::webview::FileDropEvent::Cancelled => {
                            println!("File drop cancelled");
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
