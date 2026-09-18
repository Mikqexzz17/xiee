//! Wbudowane komendy shella

use anyhow::{Result, bail};
use std::env;
use std::path::Path;

/// Wykonaj wbudowana komende. Zwraca None jesli komenda nie jest wbudowana.
pub fn try_builtin(cmd: &str, args: &[&str]) -> Option<Result<()>> {
    match cmd {
        "cd" => Some(cmd_cd(args)),
        "exit" | "quit" => Some(cmd_exit(args)),
        "help" => Some(cmd_help()),
        "echo" => Some(cmd_echo(args)),
        "pwd" => Some(cmd_pwd()),
        "clear" => Some(cmd_clear()),
        _ => None,
    }
}

fn cmd_cd(args: &[&str]) -> Result<()> {
    let path = args.first().map(|s| *s).unwrap_or("/root");
    let path = Path::new(path);
    env::set_current_dir(path)
        .map_err(|e| anyhow::anyhow!("cd: {}: {}", path.display(), e))
}

fn cmd_exit(_args: &[&str]) -> Result<()> {
    std::process::exit(0);
}

fn cmd_help() -> Result<()> {
    println!("Xiee Shell - dostepne komendy wbudowane:");
    println!("  cd [katalog]  - zmien katalog");
    println!("  pwd           - pokaz biezacy katalog");
    println!("  echo [tekst]  - wyswietl tekst");
    println!("  clear         - wyczysc ekran");
    println!("  exit / quit   - wyjdz z shella");
    println!();
    println!("Mozesz tez uruchamiac dowolne programy z systemu.");
    Ok(())
}

fn cmd_echo(args: &[&str]) -> Result<()> {
    println!("{}", args.join(" "));
    Ok(())
}

fn cmd_pwd() -> Result<()> {
    let cwd = env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

fn cmd_clear() -> Result<()> {
    print!("\x1b[2J\x1b[H");
    Ok(())
}
