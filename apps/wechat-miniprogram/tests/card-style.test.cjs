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

  const firstLevel = selectorBlock(wxss, ".section-card");
  assert.match(firstLevel, /border-radius:\s*var\(--card-radius\)/);
  assert.match(firstLevel, /background:\s*var\(--surface-color\)/);
  assert.match(firstLevel, /box-shadow:\s*var\(--shadow-soft\)/);

  const wxml = read("pages/index/index.wxml");
  const knotPack = selectorBlock(wxss, ".knot-pack");
  assert.match(wxml, /class="knot-pack"/);
  assert.match(wxml, /class="skill-thumb-image"[\s\S]*mode="aspectFit"/);
  assert.match(knotPack, /border-radius:\s*var\(--card-inner-radius\)/);
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
      [".tab-card", ".search-card", ".gear-card", ".guest-card"],
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
    ["pages/profile/index.wxss", [".account-card", ".settings-card"]],
    ["pages/profile/settings/index.wxss", [".settings-card"]],
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

test("profile pages expose page-local dark backgrounds", () => {
  const cases = [
    ["pages/profile/index.wxml", "pages/profile/index.wxss", "profile-page"],
    [
      "pages/profile/settings/index.wxml",
      "pages/profile/settings/index.wxss",
      "profile-settings-page",
    ],
  ];
  for (const [wxmlFile, wxssFile, pageClass] of cases) {
    const wxml = read(wxmlFile);
    const wxss = read(wxssFile);
    assert.match(wxml, new RegExp(`class="${pageClass} container`));
    const block = selectorBlock(wxss, `.${pageClass}.theme-dark`);
    assert.match(block, /background:\s*var\(--body-gradient\)/);
  }
});

