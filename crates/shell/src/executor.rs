//! Wykonywanie komend w Xiee Shell

use anyhow::Result;
use std::process::{Command, Stdio};
use crate::builtins;

/// Wykonaj komende (wbudowana lub zewnetrzna)
pub fn execute(line: &str) -> Result<()> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    let cmd = parts[0];
    let args = &parts[1..];
    
    // Sprobuj wbudowanych komend najpierw
    if let Some(result) = builtins::try_builtin(cmd, args) {
        return result;
    }
    
    // Uruchom zewnetrzny program
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("'{}': nie znaleziono komendy ({})", cmd, e))?;
    
    if !status.success() {
        if let Some(code) = status.code() {
            eprintln!("Komenda zakonczona z kodem: {}", code);
        }
    }
    
    Ok(())
}
