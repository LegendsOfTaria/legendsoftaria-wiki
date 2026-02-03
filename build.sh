#!/bin/bash
set -e  # exit on error

# Only install Rust if not already installed
if ! command -v cargo &> /dev/null
then
    echo "Rust not found, installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
else
    echo "Rust is already installed"
fi

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

cd "$(dirname "$0")/builder"
cargo run
