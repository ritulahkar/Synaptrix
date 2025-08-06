#!/bin/bash

# Synaptrix Logo Format Converter
# Converts a single source image to all required icon formats

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
SOURCE_IMAGE=""
OUTPUT_DIR="./icons"
ICON_NAME="synaptrix"
QUALITY=95

# Function to show usage
show_usage() {
    echo -e "${BLUE}Synaptrix Logo Format Converter${NC}"
    echo "Usage: $0 -i <input_image> [options]"
    echo ""
    echo "Options:"
    echo "  -i, --input <file>     Source image file (required)"
    echo "  -o, --output <dir>     Output directory (default: ./icons)"
    echo "  -n, --name <name>      Icon name (default: synaptrix)"
    echo "  -q, --quality <num>    PNG quality 1-100 (default: 95)"
    echo "  -h, --help            Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 -i logo.png"
    echo "  $0 -i logo.svg -o /tmp/icons -n myapp"
    echo ""
    echo "Supported input formats: PNG, JPG, SVG, GIF, WEBP, etc."
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--input)
            SOURCE_IMAGE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -n|--name)
            ICON_NAME="$2"
            shift 2
            ;;
        -q|--quality)
            QUALITY="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            show_usage
            exit 1
            ;;
    esac
done

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo -e "${RED}Error: ImageMagick is not installed.${NC}"
    echo "Install it with:"
    echo "  Ubuntu/Debian: sudo apt install imagemagick"
    echo "  Fedora:        sudo dnf install ImageMagick"
    echo "  Arch:          sudo pacman -S imagemagick"
    exit 1
fi

# Check if inkscape is available (for SVG optimization)
INKSCAPE_AVAILABLE=false
if command -v inkscape &> /dev/null; then
    INKSCAPE_AVAILABLE=true
fi

# Validate input
if [[ -z "$SOURCE_IMAGE" ]]; then
    echo -e "${RED}Error: Input image is required.${NC}"
    show_usage
    exit 1
fi

if [[ ! -f "$SOURCE_IMAGE" ]]; then
    echo -e "${RED}Error: Input file '$SOURCE_IMAGE' does not exist.${NC}"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}Synaptrix Logo Converter${NC}"
echo -e "Source: ${YELLOW}$SOURCE_IMAGE${NC}"
echo -e "Output: ${YELLOW}$OUTPUT_DIR${NC}"
echo -e "Name:   ${YELLOW}$ICON_NAME${NC}"
echo ""

# Define all required sizes
SIZES=(16 24 32 48 64 128 256 512)

# Convert to PNG formats
echo -e "${BLUE}Converting to PNG formats...${NC}"
for size in "${SIZES[@]}"; do
    output_file="$OUTPUT_DIR/${ICON_NAME}-${size}.png"
    echo -n "  Creating ${size}x${size}... "
    
    if convert "$SOURCE_IMAGE" \
        -resize "${size}x${size}" \
        -quality $QUALITY \
        -background transparent \
        -gravity center \
        -extent "${size}x${size}" \
        "$output_file" 2>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
    fi
done

# Create SVG version
echo -e "${BLUE}Creating SVG version...${NC}"
svg_output="$OUTPUT_DIR/${ICON_NAME}.svg"

# Check if source is already SVG
if [[ "${SOURCE_IMAGE,,}" == *.svg ]]; then
    echo -n "  Copying SVG... "
    cp "$SOURCE_IMAGE" "$svg_output"
    echo -e "${GREEN}✓${NC}"
elif $INKSCAPE_AVAILABLE; then
    echo -n "  Converting to SVG with Inkscape... "
    if inkscape --export-type=svg --export-filename="$svg_output" "$SOURCE_IMAGE" 2>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠ (fallback to PNG conversion)${NC}"
        convert "$SOURCE_IMAGE" -resize 512x512 -quality 100 -background transparent "$svg_output.png"
        mv "$svg_output.png" "$svg_output"
    fi
else
    echo -n "  Creating SVG (ImageMagick fallback)... "
    # Create a simple SVG wrapper (not ideal but works)
    convert "$SOURCE_IMAGE" -resize 512x512 -quality 100 -background transparent "${OUTPUT_DIR}/temp_svg.png"
    cat > "$svg_output" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<svg width="512" height="512" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <image width="512" height="512" xlink:href="data:image/png;base64,$(base64 -w 0 "${OUTPUT_DIR}/temp_svg.png")" />
</svg>
EOF
    rm "${OUTPUT_DIR}/temp_svg.png"
    echo -e "${YELLOW}✓ (embedded PNG)${NC}"
fi

# Create additional formats for different contexts
echo -e "${BLUE}Creating additional formats...${NC}"

# ICO format for Windows compatibility (if needed)
echo -n "  Creating ICO format... "
if convert "$SOURCE_IMAGE" \
    \( -clone 0 -resize 16x16 \) \
    \( -clone 0 -resize 32x32 \) \
    \( -clone 0 -resize 48x48 \) \
    \( -clone 0 -resize 64x64 \) \
    -delete 0 \
    -background transparent \
    "$OUTPUT_DIR/${ICON_NAME}.ico" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${YELLOW}⚠ (skipped)${NC}"
fi

# High-DPI versions (2x)
echo -n "  Creating high-DPI versions... "
for size in 32 48 64; do
    hidpi_size=$((size * 2))
    convert "$SOURCE_IMAGE" \
        -resize "${hidpi_size}x${hidpi_size}" \
        -quality $QUALITY \
        -background transparent \
        "$OUTPUT_DIR/${ICON_NAME}-${size}@2x.png" 2>/dev/null
done
echo -e "${GREEN}✓${NC}"

# Create desktop wallpaper version
echo -n "  Creating wallpaper version... "
if convert "$SOURCE_IMAGE" \
    -resize 1920x1080 \
    -background "rgba(44,62,80,0.8)" \
    -gravity center \
    -quality $QUALITY \
    "$OUTPUT_DIR/${ICON_NAME}-wallpaper.png" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${YELLOW}⚠ (skipped)${NC}"
fi

# Generate summary
echo ""
echo -e "${GREEN}✅ Conversion completed!${NC}"
echo -e "${BLUE}Generated files:${NC}"

# List all created files with sizes
for size in "${SIZES[@]}"; do
    file="$OUTPUT_DIR/${ICON_NAME}-${size}.png"
    if [[ -f "$file" ]]; then
        file_size=$(du -h "$file" | cut -f1)
        echo "  📄 ${ICON_NAME}-${size}.png (${file_size})"
    fi
done

if [[ -f "$OUTPUT_DIR/${ICON_NAME}.svg" ]]; then
    svg_size=$(du -h "$OUTPUT_DIR/${ICON_NAME}.svg" | cut -f1)
    echo "  📄 ${ICON_NAME}.svg (${svg_size})"
fi

if [[ -f "$OUTPUT_DIR/${ICON_NAME}.ico" ]]; then
    ico_size=$(du -h "$OUTPUT_DIR/${ICON_NAME}.ico" | cut -f1)
    echo "  📄 ${ICON_NAME}.ico (${ico_size})"
fi

# Show next steps
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Run the icon installation script:"
echo "   ./install-icons.sh"
echo ""
echo "2. Or install manually:"
echo "   sudo cp $OUTPUT_DIR/${ICON_NAME}-*.png /usr/share/icons/hicolor/*/apps/"
echo "   sudo cp $OUTPUT_DIR/${ICON_NAME}.svg /usr/share/icons/hicolor/scalable/apps/"
echo "   sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor/"
echo ""
echo -e "${GREEN}🎉 Your Synaptrix icons are ready!${NC}"