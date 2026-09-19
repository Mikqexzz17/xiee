//! xiee-splash — Ekran powitalny Xiee OS
//! Wyswietla logo przez 3 sekundy, potem uruchamia xiee-login i konczy.

use eframe::egui;
use egui::{Align2, Color32, FontId, Frame, Vec2};
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_title("Xiee OS"),
        ..Default::default()
    };
    eframe::run_native(
        "xiee-splash",
        opts,
        Box::new(|cc| Ok(Box::new(SplashApp::new(cc)))),
    ).ok();
}

struct SplashApp {
    logo: Option<egui::TextureHandle>,
    started: Instant,
    launched: bool,
}

impl SplashApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let logo = xiee_gui::wallpaper::load_wallpaper(&cc.egui_ctx, "/usr/share/xiee/logo.jpg").ok();
        Self { logo, started: Instant::now(), launched: false }
    }
}

impl eframe::App for SplashApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let elapsed = self.started.elapsed().as_secs_f32();
        let total = 3.0_f32;

        // Po 3 sekundach uruchom login i zamknij splash
        if elapsed >= total && !self.launched {
            self.launched = true;
            std::process::Command::new("xiee-login").spawn().ok();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Animacja alpha — fade in przez 0.8s, fade out przez ostatnie 0.8s
        let alpha = if elapsed < 0.8 {
            elapsed / 0.8
        } else if elapsed > total - 0.8 {
            (total - elapsed) / 0.8
        } else {
            1.0
        }.clamp(0.0, 1.0);

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::BLACK))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let size = ui.available_size();
                    ui.add_space(size.y * 0.25);

                    // Logo
                    if let Some(tex) = &self.logo {
                        let logo_size = Vec2::splat(160.0);
                        ui.add(
                            egui::Image::new((tex.id(), logo_size))
                                .corner_radius(egui::CornerRadius::same(16))
                                .tint(Color32::from_white_alpha((255.0 * alpha) as u8))
                        );
                    }

                    ui.add_space(24.0);

                    // Nazwa systemu
                    ui.painter().text(
                        ui.next_widget_position() + Vec2::new(0.0, 10.0),
                        Align2::CENTER_TOP,
                        "XIEE OS",
                        FontId::proportional(36.0),
                        Color32::from_rgba_premultiplied(
                            212, 168, 67, (255.0 * alpha) as u8
                        ),
                    );

                    ui.add_space(60.0);

                    // Pasek postepu
                    let progress = (elapsed / total).clamp(0.0, 1.0);
                    let bar_w = 200.0;
                    let bar_h = 3.0;
                    let center = ui.next_widget_position();
                    let bar_rect = egui::Rect::from_center_size(
                        center + Vec2::new(0.0, 20.0),
                        Vec2::new(bar_w, bar_h),
                    );
                    // Tlo paska
                    ui.painter().rect_filled(
                        bar_rect,
                        egui::CornerRadius::same(2),
                        Color32::from_white_alpha(30),
                    );
                    // Wypelnienie paska
                    let filled = egui::Rect::from_min_size(
                        bar_rect.min,
                        Vec2::new(bar_w * progress, bar_h),
                    );
                    ui.painter().rect_filled(
                        filled,
                        egui::CornerRadius::same(2),
                        Color32::from_rgba_premultiplied(212, 168, 67, (200.0 * alpha) as u8),
                    );
                });
            });

        ctx.request_repaint_after(Duration::from_millis(33)); // ~30 fps
    }
}