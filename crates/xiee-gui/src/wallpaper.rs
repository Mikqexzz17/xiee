//! Ladowanie i renderowanie tapety

use egui::{ColorImage, TextureHandle, Context};
use anyhow::Result;

/// Zaladuj tapete z pliku do tekstury egui
pub fn load_wallpaper(ctx: &Context, path: &str) -> Result<TextureHandle> {
    let img = image::open(path)?.to_rgba8();
    let (w, h) = img.dimensions();
    let pixels = img.into_raw();
    
    let color_image = ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &pixels,
    );
    
    Ok(ctx.load_texture("wallpaper", color_image, egui::TextureOptions::LINEAR))
}
