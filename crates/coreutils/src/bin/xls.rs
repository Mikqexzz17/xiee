//! xls - Listowanie zawartosci katalogu (zamiennik ls)
use anyhow::Result;
use std::{env, fs};

fn main() -> Result<()> {
    let path = env::args().nth(1).unwrap_or_else(|| String::from("."));
    let entries = fs::read_dir(&path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        
        if meta.is_dir() {
            println!("\x1b[34m{}/\x1b[0m", name);
        } else if meta.permissions().readonly() {
            println!("\x1b[31m{}\x1b[0m", name);
        } else {
            println!("{}", name);
        }
    }
    Ok(())
}