test("profile theme toggle uses a sun and moon slider", () => {
  const wxml = read("pages/profile/index.wxml");
  const wxss = read("app.wxss");
  const profileWxss = read("pages/profile/index.wxss");
  const headerIndex = wxml.indexOf('class="settings-card-header"');
  const toggleIndex = wxml.indexOf('class="theme-toggle {{isDarkTheme');
  const firstRowIndex = wxml.indexOf('class="settings-row"');

  assert.notEqual(headerIndex, -1);
  assert.ok(toggleIndex > headerIndex);
  assert.ok(toggleIndex < firstRowIndex);
  assert.match(wxml, /class="theme-toggle \{\{isDarkTheme/);
  assert.match(wxml, /theme-toggle-sun">☀<\/text>/);
  assert.match(wxml, /theme-toggle-moon">☾<\/text>/);
  assert.match(wxml, /class="theme-toggle-thumb"/);
  assert.doesNotMatch(wxml, /黑夜模式/);
  assert.doesNotMatch(wxml, /夜间浏览切换为暗色界面/);
  assert.doesNotMatch(wxml, /themeToggleText/);

  const header = selectorBlock(profileWxss, ".settings-card-header");
  assert.match(header, /justify-content:\s*space-between/);

  const toggle = selectorBlock(wxss, ".theme-toggle");
  assert.match(toggle, /width:\s*136rpx/);
  assert.match(toggle, /border-radius:\s*999rpx/);
  const thumb = selectorBlock(wxss, ".theme-toggle-thumb");
  assert.match(thumb, /position:\s*absolute/);
  assert.match(thumb, /border-radius:\s*999rpx/);
  const darkThumb = selectorBlock(
    wxss,
    ".theme-toggle.is-dark .theme-toggle-thumb",
  );
  assert.match(darkThumb, /transform:\s*translateX\(68rpx\)/);
});

test("inner cards and panels use homepage inner surface tokens", () => {
  const cases = [
    ["pages/index/index.wxss", [".stat-card", ".skill-card"]],
    ["pages/gears/index.wxss", [".stat-card", ".metric"]],
    ["pages/login/index.wxss", [".auth-panel"]],
    ["pages/gears/detail/index.wxss", [".summary-item"]],
    ["pages/gears/form/index.wxss", [".field", ".picker-row"]],
    ["pages/skills/index.wxss", [".category-icon", ".skill-thumb"]],
    ["pages/profile/index.wxss", [".settings-row"]],
    ["pages/profile/settings/index.wxss", [".settings-row"]],
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
  assert.match(wxml, /loadingMore/);
  assert.match(wxml, /bindtap="cacheAllKnots"/);
  assert.match(wxml, /缓存全部/);
  assert.match(wxml, /preparingKnotCache/);
  assert.match(wxml, /正在统计缓存信息/);
  assert.match(wxml, /class="cache-status-card"/);
  assert.match(wxml, /离线模式也能查询绳结详情和动图/);

  const filterCard = selectorBlock(wxss, ".knot-filter-card");
  assert.match(filterCard, /border-radius:\s*var\(--card-radius\)/);
  assert.match(filterCard, /background:\s*var\(--surface-color\)/);
  const searchInput = selectorBlock(wxss, ".knot-search-input");
  assert.match(searchInput, /background:\s*var\(--control-bg\)/);
  const filterPill = selectorBlock(wxss, ".category-filter-pill");
  assert.match(filterPill, /background:\s*var\(--control-bg\)/);
  const cacheButton = selectorBlock(wxss, ".cache-button");
  assert.match(cacheButton, /background:\s*var\(--brand-color\)/);

  assert.match(ts, /categoryFilters/);
  assert.match(ts, /categoryFilterLabels/);
  assert.match(ts, /const KNOTS_PAGE_SIZE = 10/);
  assert.match(ts, /knotListRequestSeq/);
  assert.match(ts, /searchQuery,\s*nextOffset:\s*null,\s*loadingMore:\s*false/);
  assert.match(
    ts,
    /const favoriteIds = await favoriteIdsPromise;[\s\S]*response\.items\.map\(\(item\) => mapKnotListCard\(item, favoriteIds\)\)/,
  );
  assert.match(ts, /onReachBottom\(\)/);
  assert.match(ts, /onTabItemTap\(\)/);
  assert.match(ts, /ensureSkillsPageReady\(\)/);
  assert.match(ts, /wx\.pageScrollTo\(\{ scrollTop: 0, duration: 0 \}\)/);
  assert.match(
    ts,
    /wx\.getStorageSync\(KNOT_CACHE_ENTRY_KEY\) === true[\s\S]*this\.openKnotsFromEntry\(\);[\s\S]*return;/,
  );
  assert.match(
    ts,
    /mode: "catalog"[\s\S]*offlineNotice: ""[\s\S]*loadingMore: false/,
  );
  assert.match(ts, /loadMoreKnots/);
  assert.match(ts, /preparingKnotCache/);
  assert.match(ts, /prepareAllKnotsOfflineCache/);
  assert.match(ts, /cacheAllKnotsForOffline/);
  assert.match(ts, /预计约/);
  assert.match(ts, /formatBytes/);
  assert.doesNotMatch(ts, /loadAllKnots/);
  assert.match(
    ts,
    /item\.categories\.map\(\(category\) => category\.id \|\| category\.slug\)/,
  );
  assert.match(ts, /knot\.categoryIds\.includes\(selectedCategoryId\)/);
  assert.match(ts, /q:\s*normalizeOptionalFilter\(searchQuery\)/);
  assert.match(ts, /category:[\s\S]*selectedCategoryId !== "all"/);
  assert.match(ts, /item\.types\.flatMap/);
  assert.match(ts, /buildKnotSearchText/);
  assert.match(ts, /normalizeKnotSearchText/);
  assert.match(ts, /item\.id/);
  assert.match(ts, /item\.slug/);
  assert.match(ts, /description/);
  assert.match(ts, /steps/);
  assert.match(ts, /category\.slug/);
  assert.match(ts, /type\.slug/);
});

test("skills pages expose favorite list and star controls", () => {
  const indexWxml = read("pages/skills/index.wxml");
  const indexWxss = read("pages/skills/index.wxss");
  const indexTs = read("pages/skills/index.ts");
  const detailWxml = read("pages/skills/detail/index.wxml");
  const detailWxss = read("pages/skills/detail/index.wxss");
  const detailTs = read("pages/skills/detail/index.ts");

  assert.match(indexWxml, /收藏清单/);
  assert.match(indexWxml, /bindtap="openFavoriteSkills"/);
  assert.match(indexWxml, /bindchange="onFavoriteFilterChange"/);
  assert.match(indexWxml, /range="{{favoriteFilterLabels}}"/);
  assert.match(indexWxml, /class="favorite-button/);
  assert.match(indexWxml, /catchtap="toggleFavorite"/);
  assert.match(indexWxml, /loginPrompt\.visible/);
  assert.match(indexTs, /listFavoriteSkills/);
  assert.match(indexTs, /favoriteKnot/);
  assert.match(indexTs, /unfavoriteKnot/);
  assert.match(indexTs, /requireLoginForAction/);
  assert.match(indexTs, /showOfflineWriteBlockedToast/);

  const favoriteEntry = selectorBlock(indexWxss, ".favorite-entry-card");
  assert.match(favoriteEntry, /border-radius:\s*var\(--card-radius\)/);
  assert.match(favoriteEntry, /background:\s*var\(--surface-color\)/);
  const favoriteButton = selectorBlock(indexWxss, ".favorite-button");
  assert.match(favoriteButton, /border-radius:\s*999rpx/);

  assert.match(detailWxml, /class="favorite-detail-button/);
  assert.match(detailWxml, /bindtap="toggleFavorite"/);
  assert.match(detailWxml, /loginPrompt\.visible/);
  assert.match(detailTs, /getFavoriteKnotStatus/);
  assert.match(detailTs, /favoriteKnot/);
  assert.match(detailTs, /unfavoriteKnot/);
  const detailButton = selectorBlock(detailWxss, ".favorite-detail-button");
  assert.match(detailButton, /border-radius:\s*999rpx/);
});
