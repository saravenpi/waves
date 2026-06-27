#!/usr/bin/env bash
set -euo pipefail

# Build the Waves desktop app (Tauri) for macOS, then publish the DMG to the
# website's downloads folder so it can be served like toile does.
#
# Usage:
#   ./release.sh              # build for the host arch (Apple Silicon on this Mac)
#   ./release.sh --intel      # build an Intel (x86_64) DMG instead

DESKTOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DOWNLOADS="$DESKTOP_DIR/../waves-website/static/downloads"

TARGET=""
ARCH_LABEL="aarch64"
if [[ "${1:-}" == "--intel" ]]; then
  TARGET="--target x86_64-apple-darwin"
  ARCH_LABEL="x64"
fi

echo "▶ Building Waves desktop ($ARCH_LABEL)…"
cd "$DESKTOP_DIR"
bun install
bun run tauri build $TARGET

if [[ -n "$TARGET" ]]; then
  DMG_DIR="$DESKTOP_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/dmg"
else
  DMG_DIR="$DESKTOP_DIR/src-tauri/target/release/bundle/dmg"
fi

DMG_SRC="$(ls -t "$DMG_DIR"/*.dmg | head -n 1)"
if [[ -z "$DMG_SRC" ]]; then
  echo "✗ No DMG produced in $DMG_DIR" >&2
  exit 1
fi

mkdir -p "$WEB_DOWNLOADS"
DEST="$WEB_DOWNLOADS/Waves-$ARCH_LABEL.dmg"
cp "$DMG_SRC" "$DEST"

echo "✓ Published $(basename "$DMG_SRC") → $DEST"
echo "  $(du -h "$DEST" | cut -f1)"
echo "  Website button links to /downloads/Waves-$ARCH_LABEL.dmg"
