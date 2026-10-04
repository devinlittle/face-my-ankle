#!/usr/bin/env bash
cd $SCRIPT_DIR/os

info "Cleaning UP"

cp -v $SCRIPT_DIR/apps/{achilles-ui/,achilles-api/}/*.tar.zst $MOUNT

# TODO: find things to clean up 😭
