# Release Process

## Prerequisites

- [ ] GitHub repository write access
- [ ] Code signing certificate (optional, for Windows)
- [ ] Minisign keypair for auto-updater: `tauri signer generate -w ~/.tauri/isolate.key`
- [ ] Rust toolchain: `rustup update stable`
- [ ] Node.js + pnpm installed

## Version Bump

Update version in **three files** (must match):

```bash
# 1. src-tauri/Cargo.toml
version = "X.Y.Z"

# 2. src-tauri/tauri.conf.json
"version": "X.Y.Z"

# 3. package.json
"version": "X.Y.Z"
```

## Build Process

```bash
# Install dependencies
pnpm install

# Build release (creates installer in src-tauri/target/release/bundle/)
pnpm tauri build

# Build with signing (requires TAURI_SIGNING_PRIVATE_KEY env)
TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/isolate.key) pnpm tauri build
```

**Output locations:**
- `src-tauri/target/release/bundle/msi/` — MSI installer
- `src-tauri/target/release/bundle/nsis/` — NSIS installer (.exe)

## Testing

Before release:
- [ ] `pnpm check` — TypeScript/Svelte type checking
- [ ] `pnpm test` — Unit tests
- [ ] Manual testing on clean Windows VM
- [ ] Test auto-update from previous version (if applicable)
- [ ] Verify all strategies work (Zapret, VLESS)

## Release Checklist

- [ ] Version bumped in all 3 files
- [ ] CHANGELOG updated (if maintained)
- [ ] All tests pass
- [ ] Build completes without errors
- [ ] Installer tested on clean system
- [ ] No debug code or console.log statements
- [ ] Git tag created: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`

## Publishing

1. **Create GitHub Release:**
   ```bash
   git push origin main
   git push origin vX.Y.Z
   ```

2. **Upload artifacts to GitHub Release:**
   - `Isolate_X.Y.Z_x64-setup.exe` (NSIS)
   - `Isolate_X.Y.Z_x64_en-US.msi` (MSI)
   - `latest.json` (for auto-updater)

3. **Generate `latest.json` for updater:**
   ```json
   {
     "version": "X.Y.Z",
     "notes": "Release notes here",
     "pub_date": "2024-01-01T00:00:00Z",
     "platforms": {
       "windows-x86_64": {
         "signature": "SIGNATURE_FROM_BUILD",
         "url": "https://github.com/.../releases/download/vX.Y.Z/Isolate_X.Y.Z_x64-setup.exe"
       }
     }
   }
   ```

## Post-Release

- [ ] Verify download links work
- [ ] Test auto-update on existing installation
- [ ] Announce release (if applicable)
- [ ] Monitor for crash reports / issues
- [ ] Bump version to next dev version (optional)
