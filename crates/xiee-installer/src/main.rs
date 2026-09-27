//! xiee-installer — Instalator Xiee OS na dysk
//! Kreator krok po kroku: wybor dysku -> potwierdzenie -> instalacja -> gotowe

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, RichText, Stroke, Vec2};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use xiee_gui::theme::{XieeColors, apply_xiee_theme};

fn main() {
    env_logger::init();
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_title("Xiee OS — Instalator"),
        ..Default::default()
    };
    eframe::run_native(
        "xiee-installer",
        opts,
        Box::new(|cc| Ok(Box::new(Installer::new(cc)))),
    ).ok();
}

#[derive(Clone, PartialEq)]
enum Step {
    Welcome,
    SelectDisk,
    Confirm,
    Installing,
    Done,
    Error(String),
}

#[derive(Clone)]
struct DiskInfo {
    name: String,   // np. "sda"
    size: String,   // np. "256G"
    model: String,  // np. "Samsung SSD"
}

struct Installer {
    step: Step,
    disks: Vec<DiskInfo>,
    selected: Option<usize>,
    progress: Arc<Mutex<f32>>,
    log: Arc<Mutex<Vec<String>>>,
    install_done: Arc<Mutex<bool>>,
    install_err: Arc<Mutex<Option<String>>>,
}

