//! Launcher — menu aplikacji Xiee OS

use egui::{Align2, Color32, CornerRadius, Frame, Order, RichText, Stroke, Vec2};
use xiee_gui::theme::XieeColors;

pub fn show_launcher(ctx: &egui::Context, open: &mut bool) {
    // Zamknij launcher jesli kliknieto poza nim
    let launcher_id = egui::Id::new("launcher");

    let resp = Area::new(launcher_id)
        .anchor(Align2::LEFT_BOTTOM, Vec2::new(0.0, -72.0))
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(XieeColors::LAUNCHER_BG)
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::same(16))
                .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(40)))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(280.0, 400.0));

                    ui.label(RichText::new("Xiee OS").color(Color32::WHITE).size(22.0).strong());
                    ui.label(RichText::new("v0.1.0").color(Color32::from_white_alpha(120)).size(12.0));
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let apps = [
                        ("X  XIAC",     "Centrum aplikacji"),
                        ("O  XIARR",    "Przegladarka"),
                        ("Y  xihh key", "Manager klastra"),
                        ("w  WINYY",    "Ustawienia"),
                        ("📁  XFM",     "Menedzer plikow"),
                        (">_ Terminal", "Xiee Shell"),
                    ];

                    for (name, desc) in &apps {
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new(*name).color(Color32::WHITE).size(15.0)).clicked() {
                                *open = false;
                            }
                            ui.label(RichText::new(*desc).color(Color32::from_white_alpha(100)).size(12.0));
                        });
                        ui.add_space(4.0);
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if ui.button(RichText::new("Zamknij system").color(Color32::from_rgb(255, 100, 100)).size(14.0)).clicked() {
                        std::process::exit(0);
                    }
                });
        });

    // Jesli kliknieto gdzies poza launcherem — zamknij go
    let clicked_outside = ctx.input(|i| i.pointer.any_click())
        && !resp.response.hovered();

    if clicked_outside {
        *open = false;
    }
}