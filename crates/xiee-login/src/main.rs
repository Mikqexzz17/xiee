//! Xiee OS — Ekran logowania
//! Pierwsze uruchomienie: ustaw haslo
//! Kolejne: zaloguj sie

use anyhow::Result;
use eframe::egui;
use egui::{Align2, Area, Color32, FontId, Order, RichText, TextEdit, Vec2};
use sha2::{Sha256, Digest};
use std::{fs, path::PathBuf};
use xiee_gui::theme::apply_xiee_theme;

fn main() -> Result<()> {
    env_logger::init();
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_title("Xiee OS"),
        ..Default::default()
    };
    eframe::run_native(
        "Xiee Login",
        opts,
        Box::new(|cc| Ok(Box::new(LoginApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("{}", e))
}

fn passwd_path() -> PathBuf {
    PathBuf::from("/etc/xiee/passwd")
}

fn hash_pass(pass: &str) -> String {
    let mut h = Sha256::new();
    h.update(pass.as_bytes());
    format!("{:x}", h.finalize())
}

fn save_password(pass: &str) -> Result<()> {
    let path = passwd_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, hash_pass(pass))?;
    Ok(())
}

fn check_password(pass: &str) -> bool {
    if let Ok(stored) = fs::read_to_string(passwd_path()) {
        stored.trim() == hash_pass(pass)
    } else {
        false
    }
}

fn is_first_boot() -> bool {
    !passwd_path().exists()
}

fn launch_desktop() {
    std::process::Command::new("xiee-desktop").spawn().ok();
    std::process::exit(0);
}

enum Screen {
    FirstBoot { pass: String, confirm: String, error: String },
    Login { pass: String, error: String, shake: f32 },
}

struct LoginApp {
    screen: Screen,
    logo: Option<egui::TextureHandle>,
    wallpaper: Option<egui::TextureHandle>,
}

impl LoginApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        let wallpaper = xiee_gui::wallpaper::load_wallpaper(&cc.egui_ctx, "/usr/share/xiee/wallpaper.jpg").ok();
        let logo = xiee_gui::wallpaper::load_wallpaper(&cc.egui_ctx, "/usr/share/xiee/logo.jpg").ok();
        let screen = if is_first_boot() {
            Screen::FirstBoot { pass: String::new(), confirm: String::new(), error: String::new() }
        } else {
            Screen::Login { pass: String::new(), error: String::new(), shake: 0.0 }
        };
        Self { screen, logo, wallpaper }
    }
}

