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

/// Wynik klikniecia przyciskow okna
pub enum WindowAction {
    None,
    /// Minimalizacja — okno chowa sie ale PROCES DZIALA DALEJ
    Minimize,
    Close,
}

/// Pasek tytulowy okna Xiee OS
/// Przycisk minimalizacji: kwadrat z kreseczka [—]
/// Minimalizacja NIE wstrzymuje procesu — to celowy design Xiee OS
pub struct XieeWindowBar<'a> {
    pub title: &'a str,
}

impl<'a> XieeWindowBar<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title }
    }

    /// Rysuje pasek tytulowy. Zwraca akcje uzytkownika.
    pub fn show(&self, ctx: &egui::Context, ui: &mut Ui) -> WindowAction {
        let mut action = WindowAction::None;
        let bar_height = 32.0_f32;
        let btn_size = Vec2::new(20.0, 20.0);

        // Tlo paska
        let bar_rect = ui.available_rect_before_wrap();
        let bar_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width(), bar_height));
        ui.painter().rect_filled(bar_rect, CornerRadius::same(0), Color32::from_rgb(28, 28, 38));

        ui.horizontal(|ui| {
            ui.add_space(10.0);

            // Tytul okna
            ui.label(
                egui::RichText::new(self.title)
                    .color(XieeColors::TASKBAR_BG)
                    .size(13.0)
                    .strong()
            );

            // Przyciski po prawej stronie
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);

                // [X] Zamknij
                if Self::window_btn(ui, btn_size, "✕", Color32::from_rgb(200, 60, 60)) {
                    action = WindowAction::Close;
                }
                ui.add_space(4.0);

                // [—] Minimalizuj — kwadrat z kreseczka
                // Proces NIE jest wstrzymywany (to standardowe zachowanie na Linuksie)
                if Self::window_btn(ui, btn_size, "—", Color32::from_rgb(80, 80, 100)) {
                    action = WindowAction::Minimize;
                }
            });
        });

        // Obsluz akcje
        match action {
            WindowAction::Minimize => {
                // Minimalizuj okno — proces nadal dziala w tle
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            WindowAction::Close => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            WindowAction::None => {}
        }

        action
    }

    fn window_btn(ui: &mut Ui, size: Vec2, symbol: &str, hover_color: Color32) -> bool {
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
        let painter = ui.painter();

        let bg = if resp.hovered() { hover_color } else { Color32::from_rgb(50, 50, 65) };
        painter.rect_filled(rect, CornerRadius::same(4), bg);

        // Kwadrat z kreseczka — obrys kwadratu dla [—]
        if symbol == "—" {
            painter.rect(
                rect.shrink(2.0),
                CornerRadius::same(2),
                Color32::TRANSPARENT,
                Stroke::new(1.0_f32, Color32::from_white_alpha(80)),
                StrokeKind::Inside,
            );
        }

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            symbol,
            egui::FontId::proportional(12.0),
            Color32::from_white_alpha(220),
        );
        resp.clicked()
    }
}

