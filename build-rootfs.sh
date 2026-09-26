#!/bin/bash
# build-rootfs.sh - Buduje rootfs Xiee OS
# WAZNE: rootfs musi byc na natywnym ext4 WSL, nie na /mnt/c (NTFS)
set -e

XIEE_DIR="/mnt/c/Users/mikol/Downloads/xieeos"
ROOTFS="/var/xiee-rootfs"
RELEASE_DIR="$XIEE_DIR/target/release"

echo ""
echo "=== Xiee OS rootfs builder ==="
echo "Rootfs: $ROOTFS (natywny ext4 WSL)"
echo ""

if [ "$(id -u)" != "0" ]; then
    echo "BLAD: Uruchom jako root: sudo bash build-rootfs.sh"
    exit 1
fi

# Krok 1: debootstrap — minimalne Ubuntu 22.04
echo "[1/6] Tworzenie minimalnego rootfs (Ubuntu 22.04 Jammy)..."
if [ -d "$ROOTFS/usr" ]; then
    echo "  Rootfs juz istnieje, pomijam debootstrap."
else
    mkdir -p "$ROOTFS"
    debootstrap --arch=amd64 --variant=minbase jammy "$ROOTFS" http://archive.ubuntu.com/ubuntu
    echo "  Debootstrap zakończony."
fi

# Krok 2: Montuj proc/sys/dev do chroot
echo "[2/6] Instalowanie zaleznosci graficznych..."
mount -t proc proc "$ROOTFS/proc" 2>/dev/null || true
mount -t sysfs sysfs "$ROOTFS/sys" 2>/dev/null || true
mount --bind /dev "$ROOTFS/dev" 2>/dev/null || true
mount --bind /dev/pts "$ROOTFS/dev/pts" 2>/dev/null || true

cat > "$ROOTFS/tmp/install_deps.sh" << 'INNER'
#!/bin/bash
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y --no-install-recommends \
    libgtk-3-0 \
    libxcb-render0 libxcb-shape0 libxcb-xfixes0 \
    libxkbcommon0 \
    libssl3 \
    libwebkit2gtk-4.1-0 \
    libjavascriptcoregtk-4.1-0 \
    xserver-xorg-core \
    xinit \
    x11-xserver-utils \
    openbox \
    curl \
    ca-certificates \
    fonts-dejavu-core \
    linux-image-generic \
    live-boot \
    2>/dev/null
apt-get clean
rm -rf /var/lib/apt/lists/*
INNER
chmod +x "$ROOTFS/tmp/install_deps.sh"
chroot "$ROOTFS" /tmp/install_deps.sh

# Odmontuj
umount "$ROOTFS/dev/pts" 2>/dev/null || true
umount "$ROOTFS/dev"     2>/dev/null || true
umount "$ROOTFS/sys"     2>/dev/null || true
umount "$ROOTFS/proc"    2>/dev/null || true
echo "  Zaleznosci zainstalowane."

# Krok 3: Kopiuj binaria Xiee OS
echo "[3/6] Kopiowanie binarnych Xiee OS..."
BINS=(xiee-login xiee-desktop xiee-shell xiee-splash xiac xihh-key xfm winyy xnotify init xls xcat xecho)
for bin in "${BINS[@]}"; do
    if [ -f "$RELEASE_DIR/$bin" ]; then
        cp "$RELEASE_DIR/$bin" "$ROOTFS/usr/bin/$bin"
        chmod +x "$ROOTFS/usr/bin/$bin"
        echo "  [OK] $bin"
    else
        echo "  [SKIP] $bin"
    fi
done

XIARR="$XIEE_DIR/xiarr/src-tauri/target/release/app"
if [ -f "$XIARR" ]; then
    cp "$XIARR" "$ROOTFS/usr/bin/xiarr"
    chmod +x "$ROOTFS/usr/bin/xiarr"
    echo "  [OK] xiarr"
fi

# Krok 4: Assety
echo "[4/6] Kopiowanie assetow..."
mkdir -p "$ROOTFS/usr/share/xiee"
for asset in wallpaper.jpg logo.jpg; do
    if [ -f "$XIEE_DIR/assets/$asset" ]; then
        cp "$XIEE_DIR/assets/$asset" "$ROOTFS/usr/share/xiee/"
        echo "  [OK] $asset"
    fi
done

# Krok 5: Konfiguracja systemu
echo "[5/6] Konfiguracja systemu..."

echo "xiee" > "$ROOTFS/etc/hostname"

cat > "$ROOTFS/etc/hosts" << 'EOF'
127.0.0.1   localhost
127.0.1.1   xiee
EOF

mkdir -p "$ROOTFS/etc/xiee"
mkdir -p "$ROOTFS/root/.xiee/desktop"
mkdir -p "$ROOTFS/root/Downloads"
mkdir -p "$ROOTFS/root/Documents"

# .xinitrc — uruchamia Xiee OS
cat > "$ROOTFS/root/.xinitrc" << 'EOF'
#!/bin/bash
xset s off
xset -dpms
exec xiee-splash
EOF
chmod +x "$ROOTFS/root/.xinitrc"

# Autologowanie do X przy starcie na TTY1
mkdir -p "$ROOTFS/etc/profile.d"
cat > "$ROOTFS/etc/profile.d/xiee.sh" << 'EOF'
export PATH="/usr/bin:/usr/local/bin:/bin:/sbin:/usr/sbin"
if [ "$(tty)" = "/dev/tty1" ] && [ -z "$DISPLAY" ]; then
    startx /root/.xinitrc -- :0 vt1 &>/tmp/xiee-x.log
fi
EOF
chmod +x "$ROOTFS/etc/profile.d/xiee.sh"

# Autologowanie root na TTY1 przez getty
mkdir -p "$ROOTFS/etc/systemd/system/getty@tty1.service.d"
cat > "$ROOTFS/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear %I $TERM
EOF

# Usuniecie hasla root (haslo ustawi xiee-login)
chroot "$ROOTFS" passwd -d root 2>/dev/null || true

echo "  Konfiguracja gotowa."
echo ""
echo "=== Rootfs zbudowany: $ROOTFS ==="
echo "Rozmiar: $(du -sh $ROOTFS | cut -f1)"
echo ""
echo "Nastepny krok: sudo bash build-iso.sh"
