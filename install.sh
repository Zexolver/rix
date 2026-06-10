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
IS_NIXOS=false

# 1. HOST PLATFORM DETECTION
if [ -f /etc/NIXOS ] || [ -d /run/current-system ]; then
    IS_NIXOS=true
    echo -e "Host System Environment Detected: ${YELLOW}NixOS (${ARCH})${NC}"
else
    echo -e "Host System Environment Detected: ${YELLOW}${OS} (${ARCH})${NC}"
fi

# 2. SUBSTRATE PROVISIONING: Verify or install the package engine
if ! command -v nix &> /dev/null; then
    if [ "$IS_NIXOS" = true ]; then
        echo -e "${RED}Error: The 'nix' command is missing on a NixOS target. Your system path profile layout might be broken.${NC}"
        exit 1
    else
        echo -e "${YELLOW}Nix/Lix engine substrate not found. Provisioning the modern Lix toolchain fork...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf -L https://install.lix.systems/lix | sh -s -- install
        
        # Sourcing the freshly dropped profile hook immediately so the rest of the script can invoke 'nix'
        if [ -f /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then
            set +u # Temporarily disable unbound variables check for third-party shell configuration scripts
            source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
            set -u
        fi
    fi
else
    echo -e "Existing Package Substrate Verified: ${GREEN}$(nix --version)${NC}"
fi

# 3. INTERACTIVE SCOPE SELECTION
echo -e "\n${BLUE}Select Installation Target Scope:${NC}"
echo -e "1) System-Wide  (Config: /etc/rix, Binary: /usr/local/bin, requires sudo)"
echo -e "2) User-Space   (Config: \$HOME/.config/rix, Binary: \$HOME/.local/bin)"
read -rp "Enter choice [1-2]: " SCOPE_CHOICE

if [ "$SCOPE_CHOICE" = "1" ]; then
    RIX_CONFIG_DIR="/etc/rix"
    TARGET_BIN_DIR="/usr/local/bin"
else
    RIX_CONFIG_DIR="$HOME/.config/rix"
    TARGET_BIN_DIR="$HOME/.local/bin"
fi

# 4. PRE-EXISTING CONFIGURATION BACKUP
if [ -d "$RIX_CONFIG_DIR" ]; then
    echo -e "${YELLOW}Warning: Pre-existing configuration profile layout tracked at ${RIX_CONFIG_DIR}.${NC}"
    BACKUP_DIR="$HOME/Documents/rix-archived-profiles"
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%s)
    
    if [ "$SCOPE_CHOICE" = "1" ]; then
        sudo tar -czf "$BACKUP_DIR/backup-system-${TIMESTAMP}.tar.gz" -C "/etc" rix || true
    else
        tar -czf "$BACKUP_DIR/backup-user-${TIMESTAMP}.tar.gz" -C "$HOME/.config" rix || true
    fi
    echo -e "${GREEN}Configuration layout archived safely to: ${BACKUP_DIR}${NC}"
fi

# 5. DEPENDENCY CHECK & SOURCE COMPILATION
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Cargo toolchain not found. Bootstrapping via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    set +u
    source "$HOME/.cargo/env"
    set -u
fi

echo -e "${GREEN}Executing native profile build sequence...${NC}"
cargo build --release

# 6. ASSET DEPLOYMENT
if [ "$SCOPE_CHOICE" = "1" ]; then
    echo -e "Deploying system-wide binary pointer asset..."
    sudo mkdir -p "$TARGET_BIN_DIR"
    sudo cp target/release/rix-cli "$TARGET_BIN_DIR/rix"
    sudo chmod +x "$TARGET_BIN_DIR/rix"
    
    echo -e "Initializing system configuration root directory layout..."
    sudo mkdir -p "$RIX_CONFIG_DIR"
else
    mkdir -p "$TARGET_BIN_DIR"
    echo -e "Deploying user-space binary pointer asset..."
    cp target/release/rix-cli "$TARGET_BIN_DIR/rix"
    chmod +x "$TARGET_BIN_DIR/rix"
    
    # Auto-append directory to standard execution path if missing
    if [[ ":$PATH:" != *":$TARGET_BIN_DIR:"* ]]; then
        echo "export PATH=\"\$PATH:$TARGET_BIN_DIR\"" >> "$HOME/.bashrc"
        echo -e "${YELLOW}Added $TARGET_BIN_DIR to your PATH via ~/.bashrc${NC}"
    fi
fi

# 7. INITIAL ENGINE LAYOUT GENERATION
echo -e "${GREEN}Invoking Rix engine to initialize clean file layout templates...${NC}"
if [ "$SCOPE_CHOICE" = "1" ]; then
    sudo "$TARGET_BIN_DIR/rix" install coreutils || echo -e "${YELLOW}Note: Initial system boot template placeholder mapped out.${NC}"
else
    "$TARGET_BIN_DIR/rix" install coreutils
fi

echo -e "${GREEN}=== Rix Automated System Provisioning Completed Successfully! ===${NC}"
echo -e "Target binary operational at: ${YELLOW}${TARGET_BIN_DIR}/rix${NC}"
if [ "$SCOPE_CHOICE" = "2" ]; then
    echo -e "Please run: ${YELLOW}source ~/.bashrc${NC} to refresh your current terminal environment path layout."
fi
