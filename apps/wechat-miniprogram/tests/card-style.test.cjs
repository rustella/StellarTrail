const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const miniRoot = path.resolve(__dirname, "../miniprogram");
const read = (rel) => fs.readFileSync(path.join(miniRoot, rel), "utf8");

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function findSelectorBrace(css, selector) {
  const escaped = escapeRegExp(selector);
  const pattern = new RegExp(`(^|[,\\n])\\s*${escaped}(?=\\s*[,\\{])`);
  let offset = 0;
  while (offset < css.length) {
    const match = css.slice(offset).match(pattern);
    if (!match || match.index === undefined) break;
    const index = offset + match.index;
    const brace = css.indexOf("{", index);
    const nextClose = css.indexOf("}", index);
    if (brace !== -1 && (nextClose === -1 || brace < nextClose)) return brace;
    offset = index + match[0].length;
  }
  assert.fail(`${selector} should exist`);
}

function selectorBlock(css, selector) {
  const brace = findSelectorBrace(css, selector);
  let depth = 0;
  for (let i = brace; i < css.length; i += 1) {
    if (css[i] === "{") depth += 1;
    if (css[i] === "}") {
      depth -= 1;
      if (depth === 0) return css.slice(brace + 1, i);
    }
  }
  assert.fail(`${selector} declaration block should close`);
}

function exactSelectorBlock(css, selector) {
  const escaped = escapeRegExp(selector);
  const match = css.match(new RegExp(`(^|\\n)${escaped}\\s*\\{`));
  assert.ok(match, `${selector} exact declaration block should exist`);
  const brace = css.indexOf("{", match.index);
  assert.notEqual(brace, -1, `${selector} should have a declaration block`);
  let depth = 0;
  for (let i = brace; i < css.length; i += 1) {
    if (css[i] === "{") depth += 1;
    if (css[i] === "}") {
      depth -= 1;
      if (depth === 0) return css.slice(brace + 1, i);
    }
  }
  assert.fail(`${selector} declaration block should close`);
}

function selectorGroupBlock(css, selectors) {
  const collapsed = selectors.join(",\n");
  const sameLine = selectors.join(", ");
  const index =
    css.indexOf(collapsed) >= 0
      ? css.indexOf(collapsed)
      : css.indexOf(sameLine);
  assert.notEqual(
    index,
    -1,
    `${selectors.join(", ")} should share a declaration block`,
  );
  const brace = css.indexOf("{", index);
  assert.notEqual(
    brace,
    -1,
    `${selectors.join(", ")} should have a declaration block`,
  );
  const end = css.indexOf("}", brace);
  assert.notEqual(
    end,
    -1,
    `${selectors.join(", ")} declaration block should close`,
  );
  return css.slice(brace + 1, end);
}

test("app exposes homepage card style tokens", () => {
  const wxss = read("app.wxss");
  for (const token of [
    "--card-radius: 30rpx",
    "--card-inner-radius: 24rpx",
    "--hero-card-radius: 36rpx",
    "--hero-surface-gradient:",
    "--hero-surface-border:",
    "--hero-surface-shadow:",
  ]) {
    assert.match(
      wxss,
      new RegExp(token.replace(/[()]/g, "\\$&")),
      `missing ${token}`,
    );
  }
  const darkBlock = selectorBlock(wxss, ".theme-dark");
  assert.match(darkBlock, /--hero-surface-gradient:\s*var\(--hero-gradient\)/);
  assert.match(darkBlock, /--hero-surface-shadow:\s*var\(--shadow-card\)/);
});

test("homepage keeps the canonical hero and surface tokens", () => {
  const wxss = read("pages/index/index.wxss");
  const hero = selectorBlock(wxss, ".hero-card");
  assert.match(hero, /border-radius:\s*var\(--hero-card-radius\)/);
  assert.match(hero, /background:\s*var\(--hero-surface-gradient\)/);
  assert.match(hero, /box-shadow:\s*var\(--hero-surface-shadow\)/);

  const firstLevel = selectorGroupBlock(wxss, [".quick-card", ".section-card"]);
  assert.match(firstLevel, /border-radius:\s*var\(--card-radius\)/);
  assert.match(firstLevel, /background:\s*var\(--surface-color\)/);
  assert.match(firstLevel, /box-shadow:\s*var\(--shadow-soft\)/);
});

