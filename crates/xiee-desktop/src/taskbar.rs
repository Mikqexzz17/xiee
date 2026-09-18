//! Pasek zadan Xiee OS

use egui::{Context, CornerRadius, Frame, Stroke, TopBottomPanel};
use xiee_gui::theme::{XieeColors, TASKBAR_HEIGHT, ICON_SIZE};
use xiee_gui::components::TaskbarIcon;

pub enum TaskbarAction {
    None,
    OpenLauncher,
    OpenApp(String),
}

pub struct Taskbar;

impl Taskbar {
    pub fn new() -> Self { Self }

    pub fn show(&mut self, ctx: &Context, clock: &str) -> TaskbarAction {
        let mut action = TaskbarAction::None;

        TopBottomPanel::bottom("taskbar")
            .exact_height(TASKBAR_HEIGHT)
            .frame(Frame {
                fill: XieeColors::TASKBAR_BG,
                inner_margin: egui::Margin::symmetric(8, 8),
                corner_radius: CornerRadius::ZERO,
                stroke: Stroke::new(1.5_f32, XieeColors::ICON_BORDER),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if TaskbarIcon::new("[...]", "[...]", ICON_SIZE).show(ui) {
                        action = TaskbarAction::OpenLauncher;
                    }
                    ui.add_space(12.0);
                    if TaskbarIcon::new("XIAC", "X&~", ICON_SIZE).show(ui) {
                        action = TaskbarAction::OpenApp("xiac".into());
                    }
                    ui.add_space(8.0);
                    if TaskbarIcon::new("XIARR", "O", ICON_SIZE).show(ui) {
                        action = TaskbarAction::OpenApp("xiarr".into());
                    }
                    ui.add_space(8.0);
                    if TaskbarIcon::new("xihh key", "Yk", ICON_SIZE).show(ui) {
                        action = TaskbarAction::OpenApp("xihh-key".into());
                    }
                    ui.add_space(8.0);
                    if TaskbarIcon::new("WINYY", "w", ICON_SIZE).show(ui) {
                        action = TaskbarAction::OpenApp("winyy".into());
                    }
                    // Zegar po prawej stronie
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(clock)
                                .color(XieeColors::TASKBAR_TEXT)
                                .size(15.0)
                                .strong()
                        );
                    });
                });
            });

        action
    }
}