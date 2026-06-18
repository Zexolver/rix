#!/usr/bin/env bash
set -e

echo "=== 🚀 Updating Rix ==="

# 1. Pull the latest source code
echo "📦 Fetching latest changes from GitHub..."
git pull origin main

# 2. Compile the updated binary
echo "🛠️ Compiling latest release..."
cargo build --release

# 3. Detect existing installation and replace the binary
if [ -f "/usr/local/bin/rix" ]; then
    echo "🔒 Detected system-wide installation. Requesting sudo to update binary..."
    sudo cp target/release/rix-cli /usr/local/bin/rix
    sudo chmod +x /usr/local/bin/rix
    echo "✅ System-wide Rix updated successfully!"
elif [ -f "$HOME/.local/bin/rix" ]; then
    echo "👤 Detected user-space installation. Updating binary..."
    cp target/release/rix-cli "$HOME/.local/bin/rix"
    chmod +x "$HOME/.local/bin/rix"
    echo "✅ User-space Rix updated successfully!"
else
    echo "🦀 Falling back to Cargo installation..."
    cargo install --path rix-cli --force
    echo "✅ Cargo binary updated successfully!"
fi

echo "🎉 Update complete! Your environment configurations were not touched."
