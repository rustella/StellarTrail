const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const appRoot = path.resolve(__dirname, "..");
const miniRoot = path.join(appRoot, "miniprogram");

function read(rel) {
  return fs.readFileSync(path.join(miniRoot, rel), "utf8");
}

test("register page is available from the mini program page registry", () => {
  const config = JSON.parse(read("app.json"));
  assert.ok(config.pages.includes("pages/register/index"));
});

test("login page offers WeChat account email-code and password-reset entry points", () => {
  const wxml = read("pages/login/index.wxml");
  const ts = read("pages/login/index.ts");
  const pageSource = `${wxml}
${ts}`;
  assert.match(wxml, /微信登录/);
  assert.match(wxml, /账号登录/);
  assert.match(wxml, /邮箱登陆/);
  assert.doesNotMatch(wxml, /使用微信身份快速进入/);
  assert.match(wxml, /找回密码/);
  assert.match(wxml, /重设密码并登录/);
  assert.match(wxml, /placeholder="账号或邮箱"/);
  assert.match(wxml, /placeholder="邮箱验证码"/);
  assert.match(wxml, /password="{{true}}"/);
  assert.match(wxml, /注册账号/);
  assert.match(pageSource, /WECHAT_PROFILE_PROMPT_PENDING_KEY/);
  assert.match(pageSource, /markWechatProfilePromptPending/);
  assert.match(pageSource, /afterLoginSuccess\("\/pages\/index\/index"\)/);
  assert.doesNotMatch(wxml, /open-type="chooseAvatar"/);
  assert.doesNotMatch(wxml, /type="nickname"/);
  assert.doesNotMatch(wxml, /导入并登录/);
  assert.match(pageSource, /sendLoginCode/);
  assert.match(pageSource, /sendResetCode/);
  assert.doesNotMatch(wxml, /API|后端|接口|游客|免登录|写操作|模板/);
});

