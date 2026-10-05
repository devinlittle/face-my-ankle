#!/usr/bin/env bash
cd $SCRIPT_DIR/os

info "Checking if required tools exist"
echo
just doctor
echo

IMAGE_SIZE="1G"                                       # and this is the size of the entire image
BOOT_SIZE="512M"                                      # this is the size of /boot
PACKAGES=$(grep -v '^[[:space:]]*#' package_list.txt) # essentailly `cat`s out the file without comments

if [ ! -d "$MOUNT" ]; then
  info "Creating Build Dir"
  mkdir $BUILD $MOUNT
else
  info "Deleting Old RootFS"

  sudoif umount -R $MOUNT
  sudoif losetup -d $LOOP

  sudoif rm -rf $MOUNT
  mkdir $BUILD $MOUNT
fi
echo

info "Creating ${IMAGE_SIZE} image..."
sudoif rm -f "$IMAGE"
sudoif fallocate -l "$IMAGE_SIZE" "$IMAGE"

info "Partitioning..."
sudoif sfdisk "$IMAGE" <<EOF
label: dos
type=c, size=${BOOT_SIZE}, bootable
type=83
EOF

info "Setting up loop device..."
LOOP=$(sudoif losetup -f --show -P "$IMAGE")
info "Loop device: $LOOP"

#sudoif partprobe "$LOOP" || true
#sleep 0.5

BOOT_DEV="${LOOP}p1"
ROOT_DEV="${LOOP}p2"

info "Formatting partitions..."
sudoif mkfs.vfat -F 32 -n BOOT "$BOOT_DEV"
sudoif mkfs.ext4 -L root "$ROOT_DEV"

info "Mounting..."
sudoif mount "$ROOT_DEV" "$MOUNT"
sudoif mkdir -p "$MOUNT/boot"
sudoif mount "$BOOT_DEV" "$MOUNT/boot"

info "Bootstrapping Alpine packages..."
sudoif apk --root "$MOUNT" --initdb --allow-untrusted \
  --keys-dir /etc/apk/keys \
  --repository https://dl-cdn.alpinelinux.org/alpine/latest-stable/main \
  --repository https://dl-cdn.alpinelinux.org/alpine/latest-stable/community \
  add $PACKAGES

info "Copying finalize script..."
sudoif cp "$SCRIPTS_DIR/finalize.sh" "$MOUNT/finalize.sh"
sudoif chmod +x "$MOUNT/finalize.sh"

info "Copying over the build artifacts"
cd $SCRIPT_DIR
just api::package-tar ui::package-tar
sudoif cp -v $SCRIPT_DIR/apps/{achilles-ui/,achilles-api/}/*.tar.zst $MOUNT
cd $SCRIPT_DIR/os

# Detects which chroot helper to use
if command -v xchroot >/dev/null 2>&1; then
  chroot_cmd="xchroot"
elif command -v arch-chroot >/dev/null 2>&1; then
  chroot_cmd="arch-chroot"
else
  die "Neither xchroot nor arch-chroot found"
fi

info "Using chroot command: $chroot_cmd"

info "Running finalize.sh inside chroot..."
sudoif $chroot_cmd "$MOUNT" /finalize.sh

info "Removing finalize script and tar archives..."
sudoif rm -f "$MOUNT/finalize.sh" "$MOUNT/*tar.zst"
sudoif rm -f "$MOUNT/*tar.zst"

info "Rootfs & image preparation done"

info "Unmounting and Detatching LoopBack"
sync
sync
sleep 2

sudoif fuser -km "$MOUNT" 2>/dev/null || true
sleep 1

sudoif umount -R "$MOUNT" 2>/dev/null || {
  warn "umount -R failed *shakes fist in anger*"
  sudoif umount -l -R "$MOUNT" 2>/dev/null || true
}

# i have to make sure the loop is really gone
if LOOP_DEV=$(losetup -j "$IMAGE" -O NAME -n 2>/dev/null); then
  sudoif losetup -d "$LOOP_DEV"
fi

sudoif losetup -D 2>/dev/null || true