impl eframe::App for LoginApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tapeta w tle
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(20, 30, 50)))
            .show(ctx, |ui| {
                if let Some(tex) = &self.wallpaper {
                    ui.image((tex.id(), ui.available_size()));
                }
            });

        // Przyciemnij tlo
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgba_premultiplied(0, 0, 0, 120)))
            .show(ctx, |ui| {
                let screen_rect = ui.max_rect();

                match &mut self.screen {
                    Screen::FirstBoot { pass, confirm, error } => {
                        // === PIERWSZE URUCHOMIENIE ===
                        egui::Window::new("##setup")
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .fixed_size([360.0, 480.0])
                            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                            .frame(egui::Frame::none()
                                .fill(Color32::from_rgba_premultiplied(10, 10, 20, 230))
                                .corner_radius(egui::CornerRadius::same(16))
                                .inner_margin(egui::Margin::same(32)))
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    // Logo
                                    if let Some(tex) = &self.logo {
                                        let size = Vec2::splat(90.0);
                                        ui.add(egui::Image::new((tex.id(), size)).corner_radius(egui::CornerRadius::same(12)));
                                    }
                                    ui.add_space(12.0);
                                    ui.label(RichText::new("Witaj w Xiee OS!").color(Color32::WHITE).size(22.0).strong());
                                    ui.label(RichText::new("Pierwsze uruchomienie").color(Color32::from_white_alpha(150)).size(13.0));
                                    ui.add_space(6.0);
                                    ui.label(RichText::new("Ustaw haslo dla konta root:").color(Color32::from_white_alpha(180)).size(13.0));
                                    ui.add_space(16.0);

                                    ui.label(RichText::new("Haslo:").color(Color32::from_white_alpha(200)));
                                    let _pass_resp = ui.add(
                                        TextEdit::singleline(pass)
                                            .password(true)
                                            .desired_width(280.0)
                                            .hint_text("Wpisz haslo...")
                                    );
                                    ui.add_space(10.0);
                                    ui.label(RichText::new("Powtorz haslo:").color(Color32::from_white_alpha(200)));
                                    let _confirm_resp = ui.add(
                                        TextEdit::singleline(confirm)
                                            .password(true)
                                            .desired_width(280.0)
                                            .hint_text("Powtorz haslo...")
                                    );
                                    ui.add_space(8.0);

                                    if !error.is_empty() {
                                        ui.label(RichText::new(error.as_str()).color(Color32::from_rgb(255, 100, 100)).size(12.0));
                                    }

                                    ui.add_space(12.0);
                                    let btn = ui.add_sized([280.0, 40.0], egui::Button::new(
                                        RichText::new("Ustaw haslo i zaloguj").size(15.0)
                                    ));

                                    let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
                                    if btn.clicked() || enter {
                                        if pass.is_empty() {
                                            *error = "Haslo nie moze byc puste!".into();
                                        } else if pass != confirm {
                                            *error = "Hasla nie sa identyczne!".into();
                                            confirm.clear();
                                        } else if pass.len() < 4 {
                                            *error = "Haslo musi miec min. 4 znaki!".into();
                                        } else {
                                            match save_password(pass) {
                                                Ok(_) => launch_desktop(),
                                                Err(e) => *error = format!("Blad: {}", e),
                                            }
                                        }
                                    }
                                });
                            });
                    }

                    Screen::Login { pass, error, shake } => {
                        // === EKRAN LOGOWANIA ===
                        let offset = if *shake > 0.0 {
                            *shake -= ctx.input(|i| i.unstable_dt);
                            ((*shake * 40.0).sin() * 8.0)
                        } else { 0.0 };

                        egui::Window::new("##login")
                            .title_bar(false)
                            .resizable(false)
                            .collapsible(false)
                            .fixed_size([340.0, 420.0])
                            .anchor(Align2::CENTER_CENTER, Vec2::new(offset, 0.0))
                            .frame(egui::Frame::none()
                                .fill(Color32::from_rgba_premultiplied(10, 10, 20, 230))
                                .corner_radius(egui::CornerRadius::same(16))
                                .inner_margin(egui::Margin::same(32)))
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    // Logo
                                    if let Some(tex) = &self.logo {
                                        let size = Vec2::splat(100.0);
                                        ui.add(egui::Image::new((tex.id(), size)).corner_radius(egui::CornerRadius::same(14)));
                                    }
                                    ui.add_space(14.0);
                                    ui.label(RichText::new("Xiee OS").color(Color32::WHITE).size(24.0).strong());
                                    ui.label(RichText::new("root").color(Color32::from_white_alpha(160)).size(14.0));
                                    ui.add_space(20.0);

                                    let resp = ui.add(
                                        TextEdit::singleline(pass)
                                            .password(true)
                                            .desired_width(260.0)
                                            .hint_text("Haslo...")
                                            .font(FontId::proportional(15.0))
                                    );

                                    if !error.is_empty() {
                                        ui.add_space(6.0);
                                        ui.label(RichText::new(error.as_str()).color(Color32::from_rgb(255,100,100)).size(12.0));
                                    }

                                    ui.add_space(16.0);
                                    let btn = ui.add_sized([260.0, 42.0], egui::Button::new(
                                        RichText::new("Zaloguj  →").size(16.0).strong()
                                    ));

                                    let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
                                    if btn.clicked() || enter {
                                        if check_password(pass) {
                                            launch_desktop();
                                        } else {
                                            *error = "Nieprawidlowe haslo!".into();
                                            *shake = 0.5;
                                            pass.clear();
                                        }
                                    }
                                    resp.request_focus();
                                });
                            });

                        // Zegar na dole
                        let time_pos = egui::pos2(screen_rect.center().x, screen_rect.bottom() - 30.0);
                        ui.painter().text(time_pos, Align2::CENTER_CENTER,
                            "Xiee OS v0.1.0",
                            FontId::proportional(13.0),
                            Color32::from_white_alpha(120));

                        if *shake > 0.0 { ctx.request_repaint(); }
                    }
                }
            });

        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
