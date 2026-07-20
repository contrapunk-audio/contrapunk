#!/bin/bash
# Build Contrapunk AU (Audio Unit) plugin
#
# Prerequisites: CONTRAPUNK_PLUGIN_UI_DIR=ui/build cargo xtask bundle contrapunk_plugin --release --features embed-ui
# Outputs: build/Contrapunk.component and build/Contrapunk Guitar.component
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

# Step 3: Embed CLAP, sign, and report
COMPONENT_PATH="$BUILD_DIR/Contrapunk.component"
GUITAR_COMPONENT_PATH="$BUILD_DIR/Contrapunk Guitar.component"
for component in "$COMPONENT_PATH" "$GUITAR_COMPONENT_PATH"; do
    if [ ! -d "$component" ]; then
        echo "Error: $component not found in build output"
        exit 1
    fi
    clap_name="$(basename "$component" .component).clap"
    echo "Embedding CLAP in $(basename "$component")..."
    rm -rf "$component/Contents/PlugIns"
    mkdir -p "$component/Contents/PlugIns"
    ditto "$CLAP_PATH" "$component/Contents/PlugIns/$clap_name"
done

echo "Signing AU components..."
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:--}"
codesign --force --deep --options runtime -s "$CODESIGN_IDENTITY" "$COMPONENT_PATH"
codesign --force --deep --options runtime -s "$CODESIGN_IDENTITY" "$GUITAR_COMPONENT_PATH"

echo ""
echo "AU plugins built at:"
echo "  $COMPONENT_PATH"
echo "  $GUITAR_COMPONENT_PATH"

# Step 4: Install if requested
if [ "$INSTALL" = true ]; then
    DEST_DIR="$HOME/Library/Audio/Plug-Ins/Components"
    mkdir -p "$DEST_DIR"
    for component in "$COMPONENT_PATH" "$GUITAR_COMPONENT_PATH"; do
        dest="$DEST_DIR/$(basename "$component")"
        rm -rf "$dest"
        ditto "$component" "$dest"
        echo "Installed to: $dest"
    done
    echo "Validate: auval -v aumi CpHm CpAu"
    echo "Validate: auval -v aumf CpGt CpAu"
else
    echo ""
    echo "To install both components, re-run with: ./build.sh --install"
fi
