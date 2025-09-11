#!/bin/bash
# Generate SHA256 checksums for Tauri release artifacts
# Usage: ./generate-checksums.sh [platform]

set -e

PLATFORM=${1:-"all"}

cd src-tauri/target/release/bundle

echo "Generating SHA256 checksums for platform: $PLATFORM"

case "$PLATFORM" in
    "macos")
        find . -type f \( -name "*.dmg" -o -name "*.app.tar.gz" -o -name "*.app.tar.gz.sig" \) -exec sha256sum {} \;
        ;;
    "windows")
        find . -type f \( -name "*.exe" -o -name "*.msi" -o -name "*.nsis.zip" \) -exec sha256sum {} \;
        ;;
    "linux")
        find . -type f \( -name "*.AppImage" -o -name "*.deb" -o -name "*.rpm" \) -exec sha256sum {} \;
        ;;
    "all")
        find . -type f \( -name "*.dmg" -o -name "*.exe" -o -name "*.AppImage" -o -name "*.deb" -o -name "*.msi" -o -name "*.app.tar.gz" \) -exec sha256sum {} \;
        ;;
    *)
        echo "Unknown platform: $PLATFORM"
        echo "Supported platforms: macos, windows, linux, all"
        exit 1
        ;;
esac > checksums.sha256

echo "Checksums generated and saved to checksums.sha256"
echo "File contents:"
cat checksums.sha256
