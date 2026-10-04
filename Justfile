mod api "apps/achilles-api"
mod ui "apps/achilles-ui"
mod os "os"

[private]
default: help

# Shows this help message
help:
  @just --list

# Runs a check up
doctor proj="":
    #!/usr/bin/env bash
    set -e

    echo "Running a CheckUp for 'OS'"
    echo
    just os::doctor

