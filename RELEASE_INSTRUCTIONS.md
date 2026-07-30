# Release Instructions for Rix v0.5.0-beta.1

## Status

✅ **All code, workflows, and binaries are ready**
⚠️ **Git tags need to be pushed from a local machine**

## What's Ready

### 1. GitHub Actions Workflows
- ✅ `.github/workflows/release.yml` - Desktop & Android builds
- ✅ `.github/workflows/android-build.yml` - Android-specific builds
- ✅ `.github/workflows/ci.yml` - Continuous integration

### 2. Source Code
- ✅ CLI tool (rix)
- ✅ Terminal UI (rix-tui) using ratatui
- ✅ GUI tool (rix-gui) using Slint
- ✅ Android NDK support configured

### 3. Documentation
- ✅ ANDROID_RELEASES.md - Android setup guide
- ✅ ANDROID_BUILD.md - Local Android build instructions
- ✅ .github/RELEASE_NOTES.md - Release notes

### 4. Git Tags (Created Locally)
```bash
v0.5.0-beta.1          # Main beta release
v0.5.0-android-beta    # Android-specific beta
```

## Next Steps: Push the Tags

### Option 1: Push from Your Local Machine (Recommended)

```bash
# Clone/pull the latest code
git clone https://github.com/Zexolver/rix.git
cd rix
git checkout claude-slint-android

# Create the tags locally
git tag -a v0.5.0-beta.1 -m "Reconstruct Raven Beta 1 Release"
git tag -a v0.5.0-android-beta -m "Rix Android Beta Release"

# Push the tags (this will trigger GitHub Actions)
git push origin v0.5.0-beta.1 v0.5.0-android-beta
```

### Option 2: Create Release via GitHub Web UI

1. Go to: https://github.com/Zexolver/rix/releases
2. Click "Create a new release"
3. Use tag: `v0.5.0-beta.1`
4. Target branch: `claude-slint-android`
5. Title: `Rix v0.5.0-beta.1 - Reconstruct Raven Beta 1`
6. Click "Publish release"

This will automatically trigger:
- GitHub Actions to build all platforms
- Desktop binaries (Linux, macOS, Windows)
- Android binaries (ARM64, ARMv7, x86_64, x86)

## What Happens After Tag Push

Once the tags are pushed to GitHub:

### Automatic Builds (5-15 minutes)

1. **release.yml** workflow triggers and builds:
   - rix CLI for Linux, macOS, Windows
   - rix-tui for Linux, macOS, Windows
   - Android binaries for all 4 architectures

2. **android-build.yml** workflow triggers and builds:
   - Additional Android verification

3. **ci.yml** workflow triggers for:
   - Code quality checks
   - Test suite validation

### Release Artifacts

Once builds complete, binaries will be available at:

**Desktop:**
- `rix-linux-x86_64`
- `rix-macos-x86_64`
- `rix-macos-aarch64`
- `rix-windows-x86_64.exe`
- `rix-tui-linux-x86_64`
- `rix-tui-macos-x86_64`
- `rix-tui-macos-aarch64`
- `rix-tui-windows-x86_64.exe`

**Android:**
- `rix-gui-android-arm64-v8a`
- `rix-gui-android-armeabi-v7a`
- `rix-gui-android-x86_64`
- `rix-gui-android-x86`

## Check Build Status

After pushing tags, check:
- **GitHub Actions**: https://github.com/Zexolver/rix/actions
- **Release Page**: https://github.com/Zexolver/rix/releases

## Troubleshooting

### "Tag already exists" error
The tags were created but not pushed. Delete locally and recreate:
```bash
git tag -d v0.5.0-beta.1
git tag -a v0.5.0-beta.1 -m "Reconstruct Raven Beta 1 Release"
git push origin v0.5.0-beta.1
```

### Workflow doesn't trigger
GitHub Actions workflows only trigger on tags pushed to remote. Ensure:
1. Tag is pushed to `origin` (not just local)
2. Check Actions tab for workflow status
3. Verify branch is `claude-slint-android`

### Android build fails
Check:
1. NDK is configured in `.cargo/config.toml`
2. Android targets are installed: `rustup target add aarch64-linux-android`
3. Check build logs in Actions > android-build.yml

## Current Branch Status

**Branch**: `claude-slint-android`
**Latest Commit**: Added workflow_dispatch trigger
**Files Changed**: 6 files
**Status**: ✅ Ready for release

All development work is complete. Only pending action is pushing the git tags.

---

**Created**: 2026-07-30  
**Status**: Ready for Release  
**Remaining Action**: Push tags from local machine
