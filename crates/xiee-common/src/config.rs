//! Konfiguracja systemu Xiee OS
//! Plik: ~/.xiee/config.json

use std::fs;
use std::path::PathBuf;

/// Tryb pracy systemu
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemMode {
    /// Normalny tryb — pelne funkcje
    Normal,
    /// Ultra Lite — absolutne minimum CPU/RAM
    UltraLite,
}

impl Default for SystemMode {
    fn default() -> Self { SystemMode::Normal }
}

/// Glowna konfiguracja Xiee OS
/// Zapisywana w ~/.xiee/config.json
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct XieeConfig {
    /// Tryb systemu
    #[serde(default)]
    pub mode: SystemMode,

    /// Pokazuj tapete na pulpicie (wylaczone w Ultra Lite)
    #[serde(default = "default_true")]
    pub wallpaper: bool,

    /// Pokazuj ikony na pulpicie (wylaczone w Ultra Lite)
    #[serde(default = "default_true")]
    pub desktop_icons: bool,

    /// Animacje UI (wylaczone w Ultra Lite)
    #[serde(default = "default_true")]
    pub animations: bool,

    /// Powiadomienia systemowe xnotify (wylaczone w Ultra Lite)
    #[serde(default = "default_true")]
    pub notifications: bool,

    /// Pokazuj splash screen przy starcie (wylaczone w Ultra Lite)
    #[serde(default = "default_true")]
    pub splash: bool,

    /// Czas trwania splash screenu w sekundach
    #[serde(default = "default_splash_secs")]
    pub splash_secs: u64,

    /// Jak czesto odswiezac zegar na pasku (sekundy)
    #[serde(default = "default_clock_interval")]
    pub clock_interval_secs: u64,

    /// Zaokraglenia UI (wylaczone w Ultra Lite = płaski wygląd)
    #[serde(default = "default_true")]
    pub rounded_ui: bool,

    /// Nazwa hosta systemu
    #[serde(default = "default_hostname")]
    pub hostname: String,
}

fn default_true() -> bool { true }
fn default_splash_secs() -> u64 { 3 }
fn default_clock_interval() -> u64 { 5 }
fn default_hostname() -> String { "xiee".to_string() }

impl Default for XieeConfig {
    fn default() -> Self {
        Self {
            mode: SystemMode::Normal,
            wallpaper: true,
            desktop_icons: true,
            animations: true,
            notifications: true,
            splash: true,
            splash_secs: 3,
            clock_interval_secs: 5,
            rounded_ui: true,
            hostname: "xiee".to_string(),
        }
    }
}

impl XieeConfig {
    /// Sciezka do pliku konfiguracyjnego
    pub fn config_path() -> PathBuf {
        PathBuf::from("/root/.xiee/config.json")
    }

    /// Wczytaj konfiguracje z pliku (lub zwroc domyslna)
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<XieeConfig>(&data) {
                return cfg;
            }
        }
        Self::default()
    }

    /// Zapisz konfiguracje do pliku
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Zastosuj preset Ultra Lite — absolutne minimum
    pub fn apply_ultra_lite(&mut self) {
        self.mode = SystemMode::UltraLite;
        self.wallpaper = false;
        self.desktop_icons = false;
        self.animations = false;
        self.notifications = false;
        self.splash = false;
        self.splash_secs = 0;
        self.clock_interval_secs = 30;
        self.rounded_ui = false;
    }

    /// Przywroc preset Normal
    pub fn apply_normal(&mut self) {
        self.mode = SystemMode::Normal;
        self.wallpaper = true;
        self.desktop_icons = true;
        self.animations = true;
        self.notifications = true;
        self.splash = true;
        self.splash_secs = 3;
        self.clock_interval_secs = 5;
        self.rounded_ui = true;
    }

    /// Czy aktualnie Ultra Lite?
    pub fn is_ultra_lite(&self) -> bool {
        self.mode == SystemMode::UltraLite
    }

    /// Wersja systemu
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
