//! Xiee Desktop - Glowny proces srodowiska graficznego

use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Align2, Color32, Vec2, Pos2};
use xiee_gui::theme;
use chrono::Local;
use std::{fs, path::PathBuf};

mod launcher;
mod taskbar;

fn main() -> Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_title("Xiee Desktop"),
        ..Default::default()
    };
    eframe::run_native(
        "Xiee Desktop",
        options,
        Box::new(|cc| Ok(Box::new(XieeDesktop::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Skrot na pulpicie
#[derive(Clone)]
struct DesktopIcon {
    name: String,
    command: String,
    icon: String,
}

impl DesktopIcon {
    fn load_all() -> Vec<Self> {
        let dir = PathBuf::from("/root/.xiee/desktop");
        let mut icons = Vec::new();

        // Wbudowane stale skroty
        icons.push(DesktopIcon {
            name: "XFM".into(),
            command: "xfm".into(),
            icon: "📁".into(),
        });

        // Skroty z katalogu ~/.xiee/desktop/*.shortcut
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "shortcut").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let parts: Vec<&str> = content.trim().splitn(3, '|').collect();
                        if parts.len() == 3 {
                            icons.push(DesktopIcon {
                                name: parts[0].to_string(),
                                command: parts[1].to_string(),
                                icon: parts[2].to_string(),
                            });
                        }
                    }
                }
            }
        }

        icons
    }

    fn launch(&self) {
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", &self.command]);
        cmd.spawn().ok();
    }
}

pub struct XieeDesktop {
    launcher_open: bool,
    wallpaper: Option<egui::TextureHandle>,
    taskbar: taskbar::Taskbar,
    desktop_icons: Vec<DesktopIcon>,
    selected_icon: Option<usize>,
    last_icon_reload: std::time::Instant,
}

impl XieeDesktop {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_xiee_theme(&cc.egui_ctx);
        let wallpaper = xiee_gui::wallpaper::load_wallpaper(
            &cc.egui_ctx, "/usr/share/xiee/wallpaper.jpg",
        ).ok();
        // Upewnij sie ze katalog istnieje
        fs::create_dir_all("/root/.xiee/desktop").ok();
        Self {
            launcher_open: false,
            wallpaper,
            taskbar: taskbar::Taskbar::new(),
            desktop_icons: DesktopIcon::load_all(),
            selected_icon: None,
            last_icon_reload: std::time::Instant::now(),
        }
    }
}

fn current_time_str() -> String {
    Local::now().format("%H:%M").to_string()
}

impl eframe::App for XieeDesktop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Odswiez ikony co 5 sekund (zeby widac bylo nowe pobrane pliki)
        if self.last_icon_reload.elapsed().as_secs() >= 5 {
            self.desktop_icons = DesktopIcon::load_all();
            self.last_icon_reload = std::time::Instant::now();
        }

        // Tapeta
        CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(30, 40, 60)))
            .show(ctx, |ui| {
                if let Some(tex) = &self.wallpaper {
                    ui.image((tex.id(), ui.available_size()));
                } else {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(30, 45, 80));
                    ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Xiee OS",
                        egui::FontId::proportional(48.0),
                        Color32::from_rgba_premultiplied(255, 255, 255, 30));
                }

                // Ikony na pulpicie — lewa gorna czesc
                let icons = self.desktop_icons.clone();
                for (i, icon) in icons.iter().enumerate() {
                    let col = (i % 2) as f32;
                    let row = (i / 2) as f32;
                    let x = 24.0 + col * 100.0;
                    let y = 24.0 + row * 100.0;
                    let icon_rect = egui::Rect::from_min_size(
                        Pos2::new(x, y),
                        Vec2::new(80.0, 80.0),
                    );

                    let is_selected = self.selected_icon == Some(i);
                    let resp = ui.allocate_rect(icon_rect, egui::Sense::click());

                    // Tlo ikony gdy zaznaczona
                    if is_selected || resp.hovered() {
                        ui.painter().rect_filled(
                            icon_rect,
                            egui::CornerRadius::same(8),
                            Color32::from_rgba_premultiplied(212, 168, 67, 50),
                        );
                    }

                    // Emoji ikony
                    ui.painter().text(
                        Pos2::new(x + 40.0, y + 30.0),
                        Align2::CENTER_CENTER,
                        &icon.icon,
                        egui::FontId::proportional(28.0),
                        Color32::WHITE,
                    );

                    // Nazwa ikony
                    ui.painter().text(
                        Pos2::new(x + 40.0, y + 62.0),
                        Align2::CENTER_CENTER,
                        &icon.name,
                        egui::FontId::proportional(11.0),
                        Color32::WHITE,
                    );

                    if resp.clicked() {
                        self.selected_icon = Some(i);
                    }
                    if resp.double_clicked() {
                        icon.launch();
                        self.selected_icon = None;
                    }
                }
            });

        // Klik w puste miejsce = odznacz ikone
        if ctx.input(|i| i.pointer.any_click()) {
            if !ctx.is_pointer_over_area() {
                self.selected_icon = None;
            }
        }

        // Launcher
        if self.launcher_open {
            launcher::show_launcher(ctx, &mut self.launcher_open);
        }

        // Pasek zadan
        let clock = current_time_str();
        match self.taskbar.show(ctx, &clock) {
            taskbar::TaskbarAction::OpenLauncher => {
                self.launcher_open = !self.launcher_open;
            }
            taskbar::TaskbarAction::OpenApp(app) => {
                let cmd = match app.as_str() {
                    "xiac"     => "xiac",
                    "xiarr"    => "xiarr",
                    "xihh-key" => "xihh-key",
                    "winyy"    => "winyy",
                    "xfm"      => "xfm",
                    _          => return,
                };
                std::process::Command::new(cmd).spawn().ok();
            }
            taskbar::TaskbarAction::None => {}
        }

        ctx.request_repaint_after(std::time::Duration::from_secs(5));
    }
}