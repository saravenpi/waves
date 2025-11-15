# Building and Releasing WAVES

This document explains how to build WAVES binaries for distribution.

## Automated Builds (Recommended)

We use GitHub Actions to automatically build binaries for all platforms (macOS, Linux, Windows).

### Triggering a Build

#### Option 1: Manual Workflow Dispatch
```bash
gh workflow run build-release.yml
```

Then check the status:
```bash
gh run list --workflow=build-release.yml
```

#### Option 2: Create a Git Tag
```bash
git tag v0.1.0
git push origin v0.1.0
```

This will automatically:
1. Build binaries for all platforms
2. Create installers (DMG, tar.gz, zip)
3. Create a GitHub Release with all artifacts

### Downloading Build Artifacts

After a workflow completes successfully, download and update the website binaries:

```bash
./update-website-binaries.sh
```

This script will:
1. Download the latest successful build artifacts
2. Copy them to `installers/`
3. Copy them to `../waves-website/static/downloads/` (if it exists)

Or manually download specific artifacts:
```bash
# Get the run ID
RUN_ID=$(gh run list --workflow=build-release.yml --status=success --limit=1 --json databaseId --jq '.[0].databaseId')

# Download all artifacts
gh run download $RUN_ID

# Or download specific artifact
gh run download $RUN_ID --name waves-macos
```

## Local Builds

### macOS

Build the binary:
```bash
make release
```

Create DMG installer:
```bash
./install-macos.sh
```

### Linux

Build the binary:
```bash
cargo build --release
```

Create tar.gz package:
```bash
mkdir -p installers/linux-package
cp target/release/waves installers/linux-package/
cp waves_logo.png installers/linux-package/
cp install-linux.sh installers/linux-package/
cd installers
tar -czf waves-linux-x86_64.tar.gz linux-package
```

### Windows

On a Windows machine:
```powershell
cargo build --release
```

Create zip package:
```powershell
New-Item -ItemType Directory -Force -Path installers\windows-package
Copy-Item target\release\waves.exe installers\windows-package\
Copy-Item waves_logo.png installers\windows-package\
Copy-Item install-windows.ps1 installers\windows-package\
Compress-Archive -Path installers\windows-package\* -DestinationPath installers\waves-windows-x86_64.zip
```

## Updating the Website

After building new binaries (either via GitHub Actions or locally):

1. Copy binaries to website downloads directory:
```bash
cp installers/waves-*.{dmg,tar.gz,zip} ../waves-website/static/downloads/
```

2. Build and deploy the website:
```bash
cd ../waves-website
bun run build
# Deploy the build/ directory to your hosting service
```

## CI/CD Workflow Details

The GitHub Actions workflow (`.github/workflows/build-release.yml`) does the following:

### macOS Job
- Builds on `macos-latest` runner
- Compiles release binary
- Creates proper macOS app bundle with Info.plist
- Generates DMG using `hdiutil`
- Uploads `waves-macos.dmg` artifact

### Linux Job
- Builds on `ubuntu-latest` runner
- Installs required system dependencies (ALSA, XCB, etc.)
- Compiles release binary
- Creates tar.gz with binary, logo, and install script
- Uploads `waves-linux-x86_64.tar.gz` artifact

### Windows Job
- Builds on `windows-latest` runner
- Compiles release binary (waves.exe)
- Creates zip package with executable, logo, and install script
- Uploads `waves-windows-x86_64.zip` artifact

### Release Job
- Only runs when a tag is pushed (e.g., `v0.1.0`)
- Downloads all platform artifacts
- Creates GitHub Release with all binaries attached

## Version Management

To release a new version:

1. Update version in `Cargo.toml`:
```toml
[package]
version = "0.2.0"
```

2. Update version in workflow Info.plist (`.github/workflows/build-release.yml`)

3. Commit and tag:
```bash
git add Cargo.toml .github/workflows/build-release.yml
git commit -m "🔖 Release v0.2.0"
git tag v0.2.0
git push origin master
git push origin v0.2.0
```

4. GitHub Actions will automatically build and create the release

5. Download artifacts and update website:
```bash
./update-website-binaries.sh
```

## Troubleshooting

### Workflow Fails on Windows
- Check that all Rust dependencies support Windows
- Ensure no Unix-specific code in the codebase
- Check the workflow logs for specific errors

### Artifacts Not Uploading
- Verify artifact paths in workflow YAML
- Check that files exist after build steps
- Review GitHub Actions logs for permissions issues

### DMG Creation Fails
- Ensure app bundle structure is correct
- Verify Info.plist syntax
- Check that binary is executable

### Website Downloads Not Working
- Verify files are in `static/downloads/` directory
- Check file permissions
- Rebuild website after updating binaries
- Verify deployment includes static files
