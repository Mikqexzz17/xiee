#!/bin/bash
# build-iso.sh - Pakuje rootfs Xiee OS do bootowalnego ISO
set -e

XIEE_DIR="/mnt/c/Users/mikol/Downloads/xieeos"
ROOTFS="/var/xiee-rootfs"
ISO_DIR="/var/xiee-iso"
ISO_OUT="$XIEE_DIR/xieeos-0.1.0.iso"

echo ""
echo "=== Xiee OS ISO builder ==="
echo ""

if [ "$(id -u)" != "0" ]; then
    echo "BLAD: Uruchom jako root: sudo bash build-iso.sh"
    exit 1
fi

# Sprawdz jadro
VMLINUZ="$ROOTFS/boot/vmlinuz"
INITRD="$ROOTFS/boot/initrd.img"

if [ ! -f "$VMLINUZ" ]; then
    echo "BLAD: Brak jadra w $VMLINUZ"
    echo "Uruchom: wsl -e bash -c 'sudo cp /boot/vmlinuz-7.0.0-28-generic /var/xiee-rootfs/boot/vmlinuz'"
    exit 1
fi

echo "[1/4] Jadro: $(ls -lh $VMLINUZ | awk '{print $5}')  initrd: $(ls -lh $INITRD | awk '{print $5}')"

# Krok 2: SquashFS
echo "[2/4] Pakowanie rootfs do SquashFS..."
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR/live"
mksquashfs "$ROOTFS" "$ISO_DIR/live/filesystem.squashfs" \
    -comp xz -noappend -quiet \
    -e boot -e proc -e sys -e dev -e tmp -e run
SIZE=$(du -sh "$ISO_DIR/live/filesystem.squashfs" | cut -f1)
echo "  SquashFS: $SIZE"

cp "$VMLINUZ" "$ISO_DIR/live/vmlinuz"
cp "$INITRD"  "$ISO_DIR/live/initrd.img"

# Krok 3: GRUB
echo "[3/4] Konfigurowanie GRUB..."
mkdir -p "$ISO_DIR/boot/grub"

cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set timeout=3
set default=0

set color_normal=yellow/black
set color_highlight=black/yellow
set menu_color_normal=yellow/black
set menu_color_highlight=black/yellow

menuentry "  Xiee OS 0.1.0  " {
    linux  /live/vmlinuz boot=live quiet splash nomodeset
    initrd /live/initrd.img
}

menuentry "  Xiee OS 0.1.0 (tryb awaryjny)  " {
    linux  /live/vmlinuz boot=live nomodeset
    initrd /live/initrd.img
}
EOF

grub-mkstandalone \
    --format=i386-pc \
    --output="$ISO_DIR/boot/grub/grub_eltorito.img" \
    --install-modules="linux normal iso9660 biosdisk memdisk search tar ls" \
    --modules="linux normal iso9660 biosdisk search" \
    "boot/grub/grub.cfg=$ISO_DIR/boot/grub/grub.cfg" 2>/dev/null

# Krok 4: ISO
echo "[4/4] Tworzenie ISO..."
xorriso -as mkisofs \
    -o "$ISO_OUT" \
    -b boot/grub/grub_eltorito.img \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -iso-level 3 \
    -volid "XIEEOS_01" \
    -full-iso9660-filenames \
    "$ISO_DIR" 2>/dev/null

ISO_SIZE=$(du -sh "$ISO_OUT" | cut -f1)
echo ""
echo "==========================================="
echo "  Xiee OS ISO gotowy!"
echo "==========================================="
echo "  Plik:    $ISO_OUT"
echo "  Rozmiar: $ISO_SIZE"
echo ""
echo "  Jak uruchomic:"
echo "  VirtualBox -> Nowa maszyna -> Wybierz ISO -> Start"
echo "  Rufus (Windows) -> wybierz xieeos-0.1.0.iso -> Start"
echo "==========================================="
