#!/usr/bin/env bash

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Starting Rix Multi-Platform Core Provisioning Layer ===${NC}"

# 0. REPOSITORY ROOT VERIFICATION
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: install.sh must be executed from the root directory of the 'rix' repository.${NC}"
    exit 1
fi

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
        echo -e "${RED}Error: The 'nix' command is missing on a NixOS target. Your system path profile layout might be broken.${NC}\n"
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
echo -e "1) User-Space   (Config: \$HOME/.config/rix, Binary: \$HOME/.local/bin)"
echo -e "2) System-Wide  (Config: /etc/rix, Binary: /usr/local/bin, requires sudo)"
read -rp "Enter choice [1-2]: " SCOPE_CHOICE

if [ "$SCOPE_CHOICE" = "1" ]; then
    RIX_CONFIG_DIR="$HOME/.config/rix"
    TARGET_BIN_DIR="$HOME/.local/bin"
else
    RIX_CONFIG_DIR="/etc/rix"
    TARGET_BIN_DIR="/usr/local/bin"
fi

# 4. PRE-EXISTING CONFIGURATION BACKUP
if [ -d "$RIX_CONFIG_DIR" ]; then
    echo -e "${YELLOW}Warning: Pre-existing configuration profile layout tracked at ${RIX_CONFIG_DIR}.${NC}"
    BACKUP_DIR="$HOME/Documents/rix-archived-profiles"
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%s)
      
    if [ "$SCOPE_CHOICE" = "1" ]; then
        tar -czf "$BACKUP_DIR/backup-user-${TIMESTAMP}.tar.gz" -C "$HOME/.config" rix || true
    else
        sudo tar -czf "$BACKUP_DIR/backup-system-${TIMESTAMP}.tar.gz" -C "/etc" rix || true
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

BAR_BLOCK="========================================"
SPACE_BLOCK="                                        "
START_TIME=$(date +%s)

# Temporarily suspend instant-exit so a Cargo syntax error doesn't silently kill the script
set +e   

# Force progress output, PROVIDE A DUMMY WIDTH, pipe standard error, and convert \r to \n
env CARGO_TERM_PROGRESS_WHEN=always CARGO_TERM_PROGRESS_WIDTH=80 cargo build --release --bin rix-cli 2>&1 | tr '\r' '\n' | while IFS= read -r line; do
      
    # Strip terminal color formatting codes for clean regex matching
    CLEAN_LINE=$(echo "$line" | sed 's/\x1b\[[0-9;]*m//g')

    # Match the cargo progress pattern: e.g., "Building [==>] 37/185"
    if [[ "$CLEAN_LINE" =~ ([0-9]+)/([0-9]+) ]]; then
        CURRENT="${BASH_REMATCH[1]}"
        TOTAL="${BASH_REMATCH[2]}"
         
        if [ "$TOTAL" -eq 0 ] || [ "$CURRENT" -gt "$TOTAL" ]; then
            continue
        fi
         
        PERCENT=$(( CURRENT * 100 / TOTAL ))
         
        BAR_WIDTH=30
        FILLED=$(( PERCENT * BAR_WIDTH / 100 ))
        UNFILLED=$(( BAR_WIDTH - FILLED ))
         
        BAR_STR="${BAR_BLOCK:0:$FILLED}"
        SPACE_STR="${SPACE_BLOCK:0:$UNFILLED}"
         
        NOW=$(date +%s)
        ELAPSED=$(( NOW - START_TIME ))
         
        if [ "$CURRENT" -gt 0 ] && [ "$ELAPSED" -gt 0 ]; then
            TOTAL_ESTIMATED_TIME=$(( ELAPSED * TOTAL / CURRENT ))
            REMAINING_TIME=$(( TOTAL_ESTIMATED_TIME - ELAPSED ))
             
            if [ "$REMAINING_TIME" -ge 60 ]; then
                ETA_STR="$(( REMAINING_TIME / 60 ))m $(( REMAINING_TIME % 60 ))s remaining"
            else
                ETA_STR="${REMAINING_TIME}s remaining"
            fi
        else
            ETA_STR="Calculating ETA..."
        fi
         
        printf "\r\033[K${BLUE}🛠  Compiling [${GREEN}${BAR_STR}>${SPACE_STR}${BLUE}] ${YELLOW}%d/%d${NC} (%d%%) | %s" "$CURRENT" "$TOTAL" "$PERCENT" "$ETA_STR"
     
    # Catch clean "Finished" states if the project is already cached and compiled
    elif [[ "$CLEAN_LINE" == *"Finished release"* ]] || [[ "$CLEAN_LINE" == *"error:"* ]]; then
        printf "\r\033[K${GREEN}%s${NC}\n" "$CLEAN_LINE"
    fi
