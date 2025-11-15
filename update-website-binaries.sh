#!/bin/bash
# Script to update website binaries after GitHub Actions build completes
# This script downloads the latest artifacts from GitHub Actions and updates the website

set -e

echo "🔍 Fetching latest workflow run..."

# Get the latest successful workflow run
RUN_ID=$(gh run list --workflow=build-release.yml --status=success --limit=1 --json databaseId --jq '.[0].databaseId')

if [ -z "$RUN_ID" ]; then
    echo "❌ No successful workflow runs found"
    echo "Run the workflow first with: gh workflow run build-release.yml"
    exit 1
fi

echo "✅ Found workflow run: $RUN_ID"
echo "📥 Downloading artifacts..."

# Create temp directory for downloads
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Download artifacts
gh run download "$RUN_ID"

echo "📦 Downloaded artifacts to $TEMP_DIR"
ls -lh

# Copy to installers directory
echo "📁 Updating installers directory..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALLERS_DIR="$SCRIPT_DIR/installers"

mkdir -p "$INSTALLERS_DIR"

cp waves-macos/waves-macos.dmg "$INSTALLERS_DIR/"
cp waves-linux/waves-linux-x86_64.tar.gz "$INSTALLERS_DIR/"
cp waves-windows/waves-windows-x86_64.zip "$INSTALLERS_DIR/"

echo "✅ Updated installers directory"
ls -lh "$INSTALLERS_DIR"

# Copy to website if it exists
WEBSITE_DIR="$SCRIPT_DIR/../waves-website/static/downloads"
if [ -d "$WEBSITE_DIR" ]; then
    echo "🌐 Updating website downloads directory..."
    mkdir -p "$WEBSITE_DIR"
    cp waves-macos/waves-macos.dmg "$WEBSITE_DIR/"
    cp waves-linux/waves-linux-x86_64.tar.gz "$WEBSITE_DIR/"
    cp waves-windows/waves-windows-x86_64.zip "$WEBSITE_DIR/"
    echo "✅ Updated website downloads"
    ls -lh "$WEBSITE_DIR"
else
    echo "ℹ️  Website directory not found at $WEBSITE_DIR"
fi

# Cleanup
cd - > /dev/null
rm -rf "$TEMP_DIR"

echo ""
echo "🎉 All binaries updated successfully!"
echo ""
echo "File sizes:"
ls -lh "$INSTALLERS_DIR"/waves-*
echo ""
echo "Next steps:"
echo "1. Test the binaries to ensure they work correctly"
echo "2. Commit the updated installers if needed"
echo "3. Deploy the website with the new downloads"
