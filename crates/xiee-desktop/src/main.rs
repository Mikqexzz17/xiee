//! Xiee Desktop - Glowny proces srodowiska graficznego

use anyhow::Result;
use eframe::egui;
use egui::{CentralPanel, Align2};
use xiee_gui::theme;
use chrono::Local;

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

pub struct XieeDesktop {
    launcher_open: bool,
    wallpaper: Option<egui::TextureHandle>,
    taskbar: taskbar::Taskbar,
}

impl XieeDesktop {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_xiee_theme(&cc.egui_ctx);
        let wallpaper = xiee_gui::wallpaper::load_wallpaper(
            &cc.egui_ctx, "/usr/share/xiee/wallpaper.jpg",
        ).ok();
        Self { launcher_open: false, wallpaper, taskbar: taskbar::Taskbar::new() }
    }
}

fn current_time_str() -> String {
    Local::now().format("%H:%M").to_string()
}

impl eframe::App for XieeDesktop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(30, 40, 60)))
            .show(ctx, |ui| {
                if let Some(tex) = &self.wallpaper {
                    ui.image((tex.id(), ui.available_size()));
                } else {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 45, 80));
                    ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Xiee OS",
                        egui::FontId::proportional(48.0),
                        egui::Color32::from_rgba_premultiplied(255, 255, 255, 30));
                }
            });

        if self.launcher_open {
            launcher::show_launcher(ctx, &mut self.launcher_open);
        }

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

        ctx.request_repaint_after(std::time::Duration::from_secs(30));
    }
}