done

# Grab Cargo's exit status from the first item in the pipeline
CARGO_STATUS=${PIPESTATUS[0]}

# Re-enable strict error handling
set -e

if [ "$CARGO_STATUS" -ne 0 ]; then
    echo -e "\n${RED}Error: Native compilation sequence dropped with an unhandled exit code (${CARGO_STATUS}).${NC}"
    echo -e "${YELLOW}Tip: Run 'cargo build --release' manually to see the raw compiler error.${NC}"
    exit "$CARGO_STATUS"
fi

echo "" # Flush the progress bar output safely

# 6. ASSET DEPLOYMENT
if [ "$SCOPE_CHOICE" = "1" ]; then
    mkdir -p "$TARGET_BIN_DIR"
    echo -e "Deploying user-space binary pointer asset..."
    cp target/release/rix-cli "$TARGET_BIN_DIR/rix"
    chmod +x "$TARGET_BIN_DIR/rix"
       
    # Auto-append directory to standard execution path if missing
    if [[ ":$PATH:" != *":$TARGET_BIN_DIR:"* ]]; then
        echo "export PATH=\"\$PATH:$TARGET_BIN_DIR\"" >> "$HOME/.bashrc"
        echo -e "${YELLOW}Added $TARGET_BIN_DIR to your PATH via ~/.bashrc${NC}"
    fi
else
    echo -e "Deploying system-wide binary pointer asset..."
    sudo mkdir -p "$TARGET_BIN_DIR"
    sudo cp target/release/rix-cli "$TARGET_BIN_DIR/rix"
    sudo chmod +x "$TARGET_BIN_DIR/rix"
       
    echo -e "Initializing system configuration root directory layout..."
    sudo mkdir -p "$RIX_CONFIG_DIR"
fi

# 7. INITIAL ENGINE LAYOUT GENERATION
echo -e "${GREEN}Invoking Rix engine to initialize clean file layout templates...${NC}"
if [ "$SCOPE_CHOICE" = "1" ]; then
    "$TARGET_BIN_DIR/rix" init
    "$TARGET_BIN_DIR/rix" install coreutils
else
    sudo "$TARGET_BIN_DIR/rix" init
    sudo "$TARGET_BIN_DIR/rix" install coreutils || echo -e "${YELLOW}Note: Initial system boot template placeholder mapped out.${NC}"
    
    # Permission Fix: If run as sudo, rix creates local ~/.config/rix/rix.toml files as root.
    # Reassign ownership back to the standard user to prevent future Permission Denied errors.
    if [ -d "$HOME/.config/rix" ]; then
        sudo chown -R "$USER:$USER" "$HOME/.config/rix" 2>/dev/null || true
    fi
fi

echo -e "\n${GREEN}=== Rix Automated System Provisioning Completed Successfully! ===${NC}"
echo -e "Target binary operational at: ${YELLOW}${TARGET_BIN_DIR}/rix${NC}"
if [ "$SCOPE_CHOICE" = "1" ]; then
    echo -e "Please run: ${YELLOW}source ~/.bashrc${NC} to refresh your current terminal environment path layout."
fi
