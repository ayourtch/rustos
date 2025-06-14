#!/bin/sh
set -eux

# UEFI bootloader
rustup target add x86_64-unknown-uefi

# kernel
rustup target add x86_64-unknown-none
