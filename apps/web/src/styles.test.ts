import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const WEB_STYLES = resolve(process.cwd(), "src/styles.css");
const WECHAT_STYLES = resolve(
  process.cwd(),
  "../../apps/wechat-miniprogram/miniprogram/app.wxss",
);

function readCss(path: string) {
  return readFileSync(path, "utf8");
}

function extractBlock(css: string, selector: string) {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = css.match(
    new RegExp(`${escapedSelector}\\s*\\{([\\s\\S]*?)\\n\\}`),
  );

  if (!match) {
    throw new Error(`Missing CSS block: ${selector}`);
  }

  return match[1];
}

function extractVariables(block: string) {
  return Object.fromEntries(
    Array.from(block.matchAll(/(--[a-z0-9-]+):\s*([^;]+);/gi)).map(
      ([, name, value]) => [name, value.trim()],
    ),
  );
}

describe("web light theme color tokens", () => {
  it("uses the WeChat light-theme teal highlight system", () => {
    const webRootBlock = extractBlock(readCss(WEB_STYLES), ":root");
    const webTokens = extractVariables(webRootBlock);
    const wechatTokens = extractVariables(
      extractBlock(readCss(WECHAT_STYLES), "page,\n.theme-light"),
    );

    expect(webTokens["--brand-color"]).toBe(wechatTokens["--brand-color"]);
    expect(webTokens["--accent-color"]).toBe(wechatTokens["--accent-color"]);
    expect(webTokens["--page-bg"]).toBe(wechatTokens["--page-bg"]);
    expect(webTokens["--text-color"]).toBe(wechatTokens["--text-color"]);
    expect(webTokens["--muted-color"]).toBe(wechatTokens["--muted-color"]);
    expect(webTokens["--heading-muted-color"]).toBe(
      wechatTokens["--heading-muted-color"],
    );
    expect(webTokens["--control-border"]).toBe(
      wechatTokens["--control-border"],
    );
    expect(webTokens["--border-color"]).toBe(wechatTokens["--border-color"]);
    expect(webTokens["--chip-bg"]).toBe(wechatTokens["--chip-bg"]);
    expect(webRootBlock).not.toContain("#1f3a24");
    expect(webRootBlock).not.toContain("#5d7d46");
  });

  it("uses the brand color for primary call-to-action buttons", () => {
    const styles = readCss(WEB_STYLES);
    expect(extractBlock(styles, ".primary-button")).toContain(
      "background: var(--brand-color);",
    );
  });
});
