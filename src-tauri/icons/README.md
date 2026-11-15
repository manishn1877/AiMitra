# Icons

Place your app icons here:

## Required Files

| File | Size | Platform |
|------|------|----------|
| `icon.ico` | 256x256 (multi-res) | Windows |
| `icon.icns` | 1024x1024 | macOS |
| `32x32.png` | 32x32 | Linux/Windows |
| `128x128.png` | 128x128 | Linux/macOS |
| `128x128@2x.png` | 256x256 | macOS Retina |
| `icon.png` | 512x512 | Tray/Generic |

## Quick Generation

```bash
# From a 1024x1024 source PNG (source.png)
# Install ImageMagick first

# Windows ICO (multi-resolution)
magick source.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

# macOS ICNS
mkdir icon.iconset
for size in 16 32 64 128 256 512 1024; do
  magick source.png -resize ${size}x${size} icon.iconset/icon_${size}x${size}.png
  magick source.png -resize $((size*2))x$((size*2)) icon.iconset/icon_${size}x${size}@2x.png
done
iconutil -c icns icon.iconset -o icon.icns

# PNGs
magick source.png -resize 32x32 32x32.png
magick source.png -resize 128x128 128x128.png
magick source.png -resize 256x256 128x128@2x.png
cp source.png icon.png
```

## Design Guidelines

- **Style**: Clean, geometric, code-themed (brackets, chevron, cursor)
- **Colors**: Brand blue (#0ea5e9) on dark, works on light/dark backgrounds
- **Safe area**: Keep main symbol within 70% of canvas
- **No text** — icon should work at 16x16

## Placeholder

For now, create a simple placeholder:

```bash
# Quick SVG placeholder
cat > icon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="48" fill="#0ea5e9"/>
  <path d="M64 96h128M64 128h96M64 160h64" stroke="white" stroke-width="16" stroke-linecap="round" fill="none"/>
  <circle cx="192" cy="160" r="24" fill="white" opacity="0.2"/>
  <path d="M184 160h16M192 152v16" stroke="white" stroke-width="6" stroke-linecap="round"/>
</svg>
EOF

# Convert with ImageMagick
magick icon.svg icon.png
# ... then generate other sizes
```