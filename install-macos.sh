#!/bin/bash

set -e

echo "🎵 Installing Waves music player..."

# Build release version
echo "📦 Building release version..."
cargo build --release

echo ""
echo "🏗️  Creating macOS application bundle..."
rm -rf macos-bundle/Waves.app/Contents/MacOS
mkdir -p macos-bundle/Waves.app/Contents/MacOS
cp target/release/waves macos-bundle/Waves.app/Contents/MacOS/waves
cp waves-launcher.sh macos-bundle/Waves.app/Contents/MacOS/waves-launcher
chmod +x macos-bundle/Waves.app/Contents/MacOS/waves
chmod +x macos-bundle/Waves.app/Contents/MacOS/waves-launcher

echo "📋 Installing Waves.app to /Applications..."
rm -rf /Applications/Waves.app
cp -R macos-bundle/Waves.app /Applications/

echo "✍️  Code signing app..."
codesign --deep --force --sign - /Applications/Waves.app 2>/dev/null || echo "⚠️  Code signing failed (continuing anyway)"

echo "🔓 Removing quarantine attribute..."
xattr -cr /Applications/Waves.app

echo "📝 Registering with macOS..."
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f /Applications/Waves.app

echo "✅ Installation complete!"
echo ""
echo "🎵 Waves is now installed at /Applications/Waves.app"
echo ""
echo "You can now:"
echo "  • Launch from Spotlight (Cmd+Space, type 'Waves')"
echo "  • Launch from Applications folder"
echo "  • Open audio files with Waves from Finder"
echo ""
echo "Enjoy your music! 🎶"
