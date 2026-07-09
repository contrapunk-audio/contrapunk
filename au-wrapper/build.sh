#!/bin/bash
# Build Contrapunk AU (Audio Unit) plugin
#
# Prerequisites: CONTRAPUNK_PLUGIN_UI_DIR=ui/build cargo xtask bundle contrapunk_plugin --release --features embed-ui
# Output: build/Contrapunk.component
#
# Install (automatic with --install):
#   ./build.sh --install
# Validate:
#   auval -v aumi CpHm CpAu

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$SCRIPT_DIR/build"
INSTALL=false

for arg in "$@"; do
    case "$arg" in
        --install) INSTALL=true ;;
    esac
done

# Step 1: Ensure CLAP is built
CLAP_PATH="$PROJECT_ROOT/target/bundled/Contrapunk.clap"
if [ ! -d "$CLAP_PATH" ]; then
    echo "Building CLAP plugin first..."
    cd "$PROJECT_ROOT"
    cargo xtask bundle contrapunk_plugin --release --features embed-ui
fi

# Step 2: CMake configure + build
echo "Configuring AU wrapper..."
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

cmake "$SCRIPT_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_OSX_ARCHITECTURES="arm64;x86_64" \
    -DCLAP_BUNDLE_PATH="$CLAP_PATH"

echo "Building AU wrapper..."
cmake --build . --config Release -j "$(sysctl -n hw.ncpu)"

# Step 3: Sign and report
COMPONENT_PATH=$(find "$BUILD_DIR" -name "Contrapunk.component" -type d | head -1)
if [ -z "$COMPONENT_PATH" ]; then
    echo "Error: Contrapunk.component not found in build output"
    exit 1
fi

echo "Embedding CLAP..."
rm -rf "$COMPONENT_PATH/Contents/PlugIns/Contrapunk.clap"
ditto "$CLAP_PATH" "$COMPONENT_PATH/Contents/PlugIns/Contrapunk.clap"

echo "Signing AU component..."
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:--}"
codesign --force --deep --options runtime -s "$CODESIGN_IDENTITY" "$COMPONENT_PATH"

echo ""
echo "AU plugin built at:"
echo "  $COMPONENT_PATH"

# Step 4: Install if requested
if [ "$INSTALL" = true ]; then
    DEST=~/Library/Audio/Plug-Ins/Components/Contrapunk.component
    rm -rf "$DEST"
    cp -r "$COMPONENT_PATH" "$DEST"
    echo ""
    echo "Installed to: $DEST"
    echo "Validate: auval -v aumi CpHm CpAu"
else
    echo ""
    echo "To install:"
    echo "  cp -r \"$COMPONENT_PATH\" ~/Library/Audio/Plug-Ins/Components/"
    echo "Or re-run with: ./build.sh --install"
fi
