/// Konfiguracja systemu Xiee OS
#[derive(Debug, Clone)]
pub struct XieeConfig {
    /// Nazwa systemu
    pub hostname: String,
    /// Wersja systemu
    pub version: String,
}

impl Default for XieeConfig {
    fn default() -> Self {
        Self {
            hostname: String::from("xiee"),
            version: String::from(env!("CARGO_PKG_VERSION")),
        }
    }
}
