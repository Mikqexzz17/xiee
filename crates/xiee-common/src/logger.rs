use log::LevelFilter;
use env_logger::Builder;

/// Inicjalizuje logger dla komponentow Xiee OS
pub fn init_logger(level: LevelFilter) {
    Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
}