test("page hero cards align with homepage in light and dark modes", () => {
  const cases = [
    ["pages/gears/index.wxss", ".hero-card"],
    ["pages/skills/index.wxss", ".hero-card"],
    ["pages/login/index.wxss", ".login-card.hero"],
    ["pages/register/index.wxss", ".register-card.hero"],
    ["pages/gears/form/index.wxss", ".intro-card"],
  ];
  for (const [file, heroSelector] of cases) {
    const wxss = read(file);
    const hero = exactSelectorBlock(wxss, heroSelector);
    assert.match(
      hero,
      /border-radius:\s*var\(--hero-card-radius\)/,
      `${file} hero radius`,
    );
    assert.match(
      hero,
      /background:\s*var\(--hero-surface-gradient\)/,
      `${file} hero background`,
    );
    assert.match(
      hero,
      /box-shadow:\s*var\(--hero-surface-shadow\)/,
      `${file} hero shadow`,
    );
    assert.doesNotMatch(
      wxss,
      /#0f172a\s+0%[\s\S]*#0f766e\s+100%/,
      `${file} should not use the old heavy light hero`,
    );
    assert.match(
      wxss,
      /theme-dark[\s\S]*background:\s*var\(--body-gradient\)/,
      `${file} dark page background`,
    );
  }
});

test("primary page cards share homepage surface tokens", () => {
  const cases = [
    [
      "pages/gears/index.wxss",
      [
        ".tab-card",
        ".search-card",
        ".gear-card",
        ".guest-card",
        ".template-card",
      ],
    ],
    [
      "pages/skills/index.wxss",
      [
        ".state-card",
        ".skill-category-card",
        ".knot-filter-card",
        ".skill-card",
      ],
    ],
    ["pages/login/index.wxss", [".login-card"]],
    ["pages/register/index.wxss", [".register-card"]],
    ["pages/profile/index.wxss", [".account-card"]],
    ["pages/gears/detail/index.wxss", [".info-card", ".state-card"]],
    [
      "pages/gears/form/index.wxss",
      [".form-card", ".error-card", ".state-card"],
    ],
    [
      "pages/skills/detail/index.wxss",
      [".detail-hero", ".info-card", ".state-card"],
    ],
  ];
  for (const [file, selectors] of cases) {
    const wxss = read(file);
    for (const selector of selectors) {
      const block = selectorBlock(wxss, selector);
      assert.match(
        wxss,
        new RegExp(selector.replace(".", "\\.")),
        `${file} ${selector} exists`,
      );
      assert.match(
        block,
        /border-radius:\s*var\(--card-radius\)/,
        `${file} ${selector} radius`,
      );
      assert.match(
        block,
        /background:\s*var\(--surface-color\)/,
        `${file} ${selector} surface`,
      );
      assert.match(
        block,
        /box-shadow:\s*var\(--shadow-soft\)/,
        `${file} ${selector} shadow`,
      );
    }
  }
});

test("inner cards and panels use homepage inner surface tokens", () => {
  const cases = [
    ["pages/index/index.wxss", [".stat-card", ".skill-card"]],
    ["pages/gears/index.wxss", [".stat-card", ".metric"]],
    ["pages/login/index.wxss", [".auth-panel"]],
    ["pages/gears/detail/index.wxss", [".summary-item"]],
    ["pages/gears/form/index.wxss", [".field", ".picker-row", ".switch-row"]],
    ["pages/skills/index.wxss", [".category-icon", ".skill-thumb"]],
  ];
  for (const [file, selectors] of cases) {
    const wxss = read(file);
    for (const selector of selectors) {
      const block = selectorBlock(wxss, selector);
      assert.match(
        block,
        /border-radius:\s*var\(--card-inner-radius\)/,
        `${file} ${selector} radius`,
      );
      assert.match(
        block,
        /background:\s*var\(--control-bg\)/,
        `${file} ${selector} control bg`,
      );
    }
  }
});

