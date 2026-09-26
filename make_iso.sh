#!/bin/bash
set -e
ISO_DIR=/var/xiee-iso
mkdir -p $ISO_DIR/boot/grub/i386-pc

# Skopiuj pliki modulow GRUB
cp -r /usr/lib/grub/i386-pc/. $ISO_DIR/boot/grub/i386-pc/

# Stworz core.img
grub-mkimage \
    -O i386-pc \
    -o /tmp/xiee-core.img \
    -p /boot/grub \
    biosdisk iso9660 linux search_label normal echo

# Polacz cdboot.img + core.img
cat /usr/lib/grub/i386-pc/cdboot.img /tmp/xiee-core.img > $ISO_DIR/boot/grub/eltorito.img
ls -lh $ISO_DIR/boot/grub/eltorito.img

# Buduj ISO
xorriso -as mkisofs \
    -o /mnt/c/Users/mikol/Downloads/xieeos/xieeos-0.1.0.iso \
    -b boot/grub/eltorito.img \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -iso-level 3 \
    -volid "XIEEOS_01" \
    -full-iso9660-filenames \
    $ISO_DIR

echo ""
echo "==========================================="
echo "  Xiee OS ISO gotowy!"
ISO_SIZE=$(du -sh /mnt/c/Users/mikol/Downloads/xieeos/xieeos-0.1.0.iso | cut -f1)
echo "  Rozmiar: $ISO_SIZE"
echo "==========================================="
