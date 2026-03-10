#!/bin/bash

set -e

echo "🎵 Installing Waves Music Player for macOS..."

if [ ! -f "target/release/waves" ]; then
    echo "❌ Release binary not found. Building now..."
    cargo build --release
fi

APP_NAME="Waves.app"
APP_PATH="/Applications/$APP_NAME"
BUNDLE_PATH="macos-bundle"

echo "📦 Creating application bundle..."

if [ -d "$APP_PATH" ]; then
    echo "🗑️  Removing existing Waves.app..."
    rm -rf "$APP_PATH"
fi

mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

echo "📋 Copying binary..."
cp target/release/waves "$APP_PATH/Contents/MacOS/waves"
chmod +x "$APP_PATH/Contents/MacOS/waves"

if [ -f "$BUNDLE_PATH/Info.plist" ]; then
    echo "📋 Copying Info.plist..."
    cp "$BUNDLE_PATH/Info.plist" "$APP_PATH/Contents/Info.plist"
else
    echo "📝 Creating Info.plist..."
    cat > "$APP_PATH/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>waves</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.waves.player</string>
    <key>CFBundleName</key>
    <string>Waves</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.5.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>mp3</string>
                <string>wav</string>
                <string>flac</string>
                <string>ogg</string>
                <string>m4a</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>Audio File</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Default</string>
        </dict>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>*</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>Folder</string>
            <key>CFBundleTypeOSTypes</key>
            <array>
                <string>fold</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Default</string>
        </dict>
    </array>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF
fi

if [ -f "$BUNDLE_PATH/AppIcon.icns" ]; then
    echo "🎨 Copying icon..."
    cp "$BUNDLE_PATH/AppIcon.icns" "$APP_PATH/Contents/Resources/AppIcon.icns"
fi

echo "✅ Waves has been installed to /Applications/Waves.app"
echo ""
echo "🎉 Installation complete!"
echo ""
echo "You can now:"
echo "  • Launch Waves from your Applications folder"
echo "  • Open audio files directly with Waves from Finder"
echo "  • Find Waves in Spotlight search"
echo ""
echo "Enjoy your music! 🎶"
