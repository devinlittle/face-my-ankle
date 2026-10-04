#!/usr/bin/env bash
cd $SCRIPT_DIR/os

info "Compressing Ankle.img"

#xz -T0 -9 -k -f -v "$IMAGE"
sudoif zstd -T0 -19 "$IMAGE"

ls -lh "$BUILD"/*.img*

info "Compressed Ankle.img"