test("home page prompts to import WeChat profile after login", () => {
  const wxml = read("pages/index/index.wxml");
  const ts = read("pages/index/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.match(wxml, /open-type="chooseAvatar"/);
  assert.match(wxml, /type="nickname"/);
  assert.match(wxml, /导入资料/);
  assert.match(wxml, /跳过/);
  assert.match(pageSource, /showWechatProfilePromptIfNeeded/);
  assert.match(pageSource, /importWechatProfile/);
  assert.match(pageSource, /skipWechatProfileImport/);
  assert.match(pageSource, /uploadWechatAvatar/);
  assert.match(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.match(pageSource, /WECHAT_PROFILE_PROMPT_PENDING_KEY/);
});

test("profile page renders stored WeChat nickname and avatar", () => {
  const wxml = read("pages/profile/index.wxml");
  const ts = read("pages/profile/index.ts");
  const pageSource = `${wxml}
${ts}`;

  assert.match(wxml, /accountProfile\.avatarUrl/);
  assert.match(wxml, /accountProfile\.displayName/);
  assert.match(wxml, /wx:if="{{loggedIn}}"/);
  assert.match(wxml, /account-avatar-image/);
  assert.match(wxml, /open-type="chooseAvatar"/);
  assert.match(wxml, /bindchooseavatar="onChooseWechatAvatar"/);
  assert.doesNotMatch(wxml, /account-avatar-badge/);
  assert.match(wxml, /type="nickname"/);
  assert.match(wxml, /修改名称/);
  assert.match(wxml, /点这里后，在下方选择“用微信昵称”/);
  assert.doesNotMatch(wxml, /导入微信名称/);
  assert.doesNotMatch(wxml, /使用自定义名称/);
  assert.doesNotMatch(wxml, /导入昵称/);
  assert.doesNotMatch(wxml, /bindtap="saveNickname"/);
  assert.match(wxml, /bindtap="openNicknameModal"/);
  assert.doesNotMatch(wxml, /bindtap="importWechatNickname"/);
  assert.match(wxml, /bindsubmit="saveWechatNickname"/);
  assert.match(wxml, /name="nickname"/);
  assert.match(wxml, /bindnicknamereview="onWechatNicknameReview"/);
  assert.doesNotMatch(wxml, /bindtap="useCustomNickname"/);
  assert.match(pageSource, /buildAccountProfile/);
  assert.match(pageSource, /getCurrentUser/);
  assert.doesNotMatch(pageSource, /getUserProfile/);
  assert.match(pageSource, /loginWithWechat\(\{ nickname \}\)/);
  assert.match(pageSource, /WechatNicknameSubmitEvent/);
  assert.doesNotMatch(pageSource, /nicknameEditMode/);
  assert.match(pageSource, /saveNicknameValue/);
  assert.match(pageSource, /uploadWechatAvatar/);
  assert.match(pageSource, /isNotFoundApiError/);
  assert.match(pageSource, /normalizeProfileNickname/);
  assert.match(pageSource, /"寻径星野用户", "微信用户", "WeChat User"/);
  assert.match(pageSource, /头像保存暂不可用/);
  assert.match(pageSource, /normalizeProfileNickname\(user\.nickname\)/);
  assert.match(pageSource, /nickname \|\| user\.username \|\| user\.email/);
  assert.match(pageSource, /user\.avatar_url \|\| ""/);
});

test("register page captures account email verification and password confirmation", () => {
  const wxml = read("pages/register/index.wxml");
  const ts = read("pages/register/index.ts");
  const pageSource = `${wxml}
${ts}`;
  assert.match(wxml, /创建账号/);
  assert.match(wxml, /placeholder="账号"/);
  assert.match(wxml, /placeholder="邮箱"/);
  assert.match(wxml, /placeholder="邮箱验证码"/);
  assert.match(wxml, /placeholder="密码"/);
  assert.match(wxml, /placeholder="确认密码"/);
  assert.match(pageSource, /获取验证码/);
  assert.match(wxml, /注册并登录/);
  assert.match(wxml, /返回登录/);
  assert.doesNotMatch(wxml, /API|后端|接口|游客|免登录|写操作|模板/);
});

test("register page styles include dark theme surfaces", () => {
  const wxss = read("pages/register/index.wxss");
  assert.match(wxss, /theme-dark/);
  assert.match(wxss, /var\(--surface-color\)/);
  assert.match(wxss, /var\(--brand-color\)/);
});

test("home gear summary aligns logged-out and logged-in card surfaces", () => {
  const wxml = read("pages/index/index.wxml");
  const ts = read("pages/index/index.ts");
  const wxss = read("pages/index/index.wxss");
  const guestInlineBlock =
    wxss.match(/\.guest-inline \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.match(wxml, /showLoginForGearSummary/);
  assert.match(ts, /LOCKED_GEAR_STATS/);
  assert.match(ts, /登录后可见/);
  assert.match(ts, /快速添加新装备/);
  assert.doesNotMatch(ts, /登录后快速记录装备/);
  assert.match(ts, /value: "—"/);
  assert.match(
    guestInlineBlock,
    /border: 1rpx solid var\(--soft-border-color\)/,
  );
  assert.match(guestInlineBlock, /background: var\(--control-bg\)/);
  assert.doesNotMatch(guestInlineBlock, /var\(--notice-bg\)/);
});

test("gear page logged-out and logged-in cards share surface tokens", () => {
  const wxss = read("pages/gears/index.wxss");
  const cardSurfaceBlock =
    wxss.match(/\.tab-card,[\s\S]*?\.gear-card \{[\s\S]*?\n\}/)?.[0] ?? "";
  const metricBlock = wxss.match(/\.metric \{[\s\S]*?\n\}/)?.[0] ?? "";
  const metricValueBlock =
    wxss.match(/\.metric-value \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.match(cardSurfaceBlock, /border: 1rpx solid var\(--border-color\)/);
  assert.match(cardSurfaceBlock, /background: var\(--surface-color\)/);
  assert.match(cardSurfaceBlock, /box-shadow: var\(--shadow-soft\)/);
  assert.doesNotMatch(cardSurfaceBlock, /background: #ffffff/);
  assert.match(metricBlock, /background: var\(--control-bg\)/);
  assert.match(metricValueBlock, /color: var\(--text-color\)/);
});

test("gear form cancel falls back when there is no previous page", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const wxss = read("pages/gears/form/index.wxss");
  const ts = read("pages/gears/form/index.ts");
  const formActionsBlock =
    wxss.match(/\.form-actions \{[\s\S]*?\n\}/)?.[0] ?? "";

  assert.match(wxml, /bindtap="cancel"/);
  assert.ok(
    wxml.lastIndexOf('class="form-actions"') >
      wxml.lastIndexOf('class="form-card"'),
  );
  assert.doesNotMatch(wxml, /footer-actions/);
  assert.match(wxss, /\.form-actions \{/);
  assert.match(wxss, /env\(safe-area-inset-bottom\)/);
  assert.doesNotMatch(formActionsBlock, /position: fixed/);
  assert.match(ts, /getCurrentPages\(\)\.length > 1/);
  assert.match(ts, /wx\.navigateBack\(\)/);
  assert.match(ts, /wx\.switchTab\(\{ url: "\/pages\/gears\/index" \}\)/);
  assert.match(ts, /wx\.redirectTo\(\{/);
  assert.match(ts, /\/pages\/gears\/detail\/index\?id=/);
});

test("gear form offers purchase location presets and custom input", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const wxss = read("pages/gears/form/index.wxss");
  const ts = read("pages/gears/form/index.ts");

  assert.match(wxml, /选择或输入购买渠道/);
  assert.match(wxml, /purchaseLocationSheetVisible/);
  assert.match(wxml, /purchaseLocationOptions/);
  assert.match(wxml, /自定义输入/);
  assert.match(wxml, /自定义购买渠道/);
  assert.match(wxml, /bindtap="openPurchaseLocationSheet"/);
  assert.match(wxml, /bindtap="selectPurchaseLocation"/);
  assert.match(wxml, /bindtap="openCustomPurchaseLocation"/);
  assert.match(wxml, /bindtap="saveCustomPurchaseLocation"/);
  assert.doesNotMatch(wxml, /data-field="purchaseLocation"/);
  assert.match(ts, /PURCHASE_LOCATION_OPTIONS/);
  assert.match(ts, /selectPurchaseLocation/);
  assert.match(ts, /saveCustomPurchaseLocation/);
  assert.match(ts, /cancelCustomPurchaseLocation/);
  assert.match(ts, /customPurchaseLocationText\.trim\(\)/);
  assert.match(wxss, /\.purchase-location-sheet/);
  assert.match(wxss, /\.purchase-location-option\.selected/);
});

test("gear form offers colored tag chips and suggestions", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const wxss = read("pages/gears/form/index.wxss");
  const ts = read("pages/gears/form/index.ts");
  const utils = read("utils/gear-utils.ts");

  assert.match(wxml, /添加标签/);
  assert.match(wxml, /tagSheetVisible/);
  assert.match(wxml, /tagSuggestions/);
  assert.match(wxml, /tagColorOptions/);
  assert.match(wxml, /tag-chip/);
  assert.match(wxml, /tag-color-option/);
  assert.match(wxml, /随机/);
  assert.doesNotMatch(wxml, /data-field="tagsText"/);
  assert.doesNotMatch(wxml, /用逗号分隔/);
  assert.match(ts, /getGearTagSuggestions/);
  assert.match(ts, /selectTagSuggestion/);
  assert.match(ts, /saveCustomTag/);
  assert.match(ts, /removeTag/);
  assert.match(utils, /export type GearTagColor =/);
  assert.match(utils, /GEAR_TAG_COLOR_OPTIONS/);
  assert.match(wxss, /\.tag-color-teal/);
  assert.match(wxss, /\.tag-color-blue/);
});

test("gear atlas submission moved from gear form to detail and atlas pages", () => {
  const formWxml = read("pages/gears/form/index.wxml");
  const detailWxml = read("pages/gears/detail/index.wxml");
  const detailTs = read("pages/gears/detail/index.ts");
  const atlasListWxml = read("pages/gear-atlas/index.wxml");
  const atlasSubmitWxml = read("pages/gear-atlas/submit/index.wxml");
  const appJson = read("app.json");

  assert.doesNotMatch(formWxml, /共享给搭子参考/);
  assert.doesNotMatch(formWxml, /申请共享/);
  assert.doesNotMatch(formWxml, /confirmEnableShare/);
  assert.match(detailWxml, /装备图鉴投稿/);
  assert.match(detailWxml, /投稿到图鉴/);
  assert.match(detailTs, /submitGearToAtlas/);
  assert.match(
    detailTs,
    /只会复制分类、名称、品牌、型号、描述、重量、官方价和分类参数/,
  );
  assert.match(atlasListWxml, /装备图鉴/);
  assert.match(atlasSubmitWxml, /提交审核/);
  assert.match(appJson, /pages\/gear-atlas\/index/);
});

test("gear form exposes selectable weight and spec units", () => {
  const wxml = read("pages/gears/form/index.wxml");
  const ts = read("pages/gears/form/index.ts");
  const utils = read("utils/gear-utils.ts");

  assert.match(wxml, /<view class="field-label">重量<\/view>/);
  assert.match(wxml, /weightUnitLabels/);
  assert.match(wxml, /bindchange="onWeightUnitChange"/);
  assert.doesNotMatch(wxml, /重量（kg）/);
  assert.match(ts, /GEAR_WEIGHT_UNIT_OPTIONS/);
  assert.match(ts, /onWeightUnitChange/);
  assert.match(
    utils,
    /export type GearWeightUnit = "kg" \| "g" \| "lb" \| "oz"/,
  );
  assert.match(utils, /const CAPACITY_UNITS = \["L", "ml", "fl oz"\]/);
  assert.match(utils, /const LOAD_UNITS = \["kg", "g", "lb"\]/);
  assert.match(utils, /const LENGTH_UNITS = \["cm", "m", "mm", "in"\]/);
  assert.match(utils, /const BACK_LENGTH_UNITS = \["cm", "in"\]/);
  assert.match(
    utils,
    /const BACKPACK_SIZE_UNITS = \["", "XS", "S", "M", "L", "XL", "XXL", "均码"\]/,
  );
  assert.match(wxml, /choice-value/);
  assert.match(wxml, /item\.choiceOnly/);
  assert.match(
    utils,
    /const SHOE_SIZE_OR_LENGTH_UNITS = \["cm", "EU", "US", "UK", "in"\]/,
  );
});
