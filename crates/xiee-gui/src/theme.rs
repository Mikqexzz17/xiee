//! Motyw kolorystyczny Xiee OS

use egui::{Color32, Shadow, Style, Visuals};

/// Glowne kolory Xiee OS
pub struct XieeColors;

impl XieeColors {
    pub const TASKBAR_BG: Color32 = Color32::from_rgb(212, 168, 67);
    pub const ICON_BORDER: Color32 = Color32::from_rgb(40, 30, 10);
    pub const TASKBAR_TEXT: Color32 = Color32::from_rgb(20, 15, 5);
    pub const LAUNCHER_BG: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 200);
    pub const ICON_HOVER: Color32 = Color32::from_rgb(235, 195, 100);
    pub const APP_TEXT: Color32 = Color32::from_rgb(230, 220, 200);
}

pub const TASKBAR_HEIGHT: f32 = 72.0;
pub const ICON_SIZE: f32 = 56.0;

pub fn apply_xiee_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    style.visuals = Visuals {
        dark_mode: true,
        window_shadow: Shadow::NONE,
        ..Visuals::dark()
    };
    ctx.set_style(style);
}
