const test = require("node:test");
const assert = require("node:assert/strict");

function installWxMock(handler) {
  const storage = new Map();
  global.getApp = () => ({ globalData: { apiBaseUrl: "https://api.example.test" } });
  global.wx = {
    getStorageSync(key) {
      return storage.get(key);
    },
    setStorageSync(key, value) {
      storage.set(key, value);
    },
    removeStorageSync(key) {
      storage.delete(key);
    },
    login(options) {
      options.success({ code: "wx-login-code" });
    },
    request(options) {
      handler(options, storage);
    },
  };
  return storage;
}

function loginResponse(accessToken, refreshToken) {
  return {
    access_token: accessToken,
    expires_at: "2026-06-01T02:00:00Z",
    refresh_token: refreshToken,
    refresh_expires_at: "2026-07-01T00:00:00Z",
    user: { id: "u1", nickname: "小程序用户", avatar_url: null },
  };
}

test("loginWithWechat persists access and refresh tokens", async () => {
  const storage = installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/auth/wechat-login");
    options.success({ statusCode: 200, data: loginResponse("access-new", "refresh-new") });
  });
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(loginWithWechat());
  assert.equal(storage.get("stellartrail_access_token"), "access-new");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-new");
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "小程序用户",
    avatar_url: null,
  });
});

test("authenticated requests refresh once on 401 and retry with the new access token", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      authorization: options.header && options.header.authorization,
      data: options.data,
    });
    if (options.url.endsWith("/api/me/gears/stats?tab=available") && calls.length === 1) {
      options.success({ statusCode: 401, data: { code: "unauthorized" } });
      return;
    }
    if (options.url.endsWith("/api/auth/refresh")) {
      assert.deepEqual(options.data, { refresh_token: "refresh-old" });
      options.success({ statusCode: 200, data: loginResponse("access-new", "refresh-new") });
      return;
    }
    if (options.url.endsWith("/api/me/gears/stats?tab=available")) {
      options.success({
        statusCode: 200,
        data: {
          current_count: 0,
          archived_count: 0,
          total_value_cents: 0,
          total_weight_g: 0,
          by_category: [],
          by_status: [],
        },
      });
      return;
    }
    options.success({ statusCode: 404, data: { message: "not found" } });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_refresh_token", "refresh-old");
  const { getGearStats } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(getGearStats("available"));
  assert.deepEqual(
    calls.map((call) => call.url.replace("https://api.example.test", "")),
    ["/api/me/gears/stats?tab=available", "/api/auth/refresh", "/api/me/gears/stats?tab=available"],
  );
  assert.equal(calls[0].authorization, "Bearer access-old");
  assert.equal(calls[2].authorization, "Bearer access-new");
  assert.equal(storage.get("stellartrail_access_token"), "access-new");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-new");
});
