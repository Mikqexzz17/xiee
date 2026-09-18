//! Wspolne komponenty UI Xiee OS

use egui::{Color32, CornerRadius, Sense, Stroke, StrokeKind, Ui, Vec2};
use crate::theme::XieeColors;

pub struct TaskbarIcon<'a> {
    pub label: &'a str,
    pub symbol: &'a str,
    pub size: f32,
}

impl<'a> TaskbarIcon<'a> {
    pub fn new(label: &'a str, symbol: &'a str, size: f32) -> Self {
        Self { label, symbol, size }
    }

    pub fn show(&self, ui: &mut Ui) -> bool {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::splat(self.size),
            Sense::click(),
        );

        let painter = ui.painter();
        let is_hovered = response.hovered();
        let is_clicked = response.clicked();

        let bg_color = if is_hovered {
            XieeColors::ICON_HOVER
        } else {
            Color32::from_rgba_premultiplied(255, 245, 200, 180)
        };

        painter.rect(
            rect.shrink(2.0),
            CornerRadius::same(2),
            bg_color,
            Stroke::new(2.0_f32, XieeColors::ICON_BORDER),
            StrokeKind::Inside,
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            self.symbol,
            egui::FontId::proportional(self.size * 0.38),
            XieeColors::ICON_BORDER,
        );

        if is_hovered {
            let label_pos = rect.center_bottom() + egui::Vec2::new(0.0, 2.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_TOP,
                self.label,
                egui::FontId::proportional(10.0),
                XieeColors::TASKBAR_TEXT,
            );
        }

        is_clicked
    }
}
