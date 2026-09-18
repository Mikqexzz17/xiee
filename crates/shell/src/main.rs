//! Xiee OS Shell - Minimalny shell systemowy
//!
//! Lekka alternatywa dla bash/sh. Obsluguje podstawowe komendy,
//! uruchamianie programow i proste skrypty.

use anyhow::Result;
use log::info;
use std::io::{self, Write, BufRead};
use xiee_common::logger::init_logger;

mod executor;
mod builtins;
mod prompt;

fn main() -> Result<()> {
    init_logger(log::LevelFilter::Warn);
    
    let stdin = io::stdin();
    
    println!("Xiee Shell v{}", env!("CARGO_PKG_VERSION"));
    println!("Wpisz 'help' aby zobaczyc dostepne komendy.");
    
    loop {
        // Wyswietl prompt
        prompt::print_prompt();
        io::stdout().flush()?;
        
        // Wczytaj linie
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            // EOF - koniec sesji
            println!("\nDo widzenia!");
            break;
        }
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Wykonaj komende
        if let Err(e) = executor::execute(line) {
            eprintln!("Blad: {}", e);
        }
    }
    
    Ok(())
}
