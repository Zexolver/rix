#!/usr/bin/env bash

set -euo pipefail

# Visual Style Parameters
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Starting Rix Multi-Platform Core Provisioning Layer ===${NC}"

# 1. System Architecture Live Identification
ARCH=$(uname -m)
OS=$(uname -s)

echo -e "Host System Environment Detected: ${YELLOW}${OS} (${ARCH})${NC}"

# 2. Check for Active Pre-Existing Configuration Bloat
RIX_TARGET_DIR="$HOME/.config/rix"
if [ -d "$RIX_TARGET_DIR" ]; then
    echo -e "${YELLOW}Warning: Pre-existing configuration profile layout tracked at ${RIX_TARGET_DIR}${NC}"
    echo -e "Creating local archival package container before fresh initialization..."
    mkdir -p "$HOME/Documents/rix-archived-profiles"
    tar -czf "$HOME/Documents/rix-archived-profiles/backup-$(date +%s).tar.gz" -C "$HOME/.config" rix || true
fi

# 3. Dynamic Toolchain Resolution and Compilation Verification
echo -e "${BLUE}Executing live dependency verification matrix...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Cargo toolchain not found. Attempting local compilation channel bootstrap...${NC}"
    # This provides a non-interactive fallthrough for minimalist setups
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 4. Native In-Situ Optimization Compilation Block
echo -e "${GREEN}Executing native profile build sequence for host target framework...${NC}"
cargo build --release

# 5. Provisioning Local Binary Destination Paths
LOCAL_BIN_DIR="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN_DIR"

echo -e "Registering binary asset pointers to target path space: ${LOCAL_BIN_DIR}/rix"
cp target/release/rix-cli "$LOCAL_BIN_DIR/rix"
chmod +x "$LOCAL_BIN_DIR/rix"

# 6. Kick Off Core Structural Layout Skeletons Initialization
echo -e "${GREEN}Invoking Rix engine to deploy pristine configuration templates...${NC}"
"$LOCAL_BIN_DIR/rix" profile add core-utils --group upstream || true

echo -e "${GREEN}=== Rix Automated System Provisioning Completed Successfully! ===${NC}"
echo -e "Verify your local environment path configuration updates include: ${YELLOW}${LOCAL_BIN_DIR}${NC}"
