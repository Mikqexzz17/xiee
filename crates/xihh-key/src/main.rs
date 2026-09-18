//! xihh key — Manager klastra urzadzen Xiee OS
//! Laczy stare urzadzenia w klaster i zarzadza nimi

use eframe::egui;
use egui::{Color32, RichText, Vec2, Pos2};
use xiee_gui::theme::{XieeColors, apply_xiee_theme};

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("xihh key — Cluster Manager")
            .with_inner_size([820.0, 580.0]),
        ..Default::default()
    };
    eframe::run_native("xihh-key", options, Box::new(|cc| Ok(Box::new(XihhKeyApp::new(cc))))).ok();
}

#[derive(Clone)]
struct Device {
    name: String,
    ip: String,
    status: DeviceStatus,
    cpu: u8,
    ram_mb: u32,
    device_type: &'static str,
}

#[derive(Clone, PartialEq)]
enum DeviceStatus { Online, Offline, Connecting }

impl DeviceStatus {
    fn color(&self) -> Color32 {
        match self {
            Self::Online => Color32::from_rgb(80, 200, 80),
            Self::Offline => Color32::from_rgb(200, 80, 80),
            Self::Connecting => Color32::from_rgb(220, 180, 50),
        }
    }
    fn label(&self) -> &str {
        match self {
            Self::Online => "Online",
            Self::Offline => "Offline",
            Self::Connecting => "Laczenie...",
        }
    }
}

struct XihhKeyApp {
    devices: Vec<Device>,
    selected: Option<usize>,
    new_ip: String,
    view: View,
}

#[derive(PartialEq)]
enum View { List, Topology }

impl XihhKeyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        Self {
            new_ip: String::new(),
            selected: None,
            view: View::List,
            devices: vec![
                Device { name: "xiee-master".into(), ip: "192.168.1.1".into(), status: DeviceStatus::Online, cpu: 23, ram_mb: 512, device_type: "🖥️" },
                Device { name: "stary-lenovo".into(), ip: "192.168.1.2".into(), status: DeviceStatus::Online, cpu: 45, ram_mb: 256, device_type: "💻" },
                Device { name: "raspi-01".into(), ip: "192.168.1.3".into(), status: DeviceStatus::Connecting, cpu: 0, ram_mb: 1024, device_type: "🔲" },
                Device { name: "dell-2009".into(), ip: "192.168.1.4".into(), status: DeviceStatus::Offline, cpu: 0, ram_mb: 0, device_type: "💻" },
            ],
        }
    }
}

impl eframe::App for XihhKeyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🔗 xihh key").color(XieeColors::TASKBAR_BG).size(20.0));
                ui.add_space(20.0);
                ui.selectable_value(&mut self.view, View::List, "Lista");
                ui.selectable_value(&mut self.view, View::Topology, "Topologia");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let online = self.devices.iter().filter(|d| d.status == DeviceStatus::Online).count();
                    ui.label(RichText::new(format!("{}/{} online", online, self.devices.len()))
                        .color(Color32::from_rgb(80, 200, 80)));
                });
            });
        });

        egui::SidePanel::left("sidebar").min_width(260.0).show(ctx, |ui| {
            ui.heading("Urzadzenia w klastrze");
            ui.add_space(8.0);

            for (i, dev) in self.devices.iter().enumerate() {
                let selected = self.selected == Some(i);
                let response = ui.selectable_label(selected, 
                    RichText::new(format!("{} {}", dev.device_type, dev.name)).size(14.0));
                
                if response.clicked() {
                    self.selected = Some(i);
                }
                
                // Status dot
                ui.horizontal(|ui| {
                    ui.add_space(28.0);
                    ui.label(RichText::new(format!("● {} — {}", dev.status.label(), dev.ip))
                        .color(dev.status.color())
                        .size(11.0));
                });
                ui.add_space(2.0);
            }

            ui.add_space(16.0);
            ui.separator();
            ui.label("Dodaj urzadzenie (IP):");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_ip);
                if ui.button("Dodaj").clicked() && !self.new_ip.is_empty() {
                    self.devices.push(Device {
                        name: format!("device-{}", self.devices.len() + 1),
                        ip: self.new_ip.clone(),
                        status: DeviceStatus::Connecting,
                        cpu: 0, ram_mb: 0,
                        device_type: "💻",
                    });
                    self.new_ip.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.view {
                View::List => {
                    if let Some(idx) = self.selected {
                        let dev = &self.devices[idx];
                        ui.heading(format!("{} {}", dev.device_type, dev.name));
                        ui.add_space(8.0);
                        egui::Grid::new("dev_info").num_columns(2).spacing([20.0, 4.0]).show(ui, |ui| {
                            ui.label("Status:"); ui.label(RichText::new(dev.status.label()).color(dev.status.color())); ui.end_row();
                            ui.label("IP:"); ui.label(&dev.ip); ui.end_row();
                            ui.label("CPU:"); ui.label(format!("{}%", dev.cpu)); ui.end_row();
                            ui.label("RAM:"); ui.label(format!("{} MB", dev.ram_mb)); ui.end_row();
                        });
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.button("SSH Terminal");
                            ui.button("Ping");
                            ui.button("Restart");
                            if ui.button(RichText::new("Usun").color(Color32::from_rgb(255,80,80))).clicked() {
                                self.devices.remove(idx);
                                self.selected = None;
                            }
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Wybierz urzadzenie z listy").color(Color32::GRAY).size(16.0));
                        });
                    }
                }
                View::Topology => {
                    ui.label(RichText::new("Topologia klastra").size(16.0));
                    // Prosty widok topologii - kazde urzadzenie jako wezel
                    let rect = ui.available_rect_before_wrap();
                    let painter = ui.painter();
                    let center = rect.center();
                    
                    // Narysuj master w centrum
                    painter.circle_filled(center, 30.0, Color32::from_rgb(80, 130, 200));
                    painter.text(center, egui::Align2::CENTER_CENTER, "MASTER", 
                        egui::FontId::proportional(10.0), Color32::WHITE);
                    
                    // Narysuj pozostale urzadzenia wokol
                    let count = self.devices.len().saturating_sub(1);
                    for (i, dev) in self.devices.iter().skip(1).enumerate() {
                        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                        let r = 140.0f32;
                        let pos = Pos2::new(
                            center.x + r * angle.cos(),
                            center.y + r * angle.sin(),
                        );
                        // Linia do mastera
                        painter.line_segment([center, pos], egui::Stroke::new(1.5, dev.status.color()));
                        // Wezel
                        painter.circle_filled(pos, 22.0, Color32::from_rgb(50, 80, 120));
                        painter.text(pos, egui::Align2::CENTER_CENTER, &dev.name[..dev.name.len().min(8)],
                            egui::FontId::proportional(9.0), Color32::WHITE);
                    }
                }
            }
        });
    }
}
