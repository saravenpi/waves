#!/bin/bash

set -e

echo "🎵 Installing Waves music player..."

# Build release version
echo "📦 Building release version..."
make release

# Create app bundle structure
APP_DIR="/Applications/Waves.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "🏗️  Creating app bundle structure..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy binary and launcher
echo "📋 Copying binary and launcher..."
cp waves "$MACOS_DIR/waves"
cp waves-launcher.sh "$MACOS_DIR/waves-launcher"
chmod +x "$MACOS_DIR/waves"
chmod +x "$MACOS_DIR/waves-launcher"

# Copy Info.plist
echo "⚙️  Copying Info.plist..."
cp Info.plist "$CONTENTS_DIR/"

# Convert PNG to ICNS
echo "🎨 Creating app icon..."
if command -v sips &> /dev/null && command -v iconutil &> /dev/null; then
    ICONSET_DIR="$RESOURCES_DIR/waves.iconset"
    mkdir -p "$ICONSET_DIR"

    # Generate all required icon sizes
    sips -z 16 16     waves_logo.png --out "$ICONSET_DIR/icon_16x16.png" > /dev/null 2>&1
    sips -z 32 32     waves_logo.png --out "$ICONSET_DIR/icon_16x16@2x.png" > /dev/null 2>&1
    sips -z 32 32     waves_logo.png --out "$ICONSET_DIR/icon_32x32.png" > /dev/null 2>&1
    sips -z 64 64     waves_logo.png --out "$ICONSET_DIR/icon_32x32@2x.png" > /dev/null 2>&1
    sips -z 128 128   waves_logo.png --out "$ICONSET_DIR/icon_128x128.png" > /dev/null 2>&1
    sips -z 256 256   waves_logo.png --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null 2>&1
    sips -z 256 256   waves_logo.png --out "$ICONSET_DIR/icon_256x256.png" > /dev/null 2>&1
    sips -z 512 512   waves_logo.png --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null 2>&1
    sips -z 512 512   waves_logo.png --out "$ICONSET_DIR/icon_512x512.png" > /dev/null 2>&1
    sips -z 1024 1024 waves_logo.png --out "$ICONSET_DIR/icon_512x512@2x.png" > /dev/null 2>&1

    # Convert to ICNS
    iconutil -c icns "$ICONSET_DIR" -o "$RESOURCES_DIR/waves.icns"
    rm -rf "$ICONSET_DIR"
else
    echo "⚠️  Warning: sips or iconutil not found, skipping icon creation"
fi

# Code sign the app (ad-hoc signature)
echo "✍️  Code signing app..."
codesign --deep --force --sign - "$APP_DIR" 2>/dev/null || echo "⚠️  Code signing failed (continuing anyway)"

# Remove quarantine attribute
echo "🔓 Removing quarantine attribute..."
xattr -cr "$APP_DIR"

# Register with macOS
echo "📝 Registering with macOS..."
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP_DIR"

# Touch to update modification date
touch "$APP_DIR"

echo "✅ Installation complete!"
echo ""
echo "🎵 Waves is now installed at /Applications/Waves.app"
echo ""
echo "You can now:"
echo "  • Launch from Spotlight (Cmd+Space, type 'Waves')"
echo "  • Launch from Applications folder"
echo "  • Open audio files with Waves from Finder"
echo "  • Run 'waves' command from terminal (if in PATH)"
echo ""
echo "Enjoy your music! 🎶"
