//! XFM — Xiee File Manager

use eframe::egui;
use egui::{Color32, RichText, ScrollArea};
use std::{fs, path::{Path, PathBuf}};
use std::os::unix::fs::PermissionsExt;
use xiee_gui::theme::{XieeColors, apply_xiee_theme};

fn main() {
    env_logger::init();
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("XFM — Menedzer plikow")
            .with_inner_size([900.0, 580.0]),
        ..Default::default()
    };
    eframe::run_native("xfm", opts, Box::new(|cc| Ok(Box::new(XfmApp::new(cc))))).ok();
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    extension: String,
}

impl FileEntry {
    fn icon(&self) -> &str {
        if self.is_dir { return "📁"; }
        match self.extension.as_str() {
            "rs"                          => "🦀",
            "txt" | "md"                  => "📄",
            "jpg" | "jpeg" | "png" | "gif" => "🖼",
            "mp3" | "ogg" | "wav"         => "🎵",
            "mp4" | "mkv"                 => "🎬",
            "zip" | "tar" | "gz"          => "📦",
            "pdf"                         => "📕",
            "sh"                          => "⌨",
            "toml" | "json" | "yaml"      => "⚙",
            _                             => "📄",
        }
    }

    fn size_str(&self) -> String {
        if self.is_dir { return "—".into(); }
        if self.size < 1024 { return format!("{} B", self.size); }
        if self.size < 1024 * 1024 { return format!("{:.1} KB", self.size as f64 / 1024.0); }
        format!("{:.1} MB", self.size as f64 / 1024.0 / 1024.0)
    }

    fn open(&self, status: &mut String) {
        if self.is_dir { return; }
        match self.extension.as_str() {
            "txt" | "md" | "rs" | "toml" | "json" | "yaml" | "sh" | "log" | "conf" | "css" | "html" | "js" => {
                // Otwórz plik tekstowy w shella z xcat
                std::process::Command::new("xiee-shell")
                    .args(["-c", &format!("xcat '{}'", self.path.display())])
                    .spawn()
                    .ok();
                *status = format!("Otwarty: {}", self.name);
            }
            _ => {
                // Spróbuj uruchomić jesli wykonywalny
                let executable = self.path.metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false);
                if executable {
                    std::process::Command::new(&self.path).spawn().ok();
                    *status = format!("Uruchomiono: {}", self.name);
                } else {
                    *status = format!("Nie wiem jak otworzyc: .{}", self.extension);
                }
            }
        }
    }
}

struct XfmApp {
    current: PathBuf,
    history: Vec<PathBuf>,
    entries: Vec<FileEntry>,
    selected: Option<usize>,
    search: String,
    bookmarks: Vec<(&'static str, &'static str)>,
    status: String,
}

impl XfmApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_xiee_theme(&cc.egui_ctx);
        let start = PathBuf::from("/root");
        let mut app = Self {
            entries: vec![],
            history: vec![],
            selected: None,
            search: String::new(),
            status: String::new(),
            bookmarks: vec![
                ("🏠 Dom",        "/root"),
                ("📁 Dokumenty",  "/root/Documents"),
                ("🖼 Obrazy",     "/root/Pictures"),
                ("📥 Pobrane",    "/root/Downloads"),
                ("⚙ System",     "/usr/share/xiee"),
                ("💾 Root FS",    "/"),
            ],
            current: start.clone(),
        };
        app.load_dir(&start.clone());
        app
    }

    fn load_dir(&mut self, path: &Path) {
        self.selected = None;
        self.entries.clear();
        let rd = match fs::read_dir(path) {
            Ok(r) => r,
            Err(e) => { self.status = format!("Blad: {}", e); return; }
        };
        let mut entries: Vec<FileEntry> = rd.filter_map(|e| e.ok()).map(|e| {
            let p = e.path();
            let meta = e.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta.map(|m| m.len()).unwrap_or(0);
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            let extension = p.extension().unwrap_or_default().to_string_lossy().to_lowercase();
            FileEntry { name, path: p, is_dir, size, extension: extension.into() }
        }).collect();
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
        self.entries = entries;
        self.current = path.to_path_buf();
        self.status = format!("{} elementow", self.entries.len());
    }

    fn navigate(&mut self, path: PathBuf) {
        self.history.push(self.current.clone());
        self.load_dir(&path.clone());
    }

    fn go_back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.load_dir(&prev.clone());
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.current.parent().map(|p| p.to_path_buf()) {
            self.navigate(parent);
        }
    }
}

impl eframe::App for XfmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("📁 XFM").color(XieeColors::TASKBAR_BG).size(18.0));
                ui.add_space(8.0);
                if ui.button("⮜").on_hover_text("Wstecz").clicked() { self.go_back(); }
                if ui.button("⬆").on_hover_text("Wyzej").clicked() { self.go_up(); }
                ui.add_space(4.0);
                ui.label(RichText::new(self.current.display().to_string())
                    .color(Color32::from_white_alpha(180)).size(13.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.search)
                        .hint_text("🔍 Szukaj...").desired_width(150.0));
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status).color(Color32::GRAY).size(12.0));
                if let Some(sel) = self.selected {
                    if let Some(e) = self.entries.get(sel) {
                        ui.separator();
                        ui.label(RichText::new(format!("{} — {}", e.name, e.size_str()))
                            .color(Color32::GRAY).size(12.0));
                    }
                }
            });
        });

        egui::SidePanel::left("bookmarks").min_width(160.0).max_width(200.0).show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Ulubione").color(XieeColors::TASKBAR_BG).strong());
            ui.add_space(6.0);
            let bookmarks = self.bookmarks.clone();
            for (label, path) in bookmarks {
                if ui.selectable_label(false, RichText::new(label).size(13.0)).clicked() {
                    self.navigate(PathBuf::from(path));
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let search = self.search.to_lowercase();
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                let entries: Vec<(usize, FileEntry)> = self.entries.iter().cloned().enumerate()
                    .filter(|(_, e)| search.is_empty() || e.name.to_lowercase().contains(&search))
                    .collect();

                for (i, entry) in &entries {
                    let is_selected = self.selected == Some(*i);
                    let label = format!("{} {}", entry.icon(), entry.name);
                    let color = if entry.is_dir {
                        Color32::from_rgb(100, 160, 255)
                    } else {
                        Color32::WHITE
                    };
                    let resp = ui.selectable_label(is_selected,
                        RichText::new(&label).size(13.0).color(color));

                    if resp.clicked() { self.selected = Some(*i); }
                    if resp.double_clicked() {
                        if entry.is_dir {
                            let p = entry.path.clone();
                            self.navigate(p);
                        } else {
                            entry.open(&mut self.status);
                        }
                    }
                }
            });
        });
    }
}