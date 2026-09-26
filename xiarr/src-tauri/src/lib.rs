use std::path::PathBuf;
use std::fs;

#[tauri::command]
fn accept_download(url: String, filename: String, show_on_desktop: bool) -> Result<String, String> {
    let downloads_dir = PathBuf::from("/root/Downloads");
    fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;

    let dest = downloads_dir.join(&filename);

    // Pobierz plik przez curl
    let output = std::process::Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap(), &url])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Blad pobierania: {}", err));
    }

    // Jesli "Pokaz na pulpicie" — stworz skrot w ~/.xiee/desktop/
    if show_on_desktop {
        let desktop_dir = PathBuf::from("/root/.xiee/desktop");
        fs::create_dir_all(&desktop_dir).map_err(|e| e.to_string())?;

        // Format: nazwa|komenda_lub_sciezka|ikona
        let icon = guess_icon(&filename);
        let shortcut_name = filename.replace(' ', "_").replace('/', "_");
        let shortcut_path = desktop_dir.join(format!("{}.shortcut", shortcut_name));
        let content = format!("{}|xdg-open {}|{}", filename, dest.display(), icon);
        fs::write(&shortcut_path, content).map_err(|e| e.to_string())?;
    }

    // Powiadomienie systemowe
    std::process::Command::new("xnotify")
        .args(["XIARR", &format!("Pobrano: {}", filename), "info"])
        .spawn()
        .ok();

    Ok(format!("Pobrano: {}", dest.display()))
}

fn guess_icon(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf"                         => "📕",
        "zip" | "tar" | "gz" | "xz"  => "📦",
        "mp3" | "ogg" | "wav"         => "🎵",
        "mp4" | "mkv" | "avi"         => "🎬",
        "jpg" | "jpeg" | "png" | "gif" => "🖼",
        "deb" | "rpm" | "AppImage"    => "⚙",
        _                             => "📄",
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