use std::path::PathBuf;
use std::fs;

#[tauri::command]
fn accept_download(url: String, filename: String) -> Result<String, String> {
    // Sciezka do Downloads
    let downloads_dir = PathBuf::from("/root/Downloads");
    fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;

    let dest = downloads_dir.join(&filename);

    // Pobierz plik przez curl
    let output = std::process::Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap(), &url])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        // Powiadom systemowo
        std::process::Command::new("xnotify")
            .args(["XIARR", &format!("Pobrano: {}", filename), "info"])
            .spawn()
            .ok();
        Ok(format!("Pobrano do: {}", dest.display()))
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Blad pobierania: {}", err))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![accept_download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}