#!/usr/bin/env bash

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Starting Rix Multi-Platform Core Provisioning Layer ===${NC}"

ARCH=$(uname -m)
OS=$(uname -s)
echo -e "Host System Environment Detected: ${YELLOW}${OS} (${ARCH})${NC}"

# Create backup archive if a previous config exists
RIX_TARGET_DIR="$HOME/.config/rix"
if [ -d "$RIX_TARGET_DIR" ]; then
    echo -e "${YELLOW}Warning: Pre-existing configuration profile layout tracked.${NC}"
    mkdir -p "$HOME/Documents/rix-archived-profiles"
    tar -czf "$HOME/Documents/rix-archived-profiles/backup-$(date +%s).tar.gz" -C "$HOME/.config" rix || true
fi

# Ensure cargo is present
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Cargo toolchain not found. Bootstrapping via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo -e "${GREEN}Executing native profile build sequence...${NC}"
cargo build --release

# INTERACTIVE SCOPE SELECTION
echo -e "\n${BLUE}Select Installation Target Scope:${NC}"
echo -e "1) System-Wide  (Installs to /usr/local/bin, requires sudo)"
echo -e "2) User-Space   (Installs to $HOME/.local/bin)"
read -rp "Enter choice [1-2]: " SCOPE_CHOICE

if [ "$SCOPE_CHOICE" = "1" ]; then
    TARGET_BIN_DIR="/usr/local/bin"
    echo -e "Deploying system-wide binary pointer asset..."
    sudo cp target/release/rix-cli "$TARGET_BIN_DIR/rix"
    sudo chmod +x "$TARGET_BIN_DIR/rix"
else
    TARGET_BIN_DIR="$HOME/.local/bin"
    mkdir -p "$TARGET_BIN_DIR"
    echo -e "Deploying user-space binary pointer asset..."
    cp target/release/rix-cli "$TARGET_BIN_DIR/rix"
    chmod +x "$TARGET_BIN_DIR/rix"
    
    # Auto-append to path if missing from common shell configs
    if [[ ":$PATH:" != *":$TARGET_BIN_DIR:"* ]]; then
        echo "export PATH=\"\$PATH:$TARGET_BIN_DIR\"" >> "$HOME/.bashrc"
        echo -e "${YELLOW}Added $TARGET_BIN_DIR to your PATH via ~/.bashrc${NC}"
    fi
fi

echo -e "${GREEN}Invoking Rix engine to initialize clean file layout templates...${NC}"
# Use the correct beginner command API footprint to target upstream package registration
"$TARGET_BIN_DIR/rix" install coreutils

echo -e "${GREEN}=== Rix Automated System Provisioning Completed Successfully! ===${NC}"
echo -e "Target binary operational at: ${YELLOW}${TARGET_BIN_DIR}/rix${NC}"
if [ "$SCOPE_CHOICE" = "2" ]; then
    echo -e "Please run: ${YELLOW}source ~/.bashrc${NC} to refresh your current terminal environment path layout."
fi
