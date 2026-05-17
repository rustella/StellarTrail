import { readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const PRODUCT_ICON = resolve(process.cwd(), "public/stellartrail-icon.svg");
const INDEX_HTML = resolve(process.cwd(), "index.html");

describe("web product icon asset", () => {
  it("ships a crisp SVG icon based on the mountain trail artwork", () => {
    const icon = readFileSync(PRODUCT_ICON, "utf8");
    const size = statSync(PRODUCT_ICON).size;

    expect(icon).toContain('viewBox="0 0 144 144"');
    expect(icon).toContain('id="night-sky"');
    expect(icon).toContain('id="mountains"');
    expect(icon).toContain('id="trail"');
    expect(icon).toContain('id="compass-star"');
    expect(icon).not.toMatch(/<text/i);
    expect(size).toBeGreaterThan(3000);
  });

  it("uses the optimized icon as the browser product icon", () => {
    const html = readFileSync(INDEX_HTML, "utf8");

    expect(html).toContain(
      '<link rel="icon" type="image/svg+xml" href="/favicon.svg" />',
    );
    expect(html).toContain(
      '<link rel="apple-touch-icon" href="/apple-touch-icon.png" />',
    );
    expect(html).toContain(
      '<link rel="manifest" href="/manifest.webmanifest" />',
    );
  });
});
