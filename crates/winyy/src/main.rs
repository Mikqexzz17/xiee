//! WINYY — Ustawienia systemowe Xiee OS

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, RichText, ScrollArea, Stroke};
use xiee_gui::theme::{XieeColors, apply_xiee_theme};
use xiee_common::config::{XieeConfig, SystemMode};

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WINYY — Ustawienia")
            .with_inner_size([720.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native("winyy", options, Box::new(|cc| Ok(Box::new(WinyyApp::new(cc))))).ok();
}

struct WinyyApp {
    section: Section,
    config: XieeConfig,
    save_msg: Option<(String, std::time::Instant)>,
}

#[derive(PartialEq, Clone, Copy)]
enum Section { Tryb, System, Display, Network, About }

impl WinyyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        Self {
            section: Section::Tryb,
            config: XieeConfig::load(),
            save_msg: None,
        }
    }

    fn save(&mut self) {
        match self.config.save() {
            Ok(_)  => self.save_msg = Some(("✓ Zapisano. Zmiany widoczne po restarcie pulpitu.".into(), std::time::Instant::now())),
            Err(e) => self.save_msg = Some((format!("✕ Blad: {}", e), std::time::Instant::now())),
        }
    }
}

impl eframe::App for WinyyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Wyczysc komunikat po 4s
        if let Some((_, t)) = &self.save_msg {
            if t.elapsed().as_secs() > 4 { self.save_msg = None; }
        }

        egui::SidePanel::left("nav").min_width(190.0).show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading(RichText::new("⚙  WINYY").color(XieeColors::TASKBAR_BG).size(18.0));
            ui.add_space(16.0);
            for (s, label) in [
                (Section::Tryb,    "🔋 Tryb systemu"),
                (Section::System,  "   System"),
                (Section::Display, "   Ekran"),
                (Section::Network, "   Siec"),
                (Section::About,   "   O systemie"),
            ] {
                if ui.selectable_label(self.section == s, label).clicked() {
                    self.section = s;
                }
            }

            // Aktualny tryb — wskaznik na dole panelu
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(8.0);
            let (mode_label, mode_color) = if self.config.is_ultra_lite() {
                ("⚡ Ultra Lite", Color32::from_rgb(100, 200, 255))
            } else {
                ("◉ Normal", Color32::from_rgb(100, 220, 100))
            };
            ui.label(RichText::new(format!("Tryb: {}", mode_label)).color(mode_color).size(12.0));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Komunikat po zapisaniu
            if let Some((msg, _)) = &self.save_msg {
                let color = if msg.starts_with('✓') {
                    Color32::from_rgb(80, 220, 80)
                } else {
                    Color32::from_rgb(255, 100, 100)
                };
                ui.label(RichText::new(msg).color(color).size(13.0));
                ui.add_space(8.0);
            }

            ScrollArea::vertical().show(ui, |ui| {
                match self.section {
                    Section::Tryb => self.show_tryb(ui),
                    Section::System => self.show_system(ui),
                    Section::Display => self.show_display(ui),
                    Section::Network => self.show_network(ui),
                    Section::About => self.show_about(ui),
                }
            });
        });
    }
}

impl WinyyApp {
    fn show_tryb(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Tryb systemu").color(Color32::WHITE).size(18.0));
        ui.add_space(8.0);
        ui.label(RichText::new("Wybierz jak bardzo Xiee OS oszczedza zasoby.")
            .color(Color32::from_white_alpha(160)).size(13.0));
        ui.add_space(16.0);

        // Karta Normal
        let is_normal = self.config.mode == SystemMode::Normal;
        mode_card(ui, "◉  Normal",
            "Pelne funkcje: tapeta, ikony, animacje, powiadomienia, splash screen.",
            is_normal,
            Color32::from_rgb(60, 180, 80),
            || {});
        if ui.add(
            egui::Button::new(RichText::new(if is_normal {"  Aktywny  "} else {"  Przelacz na Normal  "}).size(13.0))
                .fill(if is_normal {Color32::from_rgb(40,140,60)} else {Color32::from_rgb(50,50,65)})
        ).clicked() && !is_normal {
            self.config.apply_normal();
            self.save();
        }

        ui.add_space(16.0);

        // Karta Ultra Lite
        let is_ultra = self.config.is_ultra_lite();
        mode_card(ui, "⚡  Ultra Lite",
            "Absolutne minimum:\n• Brak tapety (czarne tlo)\n• Brak ikon na pulpicie\n• Brak animacji i efektow\n• Brak splash screenu\n• Zegar odswieza co 30s zamiast 5s\n• Plaska grafika (zero zaokraglen)\n\nIdealny na stare komputery i klastry.",
            is_ultra,
            Color32::from_rgb(60, 160, 255),
            || {});
        if ui.add(
            egui::Button::new(RichText::new(if is_ultra {"  Aktywny  "} else {"  Przelacz na Ultra Lite  "}).size(13.0))
                .fill(if is_ultra {Color32::from_rgb(30,100,200)} else {Color32::from_rgb(50,50,65)})
        ).clicked() && !is_ultra {
            self.config.apply_ultra_lite();
            self.save();
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(12.0);
        ui.label(RichText::new("Ustawienia szczegolowe").color(XieeColors::TASKBAR_BG).size(14.0).strong());
        ui.add_space(8.0);

        ui.checkbox(&mut self.config.wallpaper,      "Tapeta na pulpicie");
        ui.checkbox(&mut self.config.desktop_icons,  "Ikony na pulpicie");
        ui.checkbox(&mut self.config.animations,     "Animacje interfejsu");
        ui.checkbox(&mut self.config.notifications,  "Powiadomienia systemowe (xnotify)");
        ui.checkbox(&mut self.config.splash,         "Splash screen przy starcie");
        ui.checkbox(&mut self.config.rounded_ui,     "Zaokraglenia i efekty UI");

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Interwał zegara (s):");
            ui.add(egui::Slider::new(&mut self.config.clock_interval_secs, 1..=60));
        });

