use std::io::{self, Write};
use std::env;

/// Wyswietl prompt shella
pub fn print_prompt() {
    let cwd = env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| String::from("?"));
    
    let user = env::var("USER").unwrap_or_else(|_| String::from("root"));
    
    print!("\x1b[32m{}\x1b[0m:\x1b[34m{}\x1b[0m\x1b[1m$\x1b[0m ", user, cwd);
    let _ = io::stdout().flush();
}
