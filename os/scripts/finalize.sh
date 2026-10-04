# TODO: actually finalize the system

echo "Installing Achilles UI and API"
zstd -dc achilles-api.tar.zst | tar -xpf - -C /
zstd -dc achilles-ui.tar.zst | tar -xpf - -C /
#zstd -dc ankle.tar.zst | tar -xpf - -C /

echo "Enabling their services"
rc-update add achilles-api default
rc-update add achilles-ui default
rc-update add agetty default

# Required for boot
cat >/boot/config.txt <<EOF
[all]
kernel=vmlinuz-rpi
initramfs initramfs-rpi
arm_64bit=1
include usercfg.txt
EOF

cat >/boot/usercfg.txt <<EOF
disable_splash=1
enable_uart=1
EOF

# DEBUG
#cat >/boot/cmdline.txt <<EOF
#modules=loop,squashfs,sd-mod,usb-storage console=tty1 console=ttyAMA0,115200 root=/dev/mmcblk0p2 rootfstype=ext4 rootwait fullscreen_logo=1 fullscreen_logo_name=logo.tga vt.global_cursor_default=0
#EOF

# NORMAL
cat >/boot/cmdline.txt <<EOF
modules=loop,squashfs,sd-mod,usb-storage quit root=/dev/mmcblk0p2 rootfstype=ext4 rootwait fullscreen_logo=1 fullscreen_logo_name=logo.tga vt.global_cursor_default=0
EOF

mkdir /tmp/initrd
cd /tmp/initrd
gzip -d </boot/initramfs-rpi | cpio -idm

mkdir -p lib/firmware
cp /lib/firmware/logo.tga lib/firmware/

find . | cpio -o -H newc | gzip >/boot/initramfs-rpi
cd /
rm -rf tmp/initrd

ash
