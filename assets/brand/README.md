# StellarTrail icon assets

This directory keeps `source-app-icon.png` as the only source artwork. Generated icon exports are direct high-quality resizes of that source image; alpha transparency and the source rounded corners are preserved for PNG, WebP, and ICO outputs.

SVG/vector redraw exports are intentionally not included.

## Source

- `source-app-icon.png`: original supplied image (144×144).

## Common raster exports

PNG exports are intentionally limited to the common sizes: 32, 180, 192, 512, and 1024 px.

Additional common formats:

- `app-icon.ico`: multi-size favicon bundle with 16, 32, 48, 64, 128, and 256 px frames.
- `app-icon-512.webp`, `app-icon-1024.webp`.
- `app-icon-512.jpg`, `app-icon-1024.jpg`.

## Platform copies

- Web/PWA raster files live under `apps/web/public/` and are linked from `apps/web/index.html`.
- Android launcher resources live under `apps/android/src/main/res/` with standard density PNG assets and adaptive icon XML pointing at the PNG foreground.
- Mini Program raster copies live under `apps/wechat-miniprogram/miniprogram/assets/icons/` with only 128, 192, and 512 px exports.
