#!/bin/bash
# Script to install native plugins for Exodus Browser
# Usage: ./scripts/install-native-plugin.sh <path-to-plugin-dylib>

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check if plugin path is provided
if [ -z "$1" ]; then
    print_error "Plugin path not provided"
    echo "Usage: $0 <path-to-plugin-dylib>"
    echo "Example: $0 examples/native-plugin/target/release/libexodus_example_plugin.dylib"
    exit 1
fi

PLUGIN_PATH="$1"

# Check if plugin file exists
if [ ! -f "$PLUGIN_PATH" ]; then
    print_error "Plugin file not found: $PLUGIN_PATH"
    exit 1
fi

# Detect platform
PLATFORM=$(uname -s)
case "$PLATFORM" in
    Darwin)
        PLUGIN_DIR="$HOME/Library/Application Support/com.exodus.browser/plugins/native"
        PLUGIN_EXT=".dylib"
        ;;
    Linux)
        PLUGIN_DIR="$HOME/.local/share/com.exodus.browser/plugins/native"
        PLUGIN_EXT=".so"
        ;;
    *)
        print_error "Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

# Check if plugin has correct extension
if [[ ! "$PLUGIN_PATH" == *"$PLUGIN_EXT" ]]; then
    print_warning "Plugin file does not have expected extension for $PLATFORM ($PLUGIN_EXT)"
fi

# Create plugin directory if it doesn't exist
echo "Creating plugin directory: $PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR"
print_success "Plugin directory created"

# Get plugin filename
PLUGIN_FILENAME=$(basename "$PLUGIN_PATH")
PLUGIN_DEST="$PLUGIN_DIR/$PLUGIN_FILENAME"

# Copy plugin to destination
echo "Installing plugin: $PLUGIN_FILENAME"
cp "$PLUGIN_PATH" "$PLUGIN_DEST"
print_success "Plugin installed to: $PLUGIN_DEST"

# Verify plugin symbols
echo "Verifying plugin symbols..."
if command -v nm &> /dev/null; then
    SYMBOLS=$(nm -g "$PLUGIN_DEST" | grep exodus_plugin || true)
    if [ -z "$SYMBOLS" ]; then
        print_error "Plugin does not contain required exodus_plugin symbols"
        echo "Required symbols: exodus_plugin_version, exodus_plugin_init, exodus_plugin_deinit"
        exit 1
    fi
    print_success "Plugin symbols verified"
    echo "$SYMBOLS"
else
    print_warning "nm command not found, skipping symbol verification"
fi

# Set appropriate permissions
chmod 644 "$PLUGIN_DEST"
print_success "Plugin permissions set"

# Check if plugin is world-writable (security risk)
if [ "$PLATFORM" = "Linux" ] || [ "$PLATFORM" = "Darwin" ]; then
    PERMS=$(stat -f "%Lp" "$PLUGIN_DEST" 2>/dev/null || stat -c "%a" "$PLUGIN_DEST" 2>/dev/null)
    if [ "$((PERMS & 0002))" -ne 0 ]; then
        print_warning "Plugin is world-writable, this is a security risk"
        chmod o-w "$PLUGIN_DEST"
        print_success "Removed world-writable permission"
    fi
fi

echo ""
print_success "Plugin installation complete!"
echo "Plugin location: $PLUGIN_DEST"
echo ""
echo "To load the plugin in Exodus Browser:"
echo "1. Open Exodus Browser"
echo "2. Navigate to Settings > Native Plugins"
echo "3. Click 'Scan Plugins' or load the plugin manually"
