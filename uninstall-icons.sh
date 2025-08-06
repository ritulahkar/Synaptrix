#!/bin/bash

# Synaptrix Uninstall Script

ICON_NAME="synaptrix"
DESKTOP_FILE="synaptrix.desktop"

echo "🗑️  Uninstalling Synaptrix icons and desktop integration..."

# Determine if removing system-wide or user installation
SYSTEM_REMOVAL=false
USER_REMOVAL=false

# Check what's installed where
if [ -f "/usr/share/applications/$DESKTOP_FILE" ] || [ -f "/usr/share/icons/hicolor/scalable/apps/$ICON_NAME.svg" ]; then
    if [ "$EUID" -eq 0 ]; then
        SYSTEM_REMOVAL=true
        echo "📁 Removing system-wide installation..."
    else
        echo "⚠️  System-wide installation detected, but you're not running as root."
        echo "   Run with sudo to remove system-wide files: sudo $0"
        echo "   Continuing with user files only..."
    fi
fi

if [ -f "$HOME/.local/share/applications/$DESKTOP_FILE" ] || [ -f "$HOME/.local/share/icons/hicolor/scalable/apps/$ICON_NAME.svg" ]; then
    USER_REMOVAL=true
    echo "📁 Removing user installation..."
fi

if [ "$SYSTEM_REMOVAL" = false ] && [ "$USER_REMOVAL" = false ]; then
    echo "ℹ️  No Synaptrix installation found to remove."
    exit 0
fi

# Function to remove icons from a directory
remove_icons() {
    local icon_base_dir="$1"
    local removed_count=0
    
    echo "   Removing icons from $icon_base_dir..."
    
    # Remove PNG icons
    for size in 16 24 32 48 64 128 256 512; do
        icon_file="$icon_base_dir/${size}x${size}/apps/$ICON_NAME.png"
        if [ -f "$icon_file" ]; then
            rm "$icon_file"
            echo "     ✅ Removed ${size}x${size} icon"
            removed_count=$((removed_count + 1))
        fi
    done
    
    # Remove SVG icon
    svg_file="$icon_base_dir/scalable/apps/$ICON_NAME.svg"
    if [ -f "$svg_file" ]; then
        rm "$svg_file"
        echo "     ✅ Removed SVG icon"
        removed_count=$((removed_count + 1))
    fi
    
    return $removed_count
}

# Remove system-wide installation
if [ "$SYSTEM_REMOVAL" = true ]; then
    echo "🔧 Removing system-wide files..."
    
    # Remove desktop file
    if [ -f "/usr/share/applications/$DESKTOP_FILE" ]; then
        rm "/usr/share/applications/$DESKTOP_FILE"
        echo "   ✅ Removed system desktop file"
    fi
    
    # Remove icons
    remove_icons "/usr/share/icons/hicolor"
    system_icons_removed=$?
    
    # Update system caches
    echo "   🔄 Updating system caches..."
    if gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ 2>/dev/null; then
        echo "   ✅ Updated system icon cache"
    fi
    
    if update-desktop-database /usr/share/applications/ 2>/dev/null; then
        echo "   ✅ Updated system desktop database"
    fi
fi

# Remove user installation
if [ "$USER_REMOVAL" = true ]; then
    echo "🔧 Removing user files..."
    
    # Remove desktop file
    if [ -f "$HOME/.local/share/applications/$DESKTOP_FILE" ]; then
        rm "$HOME/.local/share/applications/$DESKTOP_FILE"
        echo "   ✅ Removed user desktop file"
    fi
    
    # Remove icons
    remove_icons "$HOME/.local/share/icons/hicolor"
    user_icons_removed=$?
    
    # Update user caches
    echo "   🔄 Updating user caches..."
    if gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null; then
        echo "   ✅ Updated user icon cache"
    fi
    
    if update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null; then
        echo "   ✅ Updated user desktop database"
    fi
fi

# Remove any leftover cache entries
echo "🧹 Cleaning up caches..."

# Clear desktop environment caches
if [ -f "$HOME/.cache/icon-theme.cache" ]; then
    rm "$HOME/.cache/icon-theme.cache"
    echo "   ✅ Cleared icon theme cache"
fi

# Clear menu caches (various desktop environments)
for cache_file in \
    "$HOME/.cache/menus/applications.menu" \
    "$HOME/.cache/desktop-directories.cache" \
    "$HOME/.local/share/applications/mimeinfo.cache"
do
    if [ -f "$cache_file" ]; then
        rm "$cache_file"
        echo "   ✅ Cleared $(basename "$cache_file")"
    fi
done

echo ""
echo "📊 Uninstallation Summary:"
[ "$SYSTEM_REMOVAL" = true ] && echo "   System-wide: ✅ Removed"
[ "$USER_REMOVAL" = true ] && echo "   User files: ✅ Removed"
echo "   Caches: ✅ Cleared"

echo ""
echo "🎉 Synaptrix uninstallation complete!"
echo ""
echo "💡 Additional cleanup (optional):"
echo "   - Remove binary: sudo rm /usr/local/bin/synaptrix"
echo "   - Remove config: rm -rf ~/.config/synaptrix/"
echo "   - Restart desktop session for complete cleanup"

echo ""
echo "🔍 Verification:"
echo "   Check system icons: ls /usr/share/icons/hicolor/*/apps/$ICON_NAME.* 2>/dev/null || echo 'None found'"
echo "   Check user icons: ls ~/.local/share/icons/hicolor/*/apps/$ICON_NAME.* 2>/dev/null || echo 'None found'"
echo "   Check desktop files: ls /usr/share/applications/$DESKTOP_FILE ~/.local/share/applications/$DESKTOP_FILE 2>/dev/null || echo 'None found'"