impl Installer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        Self {
            step: Step::Welcome,
            disks: Self::scan_disks(),
            selected: None,
            progress: Arc::new(Mutex::new(0.0)),
            log: Arc::new(Mutex::new(vec![])),
            install_done: Arc::new(Mutex::new(false)),
            install_err: Arc::new(Mutex::new(None)),
        }
    }

    fn scan_disks() -> Vec<DiskInfo> {
        let stdout = Command::new("lsblk")
            .args(["-J", "-d", "-o", "NAME,SIZE,MODEL,TYPE"])
            .output()
            .map(|o| o.stdout)
            .unwrap_or_default();
        let json = String::from_utf8_lossy(&stdout);
        let mut disks = Vec::new();

        // Prosta parsowania JSON lsblk
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
            if let Some(arr) = val["blockdevices"].as_array() {
                for d in arr {
                    let typ = d["type"].as_str().unwrap_or("");
                    if typ == "disk" {
                        disks.push(DiskInfo {
                            name: d["name"].as_str().unwrap_or("?").to_string(),
                            size: d["size"].as_str().unwrap_or("?").to_string(),
                            model: d["model"].as_str().unwrap_or("Nieznany").trim().to_string(),
                        });
                    }
                }
            }
        }

        // Fallback jesli lsblk nie dziala
        if disks.is_empty() {
            if let Ok(out) = Command::new("lsblk").args(["-d", "-o", "NAME,SIZE"]).output() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && !parts[0].starts_with("sr") {
                        disks.push(DiskInfo {
                            name: parts[0].to_string(),
                            size: parts[1].to_string(),
                            model: "Dysk".to_string(),
                        });
                    }
                }
            }
        }
        disks
    }

    fn start_install(&mut self, disk: String) {
        let progress = self.progress.clone();
        let log      = self.log.clone();
        let done     = self.install_done.clone();
        let err      = self.install_err.clone();

        thread::spawn(move || {
            macro_rules! step {
                ($p:expr, $msg:expr) => {
                    *progress.lock().unwrap() = $p;
                    log.lock().unwrap().push($msg.to_string());
                };
            }
            macro_rules! run {
                ($cmd:expr, $args:expr) => {{
                    let r = Command::new($cmd).args($args).output();
                    match r {
                        Ok(o) if o.status.success() => Ok(()),
                        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
                        Err(e) => Err(e.to_string()),
                    }
                }};
            }

            let dev = format!("/dev/{}", disk);
            let efi_part  = format!("{}1", dev);
            let root_part = format!("{}2", dev);
            let mount_root = "/mnt/xiee-install";
            let mount_efi  = "/mnt/xiee-install/boot/efi";

            step!(0.05, "Odmontowywanie partycji...");
            let _ = Command::new("umount").args(["-R", mount_root]).output();

            step!(0.10, "Tworzenie tablicy partycji GPT...");
            if let Err(e) = run!("parted", &["-s", &dev,
                "mklabel", "gpt",
                "mkpart", "EFI", "fat32", "1MiB", "513MiB",
                "set", "1", "esp", "on",
                "mkpart", "ROOT", "ext4", "513MiB", "100%"]) {
                *err.lock().unwrap() = Some(format!("Blad partycjonowania: {}", e));
                return;
            }

            step!(0.20, "Formatowanie EFI (FAT32)...");
            let _ = Command::new("sleep").arg("1").output();
            if let Err(e) = run!("mkfs.fat", &["-F32", &efi_part]) {
                *err.lock().unwrap() = Some(format!("Blad formatowania EFI: {}", e));
                return;
            }

            step!(0.30, "Formatowanie root (ext4)...");
            if let Err(e) = run!("mkfs.ext4", &["-F", "-L", "xiee-root", &root_part]) {
                *err.lock().unwrap() = Some(format!("Blad formatowania root: {}", e));
                return;
            }

            step!(0.35, "Montowanie partycji...");
            let _ = Command::new("mkdir").args(["-p", mount_root]).output();
            if let Err(e) = run!("mount", &[&root_part, mount_root]) {
                *err.lock().unwrap() = Some(format!("Blad montowania: {}", e));
                return;
            }
            let _ = Command::new("mkdir").args(["-p", mount_efi]).output();
            let _ = run!("mount", &[&efi_part, mount_efi]);

            step!(0.40, "Kopiowanie systemu (to moze potrwac kilka minut)...");
            // Kopiuj z live rootfs lub z /
            let source = if std::path::Path::new("/run/live/rootfs/filesystem.squashfs").exists() {
                // Live boot — rozpakuj squashfs
                let _ = Command::new("unsquashfs")
                    .args(["-d", mount_root, "-f",
                           "/run/live/rootfs/filesystem.squashfs"])
                    .output();
                None
            } else {
                // Bezposrednia kopia systemu
                Some("/")
            };

            if let Some(src) = source {
                let excludes = [
                    "--exclude=/proc", "--exclude=/sys", "--exclude=/dev",
                    "--exclude=/run", "--exclude=/tmp", "--exclude=/mnt",
                    "--exclude=/var/xiee-rootfs", "--exclude=/var/xiee-iso",
                ];
                let mut args = vec!["-aAX"];
                args.extend(excludes.iter().copied());
                args.push(src);
                args.push(mount_root);
                if let Err(e) = run!("rsync", &args) {
                    *err.lock().unwrap() = Some(format!("Blad kopiowania: {}", e));
                    return;
                }
            }

            step!(0.75, "Konfigurowanie systemu na dysku...");
            // /etc/fstab
            let root_uuid = String::from_utf8_lossy(
                &Command::new("blkid").args(["-s", "UUID", "-o", "value", &root_part])
                    .output().map(|o| o.stdout).unwrap_or_default()
            ).trim().to_string();
            let efi_uuid = String::from_utf8_lossy(
                &Command::new("blkid").args(["-s", "UUID", "-o", "value", &efi_part])
                    .output().map(|o| o.stdout).unwrap_or_default()
            ).trim().to_string();

            let fstab = format!(
                "UUID={}  /          ext4  errors=remount-ro  0 1\n\
                 UUID={}  /boot/efi  vfat  umask=0077         0 1\n\
                 proc       /proc  proc  defaults  0 0\n",
                root_uuid, efi_uuid
            );
            std::fs::write(format!("{}/etc/fstab", mount_root), fstab).ok();

            step!(0.85, "Instalowanie GRUB (bootloader)...");
            // Montuj proc/dev/sys do chroot
            let _ = Command::new("mount").args(["--bind", "/dev",  &format!("{}/dev", mount_root)]).output();
            let _ = Command::new("mount").args(["-t", "proc", "proc", &format!("{}/proc", mount_root)]).output();
            let _ = Command::new("mount").args(["-t", "sysfs", "sysfs", &format!("{}/sys", mount_root)]).output();

            let grub_cmd = format!(
                "grub-install --target=x86_64-efi --efi-directory=/boot/efi \
                 --bootloader-id=XIEEOS --recheck && \
                 grub-mkconfig -o /boot/grub/grub.cfg"
            );
            let _ = Command::new("chroot").args([mount_root, "bash", "-c", &grub_cmd]).output();

            // Odmontuj
            let _ = Command::new("umount").args([&format!("{}/dev", mount_root)]).output();
            let _ = Command::new("umount").args([&format!("{}/proc", mount_root)]).output();
            let _ = Command::new("umount").args([&format!("{}/sys", mount_root)]).output();
            let _ = Command::new("umount").args(["-R", mount_root]).output();

            step!(1.0, "Instalacja zakonczona!");
            *done.lock().unwrap() = true;
        });
    }
}

