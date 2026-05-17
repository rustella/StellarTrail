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

test("login page offers both WeChat and account-password login without stale implementation wording", () => {
  const wxml = read("pages/login/index.wxml");
  assert.match(wxml, /微信一键登录/);
  assert.match(wxml, /账号密码登录/);
  assert.match(wxml, /placeholder="账号或邮箱"/);
  assert.match(wxml, /password="{{true}}"/);
  assert.match(wxml, /注册账号/);
  assert.doesNotMatch(wxml, /API|后端|接口|游客|免登录|写操作|模板/);
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
