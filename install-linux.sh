#!/bin/bash
set -e

echo "Building WAVES for Linux..."
cargo build --release

echo "Installing WAVES..."

# Create desktop entry directory if it doesn't exist
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"

# Install binary
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/waves "$INSTALL_DIR/waves"
chmod +x "$INSTALL_DIR/waves"

# Create desktop entry
cat > "$DESKTOP_DIR/waves.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=WAVES
Comment=GUI music player with Miller column browser
Exec=$INSTALL_DIR/waves %F
Icon=$PWD/assets/waves_logo.png
Terminal=false
Categories=AudioVideo;Audio;Player;
MimeType=audio/mpeg;audio/x-wav;audio/flac;audio/ogg;audio/mp4;
EOF

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR"
fi

echo "WAVES has been installed successfully!"
echo "Binary location: $INSTALL_DIR/waves"
echo "Desktop entry: $DESKTOP_DIR/waves.desktop"
echo ""
echo "Make sure $INSTALL_DIR is in your PATH."
echo "You can add it by running:"
echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
echo "  source ~/.bashrc"
