//! Zarzadzanie serwisami systemu Xiee OS

use anyhow::Result;
use log::{info, error};
use std::process::{Command, Stdio};

/// Definicja serwisu systemowego
pub struct Service {
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub critical: bool,
}

/// Lista serwisow do uruchomienia przy starcie
const SERVICES: &[Service] = &[
    Service {
        name: "login",
        command: "/usr/bin/xiee-login",
        args: &[],
        critical: true,
    },
];

/// Uruchom wszystkie serwisy
pub fn start_all() -> Result<()> {
    info!("Uruchamiam serwisy systemu...");
    for service in SERVICES {
        match start_service(service) {
            Ok(_) => info!("  [OK] {}", service.name),
            Err(e) => {
                error!("  [FAIL] {}: {}", service.name, e);
                if service.critical {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

/// Uruchom pojedynczy serwis
fn start_service(service: &Service) -> Result<()> {
    Command::new(service.command)
        .args(service.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Nie mozna uruchomic {}: {}", service.name, e))?;
    Ok(())
}
