#!/bin/bash
set -e

echo "🪟 Cross-compiling WAVES for Windows from macOS..."
echo ""

cargo xwin build --release --target x86_64-pc-windows-msvc

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📦 Windows binary location:"
    echo "   target/x86_64-pc-windows-msvc/release/waves.exe"
    echo ""
    echo "📊 Binary size:"
    ls -lh target/x86_64-pc-windows-msvc/release/waves.exe | awk '{print "   " $5}'
    echo ""

    echo "📦 Packaging Windows binary..."
    cd target/x86_64-pc-windows-msvc/release
    zip -q waves-windows-x86_64.zip waves.exe
    cd ../../..
    echo "✅ Package created: target/x86_64-pc-windows-msvc/release/waves-windows-x86_64.zip"
    echo ""

    if [ "$1" = "--upload" ]; then
        if [ -z "$2" ]; then
            echo "❌ Please specify a release tag (e.g., v0.5.0)"
            echo "Usage: ./build-windows.sh --upload v0.5.0"
            exit 1
        fi

        echo "⬆️  Uploading to release $2..."
        gh release upload "$2" target/x86_64-pc-windows-msvc/release/waves-windows-x86_64.zip --clobber
        echo "🎉 Upload complete!"
    else
        echo "💡 To upload to a release, run:"
        echo "   ./build-windows.sh --upload vX.Y.Z"
    fi
else
    echo ""
    echo "❌ Build failed!"
    exit 1
fi