impl eframe::App for Installer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sprawdz status watku instalacji
        if self.step == Step::Installing {
            if *self.install_done.lock().unwrap() {
                self.step = Step::Done;
            } else if let Some(e) = self.install_err.lock().unwrap().clone() {
                self.step = Step::Error(e);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
        }

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(18, 18, 26)))
            .show(ctx, |ui| {
                // Naglowek
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("⚙  Xiee OS — Instalator")
                        .color(XieeColors::TASKBAR_BG).size(22.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(step_label(&self.step))
                        .color(Color32::from_white_alpha(120)).size(12.0));
                });
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                match self.step.clone() {
                    Step::Welcome => self.show_welcome(ui),
                    Step::SelectDisk => self.show_select_disk(ui),
                    Step::Confirm => self.show_confirm(ui),
                    Step::Installing => self.show_installing(ui),
                    Step::Done => self.show_done(ui, ctx),
                    Step::Error(e) => self.show_error(ui, &e),
                }
            });
    }
}

impl Installer {
    fn show_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(RichText::new("Witaj w instalatorze Xiee OS!").color(Color32::WHITE).size(18.0));
            ui.add_space(16.0);

            ui.label(RichText::new("Ten kreator przeprowadzi Cie przez instalacje systemu na dysk.\nPo instalacji bedziesz mogl uruchamiac Xiee OS bez pendrive.")
                .color(Color32::from_white_alpha(180)).size(14.0));
            ui.add_space(12.0);

