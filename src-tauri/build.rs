use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Copy platform-specific binaries during build
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    // Determine the source directory based on target platform
    let binaries_dir = match target_os.as_str() {
        "windows" => "binaries/windows",
        "macos" => "binaries/macos", 
        "linux" => "binaries/linux",
        _ => {
            println!("cargo:warning=Unknown target OS: {}", target_os);
            return;
        }
    };
    
    let source_dir = Path::new(binaries_dir);
    if source_dir.exists() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let target_dir = Path::new(&out_dir).join("../../../");
        
        // Copy all files from the platform-specific directory
        if let Ok(entries) = fs::read_dir(source_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().unwrap();
                    let dest = target_dir.join(file_name);
                    
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("cargo:warning=Failed to copy {} to {}: {}", 
                                path.display(), dest.display(), e);
                    } else {
                        println!("cargo:warning=Copied {} to {}", 
                                path.display(), dest.display());
                        
                        // Make the file executable on Unix systems
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(mut perms) = fs::metadata(&dest).map(|m| m.permissions()) {
                                perms.set_mode(0o755);
                                let _ = fs::set_permissions(&dest, perms);
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("cargo:warning=Binaries directory not found: {}", source_dir.display());
    }
    
    tauri_build::build()
}
