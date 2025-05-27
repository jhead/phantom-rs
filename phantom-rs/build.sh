#!/bin/zsh

# build.sh
# Script to build the Rust project for macOS, iOS, and Linux.

# Exit immediately if a command exits with a non-zero status.
set -e

echo "Adding Rust targets..."
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim # For Apple Silicon Macs
rustup target add x86_64-apple-ios     # For Intel Macs (simulator)
rustup target add x86_64-unknown-linux-gnu
echo "Finished adding targets."

# --- Build for macOS ---
echo "\nBuilding for macOS (Apple Silicon)..."
cargo build --target aarch64-apple-darwin --release

echo "\nBuilding for macOS (Intel)..."
cargo build --target x86_64-apple-darwin --release

# --- Build for iOS ---
echo "\nBuilding for iOS (Device - ARM64)..."
cargo build --target aarch64-apple-ios --release

echo "\nBuilding for iOS (Simulator - ARM64 for Apple Silicon Macs)..."
cargo build --target aarch64-apple-ios-sim --release

# If you need to support Intel iOS simulators, uncomment the line below
# echo "\nBuilding for iOS (Simulator - x86_64 for Intel Macs)..."
# cargo build --target x86_64-apple-ios --release

# --- Build for Linux ---
echo "\nBuilding for Linux (x86_64)..."
cargo build --target x86_64-unknown-linux-gnu --release

echo "\nAll builds completed successfully!"
echo "Output binaries can be found in target/<target-triple>/release/"
