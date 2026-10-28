#!/usr/bin/env bash
set -euo pipefail

# Regeneration reference for the application icon.
# The committed PNG/ICO files make normal builds independent of these tools.
# Tools: rsvg-convert (librsvg), inkscape, or imagemagick as a fallback.

SOURCE="assets/icons/heic-converter.svg"
OUTPUT_DIR="assets/icons"

mkdir -p "$OUTPUT_DIR"
for SIZE in 16 32 48 64 128 256 512; do
  rsvg-convert -w "$SIZE" -h "$SIZE" "$SOURCE" -o "$OUTPUT_DIR/heic-converter-${SIZE}.png"
done

# Windows multi-size ICO (Pillow or ImageMagick can also produce it):
magick "$OUTPUT_DIR/heic-converter-256.png" \
  \( "$OUTPUT_DIR/heic-converter-48.png" \) \
  \( "$OUTPUT_DIR/heic-converter-32.png" \) \
  \( "$OUTPUT_DIR/heic-converter-16.png" \) \
  "$OUTPUT_DIR/heic-converter.ico"