        ui.add_space(16.0);
        if ui.add(
            egui::Button::new(RichText::new("Zapisz ustawienia szczegolowe").size(13.0))
                .fill(XieeColors::TASKBAR_BG)
        ).clicked() {
            // Zaktualizuj mode na podstawie obecnych ustawien
            if !self.config.wallpaper && !self.config.desktop_icons
                && !self.config.animations && !self.config.splash {
                self.config.mode = SystemMode::UltraLite;
            } else {
                self.config.mode = SystemMode::Normal;
            }
            self.save();
        }
    }

    fn show_system(&mut self, ui: &mut egui::Ui) {
        ui.heading("System");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label("Nazwa komputera:");
            ui.text_edit_singleline(&mut self.config.hostname);
        });
        ui.add_space(16.0);
        if ui.button("Zapisz").clicked() { self.save(); }
    }

    fn show_display(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ekran");
        ui.add_space(12.0);
        ui.label(RichText::new("Tapeta").strong());
        ui.label(RichText::new("Sciezka: /usr/share/xiee/wallpaper.jpg").color(Color32::GRAY).size(12.0));
        ui.add_space(8.0);
        ui.checkbox(&mut self.config.wallpaper, "Pokazuj tapete");
        ui.add_space(16.0);
        if ui.button("Zapisz").clicked() { self.save(); }
    }

    fn show_network(&mut self, ui: &mut egui::Ui) {
        ui.heading("Siec");
        ui.add_space(12.0);
        // Odczyt interfejsow z /proc/net/dev
        if let Ok(data) = std::fs::read_to_string("/proc/net/dev") {
            for line in data.lines().skip(2) {
                let iface = line.trim().split(':').next().unwrap_or("").trim();
                if !iface.is_empty() && iface != "lo" {
                    ui.group(|ui| {
                        ui.label(RichText::new(iface).strong());
                        ui.label(RichText::new("Aktywny").color(Color32::from_rgb(80,200,80)));
                    });
                    ui.add_space(4.0);
                }
            }
        } else {
            ui.label("Brak danych o sieci.");
        }
    }

    fn show_about(&mut self, ui: &mut egui::Ui) {
        ui.heading("O systemie");
        ui.add_space(12.0);
        egui::Grid::new("about").num_columns(2).spacing([30.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new("System:").strong());   ui.label("Xiee OS"); ui.end_row();
            ui.label(RichText::new("Wersja:").strong());   ui.label(XieeConfig::version()); ui.end_row();
            ui.label(RichText::new("Tryb:").strong());
            let m = if self.config.is_ultra_lite() { "Ultra Lite" } else { "Normal" };
            ui.label(m); ui.end_row();
            ui.label(RichText::new("Jezyk:").strong());    ui.label("Rust"); ui.end_row();
            ui.label(RichText::new("Autorzy:").strong());  ui.label("mykhuu i mikqexzz"); ui.end_row();
            ui.label(RichText::new("Licencja:").strong()); ui.label("MIT"); ui.end_row();
            ui.label(RichText::new("Cel:").strong());      ui.label("Klastry starych urzadzen"); ui.end_row();
        });
    }
}

fn mode_card(ui: &mut egui::Ui, title: &str, desc: &str, active: bool, color: Color32, _f: impl FnOnce()) {
    let bg = if active {
        Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 25)
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, 8)
    };
    let border = if active { color } else { Color32::from_white_alpha(25) };

    Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(10))
        .stroke(Stroke::new(if active {2.0_f32} else {1.0_f32}, border))
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(480.0);
            ui.label(RichText::new(title).color(color).size(16.0).strong());
            ui.add_space(6.0);
            ui.label(RichText::new(desc).color(Color32::from_white_alpha(160)).size(12.0));
        });
    ui.add_space(6.0);
}
