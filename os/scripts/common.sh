#!/usr/bin/env bash

# runs commands without sudo if id=0 and w/ sudo if id!=0
sudoif() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  else
    sudo "$@"
  fi
}

# Gives error messages: taken from Gentoo EBuilds
die() {
  echo "error: $*" >&2
  exit 1
}

# Little info helper
info() {
  echo "INFO:==> $*"
}

# Same as info but says warn
warn() {
  echo "WARN:==> $*"
}

SCRIPT_DIR="$(printf $(cd ../ && pwd))"

BUILD="$SCRIPT_DIR/os/build"
SCRIPTS_DIR="$SCRIPT_DIR/os/scripts"
IMAGE="$BUILD/Ankle.img"
MOUNT="$BUILD/mount"

export -f sudoif info warn die
export SCRIPT_DIR BUILD SCRIPTS_DIR IMAGE MOUNT
