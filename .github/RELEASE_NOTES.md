# Rix v0.5.0-beta.1 - Reconstruct Raven Beta 1

## Release Date: 2026-07-30

### Features

#### New GUI (rix-gui)
- **Slint-based cross-platform GUI** for graphical package management
- Three main screens: Menu, Packages, History
- Touch-optimized interface (480x800px base resolution)
- Full callback system for user interactions

#### Terminal UI (rix-tui)
- **Ratatui-based terminal interface** for interactive package management
- Menu navigation system
- Package installation/removal screens
- Environment history tracking with rollback
- Non-blocking event handling with proper terminal management

#### CLI Tool (rix)
- Core command-line interface
- Package management commands
- Environment operations
- Git integration

### Supported Platforms

- **Linux** (x86_64)
- **macOS** (x86_64, ARM64/Apple Silicon)
- **Windows** (x86_64)
- **Android** (ARM64, ARMv7, x86_64, x86) - via nix-on-droid

### Build Artifacts

- `rix` - CLI executable
- `rix-tui` - Terminal UI executable
- `rix-gui` - GUI executable (Slint-based)
- Android binaries for all 4 architectures

### Dependencies

- Rust 1.70+
- Edition 2021
- Ratatui 0.28 (TUI)
- Slint 1.7 (GUI)
- Crossterm 0.28 (Terminal handling)
- Tokio 1.0 (Async runtime)

### Known Limitations

- This is a beta release for testing
- Android GUI requires nix-on-droid environment
- Some features may be incomplete

### Testing

To test:

```bash
# CLI
./rix --help

# TUI (requires terminal)
./rix-tui

# GUI (requires display server)
./rix-gui

# Android (requires nix-on-droid)
nix-on-droid shell
chmod +x rix-gui-android-arm64-v8a
./rix-gui-android-arm64-v8a
```

### Contributors

- Claude Haiku 4.5 (Anthropic)

### License

See LICENSE file for details.