test("skill detail page uses a media-first layout without practice steps", () => {
  const wxml = read("pages/skills/detail/index.wxml");
  const wxss = read("pages/skills/detail/index.wxss");
  const ts = read("pages/skills/detail/index.ts");
  assert.doesNotMatch(wxml, /练习步骤/);
  assert.doesNotMatch(wxml, /steps/);
  assert.match(wxml, /class="media-stage"/);
  assert.match(wxml, /class="stage-image"/);
  assert.match(wxml, /mode="aspectFit"/);
  assert.doesNotMatch(wxml, /<video/);
  assert.match(wxml, /class="detail-summary-panel"/);
  assert.match(wxml, /class="media-control-label"/);
  assert.match(wxml, /class="media-help"/);

  const mediaStage = selectorBlock(wxss, ".media-stage");
  assert.match(mediaStage, /background:\s*#020617/);
  const mediaFrame = selectorBlock(wxss, ".media-frame");
  assert.match(mediaFrame, /height:\s*430rpx/);
  assert.match(mediaFrame, /background:\s*#f8fafc/);
  const detailHero = exactSelectorBlock(wxss, ".detail-hero");
  assert.match(detailHero, /overflow:\s*hidden/);
  const mediaHelp = selectorBlock(wxss, ".media-help");
  assert.match(mediaHelp, /text-align:\s*center/);
  const mediaToolbar = selectorBlock(wxss, ".media-toolbar");
  assert.match(mediaToolbar, /justify-content:\s*center/);
  assert.match(ts, /filter\(isDetailMediaAsset\)/);
  assert.match(
    ts,
    /media_type === "preview"[\s\S]*media_type === "draw_gif"[\s\S]*media_type === "turntable_gif"/,
  );
  assert.doesNotMatch(ts, /thumbnail|draw_mp4|turntable_mp4/);
  assert.match(ts, /静态图/);
  assert.match(ts, /旋转动图/);
  assert.doesNotMatch(ts, /高清图|系法视频|旋转视频/);
});

test("skills knot list thumbnails preserve the full image", () => {
  const wxml = read("pages/skills/index.wxml");
  const wxss = read("pages/skills/index.wxss");
  assert.match(
    wxml,
    /<image[\s\S]*wx:if="{{item\.hasThumbnail}}"[\s\S]*class="skill-thumb-image"[\s\S]*mode="aspectFit"[\s\S]*src="{{item\.thumbnailUrl}}"[\s\S]*\/>/,
  );
  const thumb = selectorBlock(wxss, ".skill-thumb");
  assert.match(thumb, /width:\s*164rpx/);
  assert.match(thumb, /height:\s*124rpx/);
  const image = selectorBlock(wxss, ".skill-thumb-image");
  assert.match(image, /width:\s*100%/);
  assert.match(image, /height:\s*100%/);
});

test("skills knot list offers search and category-only filtering", () => {
  const wxml = read("pages/skills/index.wxml");
  const wxss = read("pages/skills/index.wxss");
  const ts = read("pages/skills/index.ts");

  assert.match(wxml, /class="knot-filter-card"/);
  assert.match(wxml, /placeholder="搜索绳结名称、用途"/);
  assert.match(wxml, /bindinput="onSearchInput"/);
  assert.match(wxml, /bindchange="onCategoryFilterChange"/);
  assert.match(wxml, /class="category-filter-picker"/);
  assert.match(wxml, /range="{{categoryFilterLabels}}"/);
  assert.match(wxml, /class="category-filter-pill"/);
  assert.match(wxml, /class="result-count"/);

  const filterCard = selectorBlock(wxss, ".knot-filter-card");
  assert.match(filterCard, /border-radius:\s*var\(--card-radius\)/);
  assert.match(filterCard, /background:\s*var\(--surface-color\)/);
  const searchInput = selectorBlock(wxss, ".knot-search-input");
  assert.match(searchInput, /background:\s*var\(--control-bg\)/);
  const filterPill = selectorBlock(wxss, ".category-filter-pill");
  assert.match(filterPill, /background:\s*var\(--control-bg\)/);

  assert.match(ts, /categoryFilters/);
  assert.match(ts, /categoryFilterLabels/);
  assert.match(
    ts,
    /item\.categories\.map\(\(category\) => category\.id \|\| category\.slug\)/,
  );
  assert.match(ts, /knot\.categoryIds\.includes\(selectedCategoryId\)/);
  assert.match(ts, /item\.types\.map\(\(type\) => type\.title\)/);
});
