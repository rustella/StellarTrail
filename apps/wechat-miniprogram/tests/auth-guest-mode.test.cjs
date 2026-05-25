const test = require("node:test");
const assert = require("node:assert/strict");

let loginCalled = false;
let requests = [];

global.getApp = () => ({
  globalData: {
    apiBaseUrl: "https://api.example.test/",
  },
});

global.wx = {
  getStorageSync: () => undefined,
  setStorageSync: () => {},
  removeStorageSync: () => {},
  login: () => {
    loginCalled = true;
    throw new Error("wx.login should not be called in guest API paths");
  },
  switchTab: (options) => options.success && options.success({}),
  redirectTo: (options) => options.success && options.success({}),
  request: (options) => {
    requests.push(options);
    if (options.url.includes("/api/v1/skills/knots/detail/")) {
      options.success({
        statusCode: 200,
        data: {
          id: "bowline-knot",
          title: "布林结",
          summary: "固定绳圈",
          categories: [],
          types: [],
          media: [],
          steps: [],
        },
      });
      return;
    }
    options.success({
      statusCode: 200,
      data: {
        items: [],
        next_cursor: null,
        total: 0,
      },
    });
  },
};

const {
  ensureAccessToken,
  listFavoriteSkills,
  getKnotDetail,
  isLoginRequiredError,
  listKnots,
} = require("../.tmp-test/utils/api.js");
const { loginPageUrl } = require("../.tmp-test/utils/auth-prompt.js");

test("ensureAccessToken asks the page to show login instead of silently calling wx.login", async () => {
  loginCalled = false;

  await assert.rejects(
    () => ensureAccessToken(),
    (error) => {
      assert.equal(isLoginRequiredError(error), true);
      assert.equal(error.message, "登录后继续");
      return true;
    },
  );

  assert.equal(loginCalled, false);
});

test("public knot list and detail requests stay unauthenticated for guest users", async () => {
  loginCalled = false;
  requests = [];

  await listKnots({ offset: 0, limit: 2 }, "zh-CN");
  await getKnotDetail("bowline knot", "zh-CN");

  assert.equal(loginCalled, false);
  assert.equal(requests.length, 2);
  assert.equal(
    requests[0].url,
    "https://api.example.test/api/v1/skills/knots/list?offset=0&limit=2",
  );
  assert.equal(
    requests[1].url,
    "https://api.example.test/api/v1/skills/knots/detail/bowline%20knot",
  );
  for (const request of requests) {
    assert.equal(request.header.authorization, undefined);
    assert.equal(request.header["X-StellarTrail-Locale"], "zh-CN");
  }
});

test("favorite skills request requires login without silently calling wx.login", async () => {
  loginCalled = false;
  requests = [];

  await assert.rejects(
    () => listFavoriteSkills({ skill_category: "knots" }),
    (error) => {
      assert.equal(isLoginRequiredError(error), true);
      assert.equal(error.message, "登录后继续");
      return true;
    },
  );

  assert.equal(loginCalled, false);
  assert.deepEqual(requests, []);
});

test("login prompt encodes the current page as login redirect", () => {
  assert.equal(
    loginPageUrl("/pages/gears/form/index?template=backpacking-basic"),
    "/pages/login/index?redirect=%2Fpages%2Fgears%2Fform%2Findex%3Ftemplate%3Dbackpacking-basic",
  );
});

test("navigation helper rejects external redirects and switches tab pages", () => {
  const calls = [];
  global.wx.switchTab = (options) =>
    calls.push({ type: "switchTab", url: options.url });
  global.wx.redirectTo = (options) =>
    calls.push({ type: "redirectTo", url: options.url });
  const {
    decodeRedirect,
    navigateToRedirect,
  } = require("../.tmp-test/utils/navigation.js");

  assert.equal(
    decodeRedirect(encodeURIComponent("https://example.com")),
    "/pages/profile/index",
  );
  assert.equal(
    decodeRedirect(encodeURIComponent("/pages/gears/detail/index?id=g1")),
    "/pages/gears/detail/index?id=g1",
  );

  navigateToRedirect("/pages/profile/index");
  navigateToRedirect("/pages/gears/detail/index?id=g1");

  assert.deepEqual(calls, [
    { type: "switchTab", url: "/pages/profile/index" },
    { type: "redirectTo", url: "/pages/gears/detail/index?id=g1" },
  ]);
});
