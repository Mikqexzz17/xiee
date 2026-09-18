//! Xiee OS Init - Minimalny system inicjalizacji (PID 1)
//!
//! Zastepuje systemd - bardzo lekki, napisany w Ruscie.
//! Uruchamia serwisy, obsluguje sieroty procesow i sygnaly systemu.

use anyhow::Result;
use log::{info, warn};
use nix::sys::signal::{signal, SigHandler, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use xiee_common::logger::init_logger;

mod services;

fn main() -> Result<()> {
    init_logger(log::LevelFilter::Info);

    info!("=== Xiee OS Init v{} ===", env!("CARGO_PKG_VERSION"));
    info!("PID: {}", std::process::id());

    // Ustaw handler dla SIGCHLD (zbieranie zombie procesow)
    unsafe {
        signal(Signal::SIGCHLD, SigHandler::Handler(sigchld_handler))
            .expect("Nie mozna ustawic handlera SIGCHLD");
    }

    // Uruchom podstawowe serwisy
    services::start_all()?;

    // Glowna petla init - nigdy nie moze sie zakonczyc!
    info!("Init uruchomiony. Czekam na zdarzenia...");
    loop {
        // Zbierz zakonczonych potomkow (zapobiegaj zombie)
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(pid, code)) => {
                warn!("Proces {} zakonczyl sie z kodem {}", pid, code);
            }
            Ok(WaitStatus::Signaled(pid, sig, _)) => {
                warn!("Proces {} zakonczony sygnalem {:?}", pid, sig);
            }
            _ => {}
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

extern "C" fn sigchld_handler(_: libc::c_int) {
    // Obsluga SIGCHLD - zbieranie zombie procesow
}
