# Rix Android Releases

This guide explains how to download and test the Rix Android builds.

## Download Links

Android APK binaries are automatically built and released to GitHub. Once a beta tag is pushed, GitHub Actions will compile the binaries for all supported architectures.

### Latest Beta Release
- **Branch**: `claude-slint-android`
- **Tag**: `v0.5.0-android-beta`
- **GitHub Releases**: https://github.com/Zexolver/rix/releases/tag/v0.5.0-android-beta

### Supported Architectures

| Architecture | Device Type | File | Download |
|---|---|---|---|
| **ARM64** | Most modern Android devices (2015+) | `rix-gui-android-arm64-v8a` | [Download](https://github.com/Zexolver/rix/releases/download/v0.5.0-android-beta/rix-gui-android-arm64-v8a) |
| **ARMv7** | Older Android devices (2010-2015) | `rix-gui-android-armeabi-v7a` | [Download](https://github.com/Zexolver/rix/releases/download/v0.5.0-android-beta/rix-gui-android-armeabi-v7a) |
| **x86_64** | Android emulators, tablets | `rix-gui-android-x86_64` | [Download](https://github.com/Zexolver/rix/releases/download/v0.5.0-android-beta/rix-gui-android-x86_64) |
| **x86** | Older x86 devices | `rix-gui-android-x86` | [Download](https://github.com/Zexolver/rix/releases/download/v0.5.0-android-beta/rix-gui-android-x86) |

## Testing with nix-on-droid

### Prerequisites
- Android device with [nix-on-droid](https://github.com/nix-community/nix-on-droid) installed
- USB cable or ADB over network
- 200MB free storage space

### Installation Steps

1. **Determine Your Device Architecture**
   ```bash
   uname -m
   # Output will be: aarch64 (ARM64), armv7l (ARMv7), x86_64, or i686
   ```

2. **Download the Appropriate Binary**
   - For **ARM64** (aarch64): Download `rix-gui-android-arm64-v8a`
   - For **ARMv7** (armv7l): Download `rix-gui-android-armeabi-v7a`
   - For **x86_64**: Download `rix-gui-android-x86_64`
   - For **x86**: Download `rix-gui-android-x86`

3. **Transfer to Your Device**
   ```bash
   # Via ADB
   adb push rix-gui-android-arm64-v8a /data/local/tmp/

   # Or manually via file explorer
   # Place the binary in your Downloads or Documents folder
   ```

4. **Make the Binary Executable**
   ```bash
   nix-on-droid shell
   cd /data/local/tmp  # or wherever you placed the file
   chmod +x rix-gui-android-arm64-v8a
   ```

5. **Run the Rix GUI**
   ```bash
   ./rix-gui-android-arm64-v8a
   ```

## GitHub Actions Automation

When a new beta tag is pushed to `claude-slint-android`, GitHub Actions automatically:

1. **Compiles** the Rust GUI for all 4 Android architectures
2. **Strips** debug symbols to reduce file size
3. **Uploads** binaries to GitHub Releases
4. **Creates** a beta release with documentation

### Build Triggers

- Tags matching `v*-beta*` (e.g., `v0.5.0-android-beta`)
- Tags matching `Reconstruct Raven*` (for your Raven releases)
- Manual workflow dispatch via GitHub UI

## Workflow Status

Check the build status on GitHub:
- **Workflow File**: `.github/workflows/android-build.yml`
- **Release Workflow**: `.github/workflows/release.yml` (build-android-release job)
- **Actions Page**: https://github.com/Zexolver/rix/actions

## Troubleshooting

### Binary Won't Execute
```bash
# Ensure it has execute permissions
chmod +x rix-gui-android-arm64-v8a

# Verify it's a valid binary
file rix-gui-android-arm64-v8a
# Should output: ELF 64-bit LSB shared object, ARM aarch64
```

### Wrong Architecture
```bash
# Verify your device's native architecture
uname -m
# aarch64 = ARM64 (most common)
# armv7l = ARMv7
# x86_64 = x86_64
# i686 = x86
```

### Display Issues
- Ensure you're running inside nix-on-droid environment
- Set display environment if needed:
  ```bash
  export DISPLAY=:0
  ```

## Building Locally

To build the Android binaries locally:

```bash
# Install cargo-ndk
cargo install cargo-ndk

# Build for ARM64
cargo ndk -t arm64-v8a -o ./android-build build --release --bin rix-gui

# Binary will be at:
./android-build/arm64-v8a/release/rix-gui
```

## Features

The Rix Android GUI includes:

- **Package Management** - View and install packages
- **Environment History** - Track environment changes
- **Rollback Support** - Revert to previous states
- **Cross-Platform UI** - Slint-based interface works on Android, Linux, macOS, Windows

## Reporting Issues

Found a bug? Please report it:
- **GitHub Issues**: https://github.com/Zexolver/rix/issues
- **Include**: Device model, Android version, architecture, error messages

## Release Notes

See [CHANGELOG.md](./CHANGELOG.md) for detailed release notes and features.

---

**Last Updated**: 2026-07-30
**Status**: Beta - Testing Phase
**Supported Devices**: Android 9+ (API level 28+)
