# Xiee OS

> Lekki system operacyjny oparty na Linuksie, napisany w Rust.
> Zaprojektowany dla klastrów starych urządzeń.

## Komponenty

| Komponent | Binarny | Opis |
|-----------|---------|------|
| xiee-login | xiee-login | Ekran logowania z pierwszym uruchomieniem |
| xiee-desktop | xiee-desktop | Desktop + tapeta + taskbar + launcher |
| xiee-init | init | System inicjalizacji (PID 1) |
| xiee-shell | xiee-shell | Minimalny shell systemowy |
| xiee-coreutils | xls, xcat, xecho | Podstawowe narzedzia |
| xiac | xiac | Centrum aplikacji (App Center) |
| xiarr | xiarr | Przeglądarka internetowa (Tauri + WebKit) |
| xihh-key | xihh-key | Manager klastra urzadzen |
| xfm | xfm | Menedzer plikow |
| winyy | winyy | Ustawienia systemowe |

## Struktura projektu

`
xieeos/
├── Cargo.toml           # Workspace
├── assets/
│   ├── wallpaper.jpg    # Tapeta systemowa
│   └── logo.jpg         # Logo Xiee OS
├── xiarr/               # Przeglądarka (Tauri)
└── crates/
    ├── xiee-common/     # Wspolna biblioteka
    ├── xiee-gui/        # Biblioteka UI (egui)
    ├── xiee-login/      # Ekran logowania
    ├── xiee-desktop/    # Desktop environment
    ├── init/            # Init system
    ├── shell/           # Shell
    ├── coreutils/       # Narzedzia systemowe
    ├── xiac/            # App Center
    ├── xihh-key/        # Cluster manager
    ├── xfm/             # File manager
    └── winyy/           # Ustawienia
`

## Jak budowac

`ash
# W WSL:
source ~/.cargo/env

# Buduj wszystko (oprocz XIARR)
cargo build --release

# Buduj XIARR osobno
cd xiarr && cargo tauri build --no-bundle
`

## Kolejnosc uruchamiania

`
1. xiee-login   → ekran logowania
2. xiee-desktop → desktop z taskbarem
3. Aplikacje uruchamiane z taskbara/launchera
`

## Roadmapa

- [x] CLI fundament (init, shell, coreutils)
- [x] Desktop (tapeta, taskbar, launcher)
- [x] XIAC - App Center
- [x] XIARR - Przegladarka
- [x] xihh key - Cluster Manager
- [x] WINYY - Ustawienia
- [x] XFM - Menedzer plikow
- [x] Ekran logowania
- [ ] Obraz ISO/rootfs
- [ ] GitHub repository

## Licencja

MIT — Autor: Miki
