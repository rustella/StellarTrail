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
  request: (options) => {
    requests.push(options);
    if (options.url.includes("/api/skills/knots/detail/")) {
      options.success({
        statusCode: 200,
        data: {
          id: "bowline-knot",
          title: "布林结",
          summary: "固定绳圈",
          difficulty: "beginner",
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
    "https://api.example.test/api/skills/knots/list?offset=0&limit=2",
  );
  assert.equal(
    requests[1].url,
    "https://api.example.test/api/skills/knots/detail/bowline%20knot",
  );
  for (const request of requests) {
    assert.equal(request.header.authorization, undefined);
    assert.equal(request.header["X-StellarTrail-Locale"], "zh-CN");
  }
});

test("login prompt encodes the current page as login redirect", () => {
  assert.equal(
    loginPageUrl("/pages/gears/form/index?template=backpacking-basic"),
    "/pages/login/index?redirect=%2Fpages%2Fgears%2Fform%2Findex%3Ftemplate%3Dbackpacking-basic",
  );
});
