# Required for boot -begin-
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

cat >/boot/cmdline.txt <<EOF
modules=loop,squashfs,sd-mod,usb-storage console=tty1 root=/dev/mmcblk0p2 rootfstype=ext4 rootwait
EOF

cat >/etc/fstab <<EOF
/dev/mmcblk0p1  /boot  vfat  defaults,noatime  0 2
/dev/mmcblk0p2  /      ext4  defaults,noatime  0 1
EOF

rc-update add root boot
rc-update add localmount boot
rc-update add fsck boot
rc-update add hostname boot
# Required for boot -end-

echo "AnkleMonitor" >/etc/hostname
echo "127.0.0.1 localhost AnkleMonitor" >>/etc/hosts
rc-update add hostname boot

USERNAME="nived"
PASSWORD="pass"
SHELL_PATH="/bin/ash"

adduser -D -s "$SHELL_PATH" "$USERNAME"
echo "${USERNAME}:${PASSWORD}" | chpasswd

OPENRC_CMDS="/sbin/rc-service, /sbin/rc-status, /sbin/rc-update, /sbin/openrc"
NET_CMDS="/sbin/ip, /usr/bin/nmcli"
USER_CMDS="/usr/sbin/useradd, /usr/sbin/usermod, /usr/sbin/userdel, /usr/bin/passwd"
HOST_CMDS="/bin/hostname, /usr/bin/hostnamectl"

ALLOWED_CMDS="${OPENRC_CMDS}, ${NET_CMDS}, ${USER_CMDS}, ${HOST_CMDS}"

cat <<EOF >"/etc/sudoers"
# Allow $USERNAME passwordless sudo for specific commands
$USERNAME ALL=(ALL) NOPASSWD: $ALLOWED_CMDS
EOF

echo "%nived ALL=(ALL:ALL) ALL" >>/etc/sudoers

zstd -dc achilles-api.tar.zst | tar -xpf - -C /
zstd -dc achilles-ui.tar.zst | tar -xpf - -C /
#zstd -dc ankle.tar.zst | tar -xpf - -C /

rc-update add achilles-api default
rc-update add achilles-ui default
rc-update add agetty default
rc-update add sshd default

# Access Point stuff
# _________________________________________________________

echo "brcmfmac" >/etc/modules-load.d/brcmfmac.conf
rc-update add modules boot

cat >/etc/init.d/hostapd-ap <<'EOF'
#!/sbin/openrc-run

name="hostapd (AP)"
description="Wi-Fi Access Point for onboarding"

command="/usr/sbin/hostapd"
command_args="/etc/hostapd/hostapd-ap.conf"
command_background=true
pidfile="/run/hostapd-ap.pid"

depend() {
    after modules wpa_supplicant networkmanager
}

start_pre() {
    # Create the AP interface if it doesnt exist
    if ! ip link show ap0 >/dev/null 2>&1; then
        iw phy phy0 interface add ap0 type __ap
    fi
    ip link set ap0 up
    ip addr flush dev ap0
    ip addr add 172.30.30.1/24 dev ap0
}
EOF

chmod +x /etc/init.d/hostapd-ap

cat >/etc/hostapd/hostapd-ap.conf <<'EOF'
interface=ap0
driver=nl80211
ssid=Ankle Monitor
hw_mode=g
channel=6
wpa=2
wpa_passphrase=password
wpa_key_mgmt=WPA-PSK
rsn_pairwise=CCMP
EOF

rc-update add hostapd-ap default

cat >/etc/init.d/dnsmasq-ap <<'EOF'
#!/sbin/openrc-run

name="dnsmasq (AP)"
description="DHCP for the onboarding/management Access Point"

command="/usr/sbin/dnsmasq"
command_args="--interface=ap0 --bind-interfaces --except-interface=lo \
--dhcp-range=172.30.30.10,172.30.30.100,12h \
--dhcp-option=3,172.30.30.1 --dhcp-option=6,172.30.30.1 \
--address=/ankle.local/172.30.30.1 \
--address=/config.ankle/172.30.30.1 \
--address=/ankle.monitor/172.30.30.1 \
--address=/monitor-THIS.ankle/172.30.30.1 \
--address=/feet.feet/172.30.30.1 \
--address=/ankle.feet/172.30.30.1 \
--no-resolv --pid-file=/run/dnsmasq-ap.pid"
command_background=true
pidfile="/run/dnsmasq-ap.pid"

depend() {
    need hostapd-ap
}
EOF

chmod +x /etc/init.d/dnsmasq-ap

rc-update add dnsmasq-ap default

rc-update add dbus default
rc-update add udev sysinit

#cat >/etc/iwd/main.conf <<'EOF'
#[General]
#Blacklist=ap0
#EOF

#rc-update add iwd default
rc-update add wpa_supplicant default

cat >/etc/NetworkManager/NetworkManager.conf <<'EOF'
[device]
wifi.backend=wpa_supplicant

[keyfile]
unmanaged-devices=interface-name:ap0
EOF

rc-update add networkmanager default

rm -f /*.tar.*
