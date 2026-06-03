#!/bin/bash
# Script to test native plugin integration with Exodus Browser
# This script helps verify the plugin system is working correctly

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

echo "======================================"
echo "Exodus Browser Plugin Integration Test"
echo "======================================"
echo ""

# Check if plugin is installed
PLUGIN_DIR="$HOME/Library/Application Support/com.exodus.browser/plugins/native"
PLUGIN_PATH="$PLUGIN_DIR/libexodus_example_plugin.dylib"

if [ ! -f "$PLUGIN_PATH" ]; then
    print_error "Example plugin not found at: $PLUGIN_PATH"
    echo "Run: ./scripts/install-native-plugin.sh examples/native-plugin/target/release/libexodus_example_plugin.dylib"
    exit 1
fi

print_success "Example plugin found at: $PLUGIN_PATH"

# Verify plugin symbols
echo ""
print_info "Verifying plugin symbols..."
SYMBOLS=$(nm -g "$PLUGIN_PATH" | grep exodus_plugin || true)
if [ -z "$SYMBOLS" ]; then
    print_error "Plugin does not contain required symbols"
    exit 1
fi
print_success "Plugin symbols verified:"
echo "$SYMBOLS"

# Check plugin file permissions
echo ""
print_info "Checking plugin file permissions..."
PERMS=$(stat -f "%Lp" "$PLUGIN_PATH" 2>/dev/null || stat -c "%a" "$PLUGIN_PATH" 2>/dev/null)
if [ "$((PERMS & 0002))" -ne 0 ]; then
    print_error "Plugin is world-writable (security risk)"
    exit 1
fi
print_success "Plugin permissions are secure: $PERMS"

# Check if Exodus Browser is running
echo ""
print_info "Checking if Exodus Browser is running..."
if pgrep -x "Exodus" > /dev/null || pgrep -x "exodus" > /dev/null; then
    print_success "Exodus Browser is running"
else
    print_warning "Exodus Browser is not running"
    echo "Start Exodus Browser to test plugin integration"
fi

# Display test instructions
echo ""
echo "======================================"
echo "Manual Integration Test Steps"
echo "======================================"
echo ""
echo "1. Open Exodus Browser"
echo "2. Navigate to Settings > Native Plugins"
echo "3. Click 'Scan Plugins' to discover installed plugins"
echo "4. Verify 'Example Plugin' appears in the list"
echo "5. Enable the plugin if it's disabled"
echo "6. Click 'Commands' button on the plugin"
echo "7. Test the following commands:"
echo "   - ping: Should return { status: 'pong', message: '...' }"
echo "   - increment: Should increment internal counter"
echo "   - get_counter: Should return current counter value"
echo "   - echo: Should echo back the provided parameters"
echo ""
echo "Expected plugin metadata:"
echo "  - ID: example-plugin"
echo "  - Name: Example Plugin"
echo "  - Version: 1.0.0"
echo "  - Author: Exodus Team"
echo "  - Permissions: storage, network"
echo ""

# Create a simple test script for browser console
echo "======================================"
echo "Browser Console Test Commands"
echo "======================================"
echo ""
echo "Open Exodus Browser Developer Console (F12) and run:"
echo ""
echo "// Initialize plugin manager"
echo "await window.__TAURI_INVOKE__('init_native_plugin_manager');"
echo ""
echo "// List plugins"
echo "await window.__TAURI_INVOKE__('list_native_plugins');"
echo ""
echo "// Execute ping command"
echo "await window.__TAURI_INVOKE__('execute_plugin_command', {"
echo "  id: 'example-plugin',"
echo "  command: 'ping',"
echo "  params: {}"
echo "});"
echo ""
echo "// Execute increment command"
echo "await window.__TAURI_INVOKE__('execute_plugin_command', {"
echo "  id: 'example-plugin',"
echo "  command: 'increment',"
echo "  params: {}"
echo "});"
echo ""
echo "// Get counter value"
echo "await window.__TAURI_INVOKE__('execute_plugin_command', {"
echo "  id: 'example-plugin',"
echo "  command: 'get_counter',"
echo "  params: {}"
echo "});"
echo ""

print_success "Plugin integration test setup complete"
echo "Follow the steps above to verify the plugin system works correctly"
