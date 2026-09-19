//! xnotify - Systemowe powiadomienia Xiee OS
//! Uzycie: xnotify "Tytul" "Tresc" [info|warn|error]
//! Okienko pojawia sie w prawym dolnym rogu i znika po 4 sekundach.

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, RichText, Stroke, Vec2};
use std::time::{Duration, Instant};
use xiee_gui::theme::XieeColors;

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let title   = args.get(1).cloned().unwrap_or_else(|| "Xiee OS".into());
    let message = args.get(2).cloned().unwrap_or_else(|| "Powiadomienie".into());
    let kind    = args.get(3).cloned().unwrap_or_else(|| "info".into());

    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 80.0])
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true)
            .with_position(egui::Pos2::new(1580.0, 960.0)),
        ..Default::default()
    };

    eframe::run_native(
        "xnotify",
        opts,
        Box::new(move |_cc| Ok(Box::new(NotifyApp::new(title, message, kind)))),
    ).ok();
}

struct NotifyApp {
    title: String,
    message: String,
    accent: Color32,
    icon: &'static str,
    started: Instant,
    lifetime: Duration,
}

impl NotifyApp {
    fn new(title: String, message: String, kind: String) -> Self {
        let (accent, icon) = match kind.as_str() {
            "warn"  => (Color32::from_rgb(220, 170, 40), "⚠"),
            "error" => (Color32::from_rgb(210, 60,  60), "✕"),
            _       => (XieeColors::TASKBAR_BG,           "ℹ"),
        };
        Self {
            title,
            message,
            accent,
            icon,
            started: Instant::now(),
            lifetime: Duration::from_secs(4),
        }
    }
}

impl eframe::App for NotifyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.started.elapsed() >= self.lifetime {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let elapsed = self.started.elapsed().as_secs_f32();
        let total   = self.lifetime.as_secs_f32();
        let alpha   = if elapsed < 0.3 {
            elapsed / 0.3
        } else if elapsed > total - 0.5 {
            (total - elapsed) / 0.5
        } else {
            1.0
        }.clamp(0.0, 1.0);

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                Frame::NONE
                    .fill(Color32::from_rgba_premultiplied(20, 20, 20, (220.0 * alpha) as u8))
                    .corner_radius(CornerRadius::same(10))
                    .stroke(Stroke::new(1.5_f32, self.accent))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(300.0, 60.0));
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(self.icon)
                                    .color(self.accent)
                                    .size(22.0)
                            );
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&self.title)
                                        .color(Color32::WHITE)
                                        .size(13.0)
                                        .strong()
                                );
                                ui.label(
                                    RichText::new(&self.message)
                                        .color(Color32::from_white_alpha(180))
                                        .size(12.0)
                                );
                            });
                        });
                    });
            });

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}