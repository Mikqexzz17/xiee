//! XIAC — Xi App Center
//! Manager pakietow dla Xiee OS z interfejsem graficznym

use eframe::egui;
use egui::{CentralPanel, Color32, RichText, ScrollArea, TopBottomPanel, Vec2};
use xiee_gui::theme::{XieeColors, apply_xiee_theme};

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("XIAC — Xi App Center")
            .with_inner_size([720.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native("XIAC", options, Box::new(|cc| Ok(Box::new(XiacApp::new(cc))))).ok();
}

struct Package {
    name: &'static str,
    description: &'static str,
    version: &'static str,
    installed: bool,
}

struct XiacApp {
    packages: Vec<Package>,
    search: String,
    selected_tab: Tab,
}

#[derive(PartialEq)]
enum Tab { Browse, Installed, Updates }

impl XiacApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        Self {
            search: String::new(),
            selected_tab: Tab::Browse,
            packages: vec![
                Package { name: "xiarr", description: "Lekka przegladarka Xiee OS", version: "0.1.0", installed: false },
                Package { name: "xihh-key", description: "Manager klastra urzadzen", version: "0.1.0", installed: false },
                Package { name: "neofetch", description: "Info o systemie", version: "7.1.0", installed: false },
                Package { name: "htop", description: "Monitor procesow", version: "3.3.0", installed: false },
                Package { name: "nano", description: "Edytor tekstowy", version: "7.2", installed: false },
                Package { name: "git", description: "System kontroli wersji", version: "2.45.0", installed: false },
                Package { name: "python3", description: "Python 3.x", version: "3.12.0", installed: false },
                Package { name: "ffmpeg", description: "Konwerter audio/wideo", version: "6.1.0", installed: false },
            ],
        }
    }
}

impl eframe::App for XiacApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Gorny pasek z zakladkami
        TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("📦 XIAC").color(XieeColors::TASKBAR_BG).size(20.0));
                ui.add_space(20.0);
                ui.selectable_value(&mut self.selected_tab, Tab::Browse, "Przegladaj");
                ui.selectable_value(&mut self.selected_tab, Tab::Installed, "Zainstalowane");
                ui.selectable_value(&mut self.selected_tab, Tab::Updates, "Aktualizacje");
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            // Wyszukiwarka
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.search);
            });
            ui.add_space(8.0);
            ui.separator();

            // Lista pakietow
            ScrollArea::vertical().show(ui, |ui| {
                let search_lower = self.search.to_lowercase();
                for pkg in self.packages.iter_mut() {
                    if !search_lower.is_empty() && !pkg.name.contains(&*search_lower) {
                        continue;
                    }
                    if self.selected_tab == Tab::Installed && !pkg.installed {
                        continue;
                    }

                    ui.group(|ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(pkg.name).strong().size(15.0));
                                ui.label(RichText::new(pkg.description).color(Color32::GRAY).size(12.0));
                                ui.label(RichText::new(format!("v{}", pkg.version)).color(Color32::from_rgb(100, 180, 100)).size(11.0));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if pkg.installed {
                                    if ui.button(RichText::new("Odinstaluj").color(Color32::from_rgb(255, 80, 80))).clicked() {
                                        pkg.installed = false;
                                    }
                                } else {
                                    if ui.button(RichText::new("Instaluj").color(Color32::WHITE)).clicked() {
                                        pkg.installed = true;
                                    }
                                }
                            });
                        });
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }
}
