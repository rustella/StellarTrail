# StellarTrail icon assets

This directory contains raster icon exports generated directly from the user-supplied night-mountain artwork. The bitmap artwork is the source of truth; SVG/vector redraw exports are intentionally not included.

## Source

- `source-app-icon.png`: original attached image (634×596).
- Raster exports are generated from a square crop of the original artwork (594×594, crop box `(6, 2, 600, 596)`).

## Raster exports

PNG exports are available at 16, 24, 32, 48, 64, 72, 96, 128, 144, 152, 180, 192, 256, 384, 512, and 1024 px.

Additional formats:

- `stellartrail-icon.ico`: multi-size favicon bundle.
- `stellartrail-icon-256.webp`, `stellartrail-icon-512.webp`, `stellartrail-icon-1024.webp`.
- `stellartrail-icon-512.jpg`, `stellartrail-icon-1024.jpg`.

## Platform copies

- Web/PWA raster files live under `apps/web/public/` and are linked from `apps/web/index.html`.
- Android launcher resources live under `apps/android/src/main/res/` with PNG density assets and adaptive icon XML pointing at the PNG foreground.
- Mini Program raster copies live under `apps/wechat-miniprogram/miniprogram/assets/icons/`.
