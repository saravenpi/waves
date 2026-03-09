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
    echo "🚀 To create a release asset:"
    echo "   cd target/x86_64-pc-windows-msvc/release"
    echo "   zip waves-windows-x86_64.zip waves.exe"
    echo "   gh release upload vX.Y.Z waves-windows-x86_64.zip --clobber"
else
    echo ""
    echo "❌ Build failed!"
    exit 1
fi
