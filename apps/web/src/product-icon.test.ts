import { existsSync, readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const PUBLIC_DIR = resolve(process.cwd(), "public");
const PRODUCT_ICON = resolve(PUBLIC_DIR, "stellartrail-icon.png");
const FAVICON_SVG = resolve(PUBLIC_DIR, "favicon.svg");
const LEGACY_SVG_ALIAS = resolve(PUBLIC_DIR, "stellartrail-icon.svg");
const FAVICON_ICO = resolve(PUBLIC_DIR, "favicon.ico");
const FAVICON_16 = resolve(PUBLIC_DIR, "icons/favicon-16.png");
const FAVICON_32 = resolve(PUBLIC_DIR, "icons/favicon-32.png");
const FAVICON_48 = resolve(PUBLIC_DIR, "icons/favicon-48.png");
const APPLE_TOUCH_ICON = resolve(PUBLIC_DIR, "apple-touch-icon.png");
const ICON_180 = resolve(PUBLIC_DIR, "icons/icon-180.png");
const ICON_192 = resolve(PUBLIC_DIR, "icons/icon-192.png");
const ICON_512 = resolve(PUBLIC_DIR, "icons/icon-512.png");
const MASKABLE_ICON_512 = resolve(PUBLIC_DIR, "icons/icon-maskable-512.png");
const WEB_MANIFEST = resolve(PUBLIC_DIR, "manifest.webmanifest");
const INDEX_HTML = resolve(process.cwd(), "index.html");

function expectPngDimensions(path: string, width: number, height: number) {
  const image = readFileSync(path);

  expect([...image.subarray(0, 8)]).toEqual([
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
  ]);
  expect(image.toString("ascii", 12, 16)).toBe("IHDR");
  expect(image.readUInt32BE(16)).toBe(width);
  expect(image.readUInt32BE(20)).toBe(height);
}

function parseIcoSizes(path: string) {
  const icon = readFileSync(path);
  const reserved = icon.readUInt16LE(0);
  const type = icon.readUInt16LE(2);
  const count = icon.readUInt16LE(4);

  expect(reserved).toBe(0);
  expect(type).toBe(1);
  expect(count).toBeGreaterThanOrEqual(3);

  return Array.from({ length: count }, (_, index) => {
    const offset = 6 + index * 16;
    const width = icon[offset] === 0 ? 256 : icon[offset];
    const height = icon[offset + 1] === 0 ? 256 : icon[offset + 1];
    return `${width}x${height}`;
  });
}

describe("web product icon asset", () => {
  it("ships a raster PNG icon generated from the supplied artwork", () => {
    expectPngDimensions(PRODUCT_ICON, 512, 512);
    expect(statSync(PRODUCT_ICON).size).toBeGreaterThan(20000);
    expect(existsSync(FAVICON_SVG)).toBe(false);
    expect(existsSync(LEGACY_SVG_ALIAS)).toBe(false);
  });

  it("ships raster favicon fallbacks for browsers and app installs", () => {
    expectPngDimensions(FAVICON_16, 16, 16);
    expectPngDimensions(FAVICON_32, 32, 32);
    expectPngDimensions(FAVICON_48, 48, 48);
    expectPngDimensions(APPLE_TOUCH_ICON, 180, 180);
    expectPngDimensions(ICON_180, 180, 180);
    expectPngDimensions(ICON_192, 192, 192);
    expectPngDimensions(ICON_512, 512, 512);
    expectPngDimensions(MASKABLE_ICON_512, 512, 512);

    expect(parseIcoSizes(FAVICON_ICO)).toEqual(
      expect.arrayContaining(["16x16", "32x32", "48x48"]),
    );
  });

  it("declares installable web app icons in the manifest", () => {
    const manifest = JSON.parse(readFileSync(WEB_MANIFEST, "utf8")) as {
      name: string;
      short_name: string;
      theme_color: string;
      background_color: string;
      display: string;
      icons: Array<{
        src: string;
        sizes: string;
        type: string;
        purpose?: string;
      }>;
    };

    expect(manifest.name).toBe("寻径星野");
    expect(manifest.short_name).toBe("寻径星野");
    expect(manifest.theme_color).toBe("#0f172a");
    expect(manifest.background_color).toBe("#0B2435");
    expect(manifest.display).toBe("standalone");
    expect(manifest.icons).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          src: "/icons/icon-192.png",
          sizes: "192x192",
          type: "image/png",
        }),
        expect.objectContaining({
          src: "/icons/icon-512.png",
          sizes: "512x512",
          type: "image/png",
        }),
        expect.objectContaining({
          src: "/icons/icon-maskable-512.png",
          sizes: "512x512",
          type: "image/png",
          purpose: "maskable",
        }),
      ]),
    );
  });

  it("uses the raster icon set as the browser product icon", () => {
    const html = readFileSync(INDEX_HTML, "utf8");

    expect(html).toContain(
      '<link rel="icon" href="/favicon.ico" sizes="any" />',
    );
    expect(html).toContain('href="/icons/favicon-32.png"');
    expect(html).toContain('href="/icons/favicon-48.png"');
    expect(html).toContain(
      '<link rel="apple-touch-icon" href="/apple-touch-icon.png" />',
    );
    expect(html).toContain(
      '<link rel="manifest" href="/manifest.webmanifest" />',
    );
    expect(html).toContain('<meta name="theme-color" content="#0f172a" />');
    expect(html).not.toContain("image/svg+xml");
    expect(html).not.toContain("favicon.svg");
  });
});
