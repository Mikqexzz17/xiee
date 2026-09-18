//! xcat - Wyswietlanie zawartosci plikow (zamiennik cat)
use anyhow::Result;
use std::{env, fs, io};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        // Czytaj ze stdin
        io::copy(&mut io::stdin(), &mut io::stdout())?;
        return Ok(());
    }
    
    for path in &args {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("{}: {}", path, e))?;
        print!("{}", content);
    }
    Ok(())
}