            Frame::NONE
                .fill(Color32::from_rgba_premultiplied(200, 80, 40, 40))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label(RichText::new("⚠  Instalacja USUNIE wszystkie dane z wybranego dysku!")
                        .color(Color32::from_rgb(255, 160, 60)).size(13.0));
                });

            ui.add_space(30.0);
            if ui.add(golden_btn("Rozpocznij instalacje  →")).clicked() {
                self.step = Step::SelectDisk;
            }
        });
    }

    fn show_select_disk(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Wybierz dysk docelowy:").color(Color32::WHITE).size(15.0));
        ui.add_space(12.0);

        if self.disks.is_empty() {
            ui.label(RichText::new("Nie znaleziono dysków. Sprawdz czy jestes root.").color(Color32::RED));
            // Tryb testowy
            self.disks = vec![DiskInfo {
                name: "sda".into(), size: "256G".into(), model: "Test Disk".into()
            }];
        }

        let disks = self.disks.clone();
        for (i, disk) in disks.iter().enumerate() {
            let selected = self.selected == Some(i);
            let bg = if selected {
                Color32::from_rgba_premultiplied(212, 168, 67, 50)
            } else {
                Color32::from_rgba_premultiplied(255, 255, 255, 10)
            };

            let border = if selected { XieeColors::TASKBAR_BG } else { Color32::from_white_alpha(30) };

            Frame::NONE
                .fill(bg)
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(1.5_f32, border))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(580.0);
                    let resp = ui.horizontal(|ui| {
                        ui.label(RichText::new("💾").size(24.0));
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(format!("/dev/{}", disk.name))
                                .color(XieeColors::TASKBAR_BG).size(15.0).strong());
                            ui.label(RichText::new(format!("{}  •  {}", disk.model, disk.size))
                                .color(Color32::from_white_alpha(160)).size(12.0));
                        });
                    });
                    if resp.response.interact(egui::Sense::click()).clicked() {
                        self.selected = Some(i);
                    }
                });
            ui.add_space(6.0);
        }

        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if ui.button(RichText::new("← Wstecz").size(13.0)).clicked() {
                self.step = Step::Welcome;
            }
            ui.add_space(16.0);
            let btn = egui::Button::new(
                RichText::new("Dalej  →").size(14.0).color(
                    if self.selected.is_some() { Color32::from_rgb(20,20,20) }
                    else { Color32::from_white_alpha(60) }
                )
            ).fill(if self.selected.is_some() { XieeColors::TASKBAR_BG } else { Color32::from_rgb(50,50,60) });

            if ui.add(btn).clicked() && self.selected.is_some() {
                self.step = Step::Confirm;
            }
        });
    }

    fn show_confirm(&mut self, ui: &mut egui::Ui) {
        let disk = self.selected
            .and_then(|i| self.disks.get(i))
            .cloned()
            .unwrap_or(DiskInfo { name: "?".into(), size: "?".into(), model: "?".into() });

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("⚠  Ostatnie ostrzezenie").color(Color32::from_rgb(255,120,60)).size(18.0).strong());
            ui.add_space(16.0);
            ui.label(RichText::new(format!(
                "Nastepujacy dysk zostanie sformatowany i WSZYSTKIE DANE usunete:\n\n  /dev/{}  —  {}  —  {}",
                disk.name, disk.model, disk.size
            )).color(Color32::WHITE).size(14.0));
            ui.add_space(24.0);
            ui.label(RichText::new("Nie mozna tego cofnac!").color(Color32::from_rgb(255,80,80)).size(14.0));
            ui.add_space(30.0);

            ui.horizontal(|ui| {
                if ui.button(RichText::new("← Anuluj").size(14.0)).clicked() {
                    self.step = Step::SelectDisk;
                }
                ui.add_space(20.0);
                if ui.add(golden_btn("✓  Instaluj Xiee OS")).clicked() {
                    self.step = Step::Installing;
                    let disk_name = disk.name.clone();
                    self.start_install(disk_name);
                }
            });
        });
    }

    fn show_installing(&mut self, ui: &mut egui::Ui) {
        let progress = *self.progress.lock().unwrap();
        let log = self.log.lock().unwrap().clone();

        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Instalacja w toku...").color(Color32::WHITE).size(16.0));
            ui.add_space(20.0);

            // Pasek postepu
            let bar_w = 500.0_f32;
            let bar_h = 14.0_f32;
            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, bar_h), egui::Sense::hover());
            ui.painter().rect_filled(bar_rect, CornerRadius::same(7), Color32::from_rgb(40,40,55));
            let fill_w = bar_w * progress;
            if fill_w > 0.0 {
                let fill = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_h));
                ui.painter().rect_filled(fill, CornerRadius::same(7), XieeColors::TASKBAR_BG);
            }
            ui.add_space(8.0);
            ui.label(RichText::new(format!("{:.0}%", progress * 100.0))
                .color(XieeColors::TASKBAR_BG).size(13.0));

            ui.add_space(16.0);
            // Log kroków
            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                for line in &log {
                    let color = if line.contains("Blad") { Color32::from_rgb(255,100,100) }
                        else { Color32::from_white_alpha(180) };
                    ui.label(RichText::new(format!("  ✓ {}", line)).color(color).size(12.0));
                }
            });
        });
    }

    fn show_done(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("✓  Instalacja zakonczona!").color(Color32::from_rgb(100,220,100)).size(22.0).strong());
            ui.add_space(20.0);
            ui.label(RichText::new("Xiee OS zostal zainstalowany na dysku.\nMozesz teraz uruchomic komputer bez pendrive.")
                .color(Color32::from_white_alpha(180)).size(14.0));
            ui.add_space(30.0);
            if ui.add(golden_btn("↺  Uruchom ponownie")).clicked() {
                let _ = std::process::Command::new("reboot").output();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn show_error(&mut self, ui: &mut egui::Ui, error: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(RichText::new("✕  Blad instalacji").color(Color32::from_rgb(255,80,80)).size(20.0).strong());
            ui.add_space(16.0);
            ui.label(RichText::new(error).color(Color32::from_white_alpha(180)).size(13.0));
            ui.add_space(24.0);
            if ui.button(RichText::new("← Sprobuj ponownie").size(14.0)).clicked() {
                self.step = Step::Welcome;
                *self.progress.lock().unwrap() = 0.0;
                self.log.lock().unwrap().clear();
                *self.install_done.lock().unwrap() = false;
                *self.install_err.lock().unwrap() = None;
            }
        });
    }
}

fn step_label(step: &Step) -> &'static str {
    match step {
        Step::Welcome    => "Krok 1/4 — Witamy",
        Step::SelectDisk => "Krok 2/4 — Wybor dysku",
        Step::Confirm    => "Krok 3/4 — Potwierdzenie",
        Step::Installing => "Krok 4/4 — Instalacja",
        Step::Done       => "Gotowe!",
        Step::Error(_)   => "Blad",
    }
}

fn golden_btn(label: &str) -> egui::Button<'_> {
    egui::Button::new(
        RichText::new(label).size(15.0).color(Color32::from_rgb(20, 20, 20)).strong()
    ).fill(XieeColors::TASKBAR_BG)
     .corner_radius(CornerRadius::same(8))
     .min_size(Vec2::new(220.0, 40.0))
}