#!/bin/bash

# Synaptrix Icon Installation Script

ICON_NAME="synaptrix"
DESKTOP_FILE="synaptrix.desktop"
ICON_SOURCE_DIR="icons"

# Check if running as root for system-wide install
if [ "$EUID" -eq 0 ]; then
    ICON_DIR="/usr/share/icons/hicolor"
    APP_DIR="/usr/share/applications"
    echo "Installing system-wide..."
else
    ICON_DIR="$HOME/.local/share/icons/hicolor"
    APP_DIR="$HOME/.local/share/applications"
    echo "Installing for current user..."
fi

# Check if icons directory exists
if [ ! -d "$ICON_SOURCE_DIR" ]; then
    echo "Error: Icons directory '$ICON_SOURCE_DIR' not found!"
    echo "Please make sure you have an 'icons/' directory with your icon files."
    exit 1
fi

# Create target directories
mkdir -p "$ICON_DIR"/{16x16,24x24,32x32,48x48,64x64,128x128,256x256,512x512,scalable}/apps
mkdir -p "$APP_DIR"

# Install PNG icons from icons directory
echo "Installing PNG icons..."
for size in 16 24 32 48 64 128 256 512; do
    # Try different possible naming patterns
    icon_file=""
    
    # Check for synaptrix-SIZE.png
    if [ -f "$ICON_SOURCE_DIR/${ICON_NAME}-${size}.png" ]; then
        icon_file="$ICON_SOURCE_DIR/${ICON_NAME}-${size}.png"
    # Check for synaptrix_SIZE.png
    elif [ -f "$ICON_SOURCE_DIR/${ICON_NAME}_${size}.png" ]; then
        icon_file="$ICON_SOURCE_DIR/${ICON_NAME}_${size}.png"
    # Check for SIZE.png
    elif [ -f "$ICON_SOURCE_DIR/${size}.png" ]; then
        icon_file="$ICON_SOURCE_DIR/${size}.png"
    # Check for synaptrix-SIZExSIZE.png
    elif [ -f "$ICON_SOURCE_DIR/${ICON_NAME}-${size}x${size}.png" ]; then
        icon_file="$ICON_SOURCE_DIR/${ICON_NAME}-${size}x${size}.png"
    fi
    
    if [ -n "$icon_file" ]; then
        cp "$icon_file" "$ICON_DIR/${size}x${size}/apps/${ICON_NAME}.png"
        echo "✓ Installed ${size}x${size} icon from $(basename "$icon_file")"
    else
        echo "⚠ No ${size}x${size} icon found (tried multiple naming patterns)"
    fi
done

# Install SVG icons
echo "Installing SVG icons..."
svg_installed=false
for svg_pattern in "${ICON_NAME}.svg" "synaptrix.svg" "icon.svg" "logo.svg"; do
    if [ -f "$ICON_SOURCE_DIR/$svg_pattern" ]; then
        cp "$ICON_SOURCE_DIR/$svg_pattern" "$ICON_DIR/scalable/apps/${ICON_NAME}.svg"
        echo "✓ Installed SVG icon from $svg_pattern"
        svg_installed=true
        break
    fi
done

if [ "$svg_installed" = false ]; then
    echo "⚠ No SVG icon found"
fi

# Install desktop file
echo "Installing desktop file..."
if [ -f "$DESKTOP_FILE" ]; then
    cp "$DESKTOP_FILE" "$APP_DIR/"
    echo "✓ Installed desktop file"
else
    echo "⚠ Desktop file '$DESKTOP_FILE' not found"
fi

# List what we found in the icons directory
echo ""
echo "Icons found in '$ICON_SOURCE_DIR':"
ls -la "$ICON_SOURCE_DIR"/ 2>/dev/null || echo "Could not list icons directory"

# Update caches
echo ""
echo "Updating system caches..."
if [ "$EUID" -eq 0 ]; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor/
    update-desktop-database /usr/share/applications/
    echo "✓ Updated system caches"
else
    if gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor/ 2>/dev/null; then
        echo "✓ Updated user icon cache"
    else
        echo "⚠ Could not update icon cache (this is usually fine)"
    fi
    
    if update-desktop-database ~/.local/share/applications/ 2>/dev/null; then
        echo "✓ Updated desktop database"
    else
        echo "⚠ Could not update desktop database (this is usually fine)"
    fi
fi

echo ""
echo "Installation complete! You may need to restart your desktop session to see changes."
echo ""
echo "To verify installation:"
echo "- Check if icon appears: ls $ICON_DIR/*/apps/$ICON_NAME.*"
echo "- Test launcher: gtk-launch $ICON_NAME"