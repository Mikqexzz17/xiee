//! WINYY — Ustawienia systemowe Xiee OS

use eframe::egui;
use egui::{Color32, RichText, ScrollArea};
use xiee_gui::theme::{XieeColors, apply_xiee_theme};

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WINYY — Ustawienia")
            .with_inner_size([680.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native("winyy", options, Box::new(|cc| Ok(Box::new(WinyyApp::new(cc))))).ok();
}

struct WinyyApp {
    section: Section,
    hostname: String,
    wallpaper_path: String,
    volume: f32,
    brightness: f32,
    dark_mode: bool,
    resolution: usize,
}

#[derive(PartialEq, Clone, Copy)]
enum Section { System, Display, Network, Users, About }

impl WinyyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        Self {
            section: Section::System,
            hostname: String::from("xiee-pc"),
            wallpaper_path: String::from("/usr/share/xiee/wallpaper.jpg"),
            volume: 0.7,
            brightness: 1.0,
            dark_mode: true,
            resolution: 0,
        }
    }
}

impl eframe::App for WinyyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("nav").min_width(180.0).show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading(RichText::new("WINYY").color(XieeColors::TASKBAR_BG).size(18.0));
            ui.add_space(16.0);
            if ui.selectable_label(self.section == Section::System,    "System").clicked()      { self.section = Section::System; }
            if ui.selectable_label(self.section == Section::Display,   "Ekran").clicked()       { self.section = Section::Display; }
            if ui.selectable_label(self.section == Section::Network,   "Siec").clicked()        { self.section = Section::Network; }
            if ui.selectable_label(self.section == Section::Users,     "Uzytkownicy").clicked() { self.section = Section::Users; }
            if ui.selectable_label(self.section == Section::About,     "O systemie").clicked()  { self.section = Section::About; }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                match self.section {
                    Section::System => {
                        ui.heading("System");
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.label("Nazwa komputera:");
                            ui.text_edit_singleline(&mut self.hostname);
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Glosnosc:");
                            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0));
                        });
                        ui.add_space(4.0);
                        ui.checkbox(&mut self.dark_mode, "Tryb ciemny");
                        ui.add_space(16.0);
                        if ui.button("Zapisz").clicked() {}
                    }
                    Section::Display => {
                        ui.heading("Ekran");
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.label("Jasnosc:");
                            ui.add(egui::Slider::new(&mut self.brightness, 0.0..=1.0));
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Tapeta:");
                            ui.text_edit_singleline(&mut self.wallpaper_path);
                        });
                        ui.add_space(8.0);
                        ui.label("Rozdzielczosc:");
                        let resolutions = ["1920x1080", "1280x720", "1024x768", "800x600"];
                        egui::ComboBox::from_id_salt("res")
                            .selected_text(resolutions[self.resolution])
                            .show_ui(ui, |ui| {
                                for (i, r) in resolutions.iter().enumerate() {
                                    ui.selectable_value(&mut self.resolution, i, *r);
                                }
                            });
                    }
                    Section::Network => {
                        ui.heading("Siec");
                        ui.add_space(12.0);
                        ui.label("Interfejsy sieciowe:");
                        ui.group(|ui| {
                            ui.label(RichText::new("eth0").strong());
                            ui.label(RichText::new("Polaczony — 192.168.1.100").color(Color32::from_rgb(80,200,80)));
                        });
                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.label(RichText::new("wlan0").strong());
                            ui.label(RichText::new("Rozlaczony").color(Color32::from_rgb(200,80,80)));
                        });
                    }
                    Section::Users => {
                        ui.heading("Uzytkownicy");
                        ui.add_space(12.0);
                        ui.group(|ui| {
                            ui.label(RichText::new("root").strong().size(16.0));
                            ui.label(RichText::new("Administrator systemu").color(Color32::GRAY));
                        });
                    }
                    Section::About => {
                        ui.heading("O systemie");
                        ui.add_space(12.0);
                        egui::Grid::new("about").num_columns(2).spacing([30.0, 6.0]).show(ui, |ui| {
                            ui.label(RichText::new("System:").strong());   ui.label("Xiee OS"); ui.end_row();
                            ui.label(RichText::new("Wersja:").strong());   ui.label("0.1.0"); ui.end_row();
                            ui.label(RichText::new("Jezyk:").strong());    ui.label("Rust"); ui.end_row();
                            ui.label(RichText::new("Licencja:").strong()); ui.label("MIT"); ui.end_row();
                            ui.label(RichText::new("Cel:").strong());      ui.label("Klastry starych urzadzen"); ui.end_row();
                            ui.label(RichText::new("Autor:").strong());    ui.label("Miki"); ui.end_row();
                        });
                    }
                }
            });
        });
    }
}
