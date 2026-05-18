const test = require("node:test");
const assert = require("node:assert/strict");

function installWxMock(handler, uploadHandler, extraHandlers = {}) {
  const storage = new Map();
  global.getApp = () => ({
    globalData: { apiBaseUrl: "https://api.example.test" },
  });
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
    getNetworkType(options) {
      options.success({ networkType: "wifi" });
    },
    onNetworkStatusChange() {},
    uploadFile(options) {
      if (!uploadHandler) {
        throw new Error("unexpected wx.uploadFile call");
      }
      uploadHandler(options, storage);
    },
    downloadFile(options) {
      if (!extraHandlers.downloadFile) {
        throw new Error("unexpected wx.downloadFile call");
      }
      extraHandlers.downloadFile(options, storage);
    },
    saveFile(options) {
      if (!extraHandlers.saveFile) {
        throw new Error("unexpected wx.saveFile call");
      }
      extraHandlers.saveFile(options, storage);
    },
    getFileInfo(options) {
      if (!extraHandlers.getFileInfo) {
        options.success({ size: 128 });
        return;
      }
      extraHandlers.getFileInfo(options, storage);
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
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
    });
    assert.equal(options.url, "https://api.example.test/api/auth/wechat-login");
    options.success({
      statusCode: 200,
      data: loginResponse("access-new", "refresh-new"),
    });
  });
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(loginWithWechat());
  assert.deepEqual(calls[0].data, { code: "wx-login-code" });
  assert.equal(storage.get("stellartrail_access_token"), "access-new");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-new");
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "小程序用户",
    avatar_url: null,
  });
});

test("loginWithWechat sends provided profile without default nickname", async () => {
  const calls = [];
  installWxMock((options) => {
    calls.push({ url: options.url, data: options.data });
    options.success({
      statusCode: 200,
      data: loginResponse("access-profile", "refresh-profile"),
    });
  });
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(
    loginWithWechat({ nickname: " 微信昵称 ", avatar_url: "" }),
  );

  assert.deepEqual(calls[0].data, {
    code: "wx-login-code",
    profile: { nickname: "微信昵称" },
  });
});

test("uploadWechatAvatar uploads with bearer token and stores returned user", async () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    (options) => {
      assert.equal(
        options.url,
        "https://api.example.test/api/me/profile/avatar",
      );
      assert.equal(options.filePath, "/tmp/avatar.png");
      assert.equal(options.name, "file");
      assert.equal(options.header.authorization, "Bearer access-old");
      options.success({
        statusCode: 200,
        data: JSON.stringify({
          user: {
            id: "u1",
            nickname: "小程序用户",
            avatar_url: "https://assets.example.test/avatar.png",
          },
        }),
      });
    },
  );
  storage.set("stellartrail_access_token", "access-old");
  const { uploadWechatAvatar } = require("../.tmp-test/utils/api.js");

  const user = await uploadWechatAvatar("/tmp/avatar.png");

  assert.equal(user.avatar_url, "https://assets.example.test/avatar.png");
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "小程序用户",
    avatar_url: "https://assets.example.test/avatar.png",
  });
});

test("getCurrentUser refreshes stored profile from backend", async () => {
  const storage = installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/me/profile");
    assert.equal(options.method, "GET");
    assert.equal(options.header.authorization, "Bearer access-old");
    options.success({
      statusCode: 200,
      data: {
        user: {
          id: "u1",
          nickname: "后端昵称",
          avatar_url: "https://assets.example.test/avatar.png",
        },
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  const { getCurrentUser } = require("../.tmp-test/utils/api.js");

  const user = await getCurrentUser();

  assert.equal(user.nickname, "后端昵称");
  assert.equal(user.avatar_url, "https://assets.example.test/avatar.png");
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "后端昵称",
    avatar_url: "https://assets.example.test/avatar.png",
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
    if (
      options.url.endsWith("/api/me/gears/stats?tab=available") &&
      calls.length === 1
    ) {
      options.success({ statusCode: 401, data: { code: "unauthorized" } });
      return;
    }
    if (options.url.endsWith("/api/auth/refresh")) {
      assert.deepEqual(options.data, { refresh_token: "refresh-old" });
      options.success({
        statusCode: 200,
        data: loginResponse("access-new", "refresh-new"),
      });
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
    [
      "/api/me/gears/stats?tab=available",
      "/api/auth/refresh",
      "/api/me/gears/stats?tab=available",
    ],
  );
  assert.equal(calls[0].authorization, "Bearer access-old");
  assert.equal(calls[2].authorization, "Bearer access-new");
  assert.equal(storage.get("stellartrail_access_token"), "access-new");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-new");
});

test("getGearSpecKeyRankings calls authenticated category endpoint", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: { keys: ["battery_capacity", "rated_energy"] },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  const { getGearSpecKeyRankings } = require("../.tmp-test/utils/api.js");

  const response = await getGearSpecKeyRankings("electronics_system");

  assert.deepEqual(response, { keys: ["battery_capacity", "rated_energy"] });
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/me/gears/spec-key-rankings?category=electronics_system",
      authorization: "Bearer access-old",
    },
  ]);
});

test("getGearTagSuggestions calls authenticated suggestion endpoint", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: { items: [{ tag: "冬季", color: "blue" }] },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  const { getGearTagSuggestions } = require("../.tmp-test/utils/api.js");

  const response = await getGearTagSuggestions(12);

  assert.deepEqual(response, { items: [{ tag: "冬季", color: "blue" }] });
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/me/gears/tag-suggestions?limit=12",
      authorization: "Bearer access-old",
    },
  ]);
});

test("loginWithPassword persists access and refresh tokens", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    assert.equal(options.url, "https://api.example.test/api/auth/login");
    options.success({
      statusCode: 200,
      data: loginResponse("access-pass", "refresh-pass"),
    });
  });
  const { loginWithPassword } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(
    loginWithPassword({ account: "trail_alice", password: "OutdoorPass123!" }),
  );

  assert.equal(calls.length, 1);
  assert.equal(calls[0].method, "POST");
  assert.deepEqual(calls[0].data, {
    account: "trail_alice",
    password: "OutdoorPass123!",
  });
  assert.equal(calls[0].authorization, undefined);
  assert.equal(storage.get("stellartrail_access_token"), "access-pass");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-pass");
});

test("registerWithPassword persists the returned session", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
    });
    assert.equal(options.url, "https://api.example.test/api/auth/register");
    options.success({
      statusCode: 200,
      data: loginResponse("access-register", "refresh-register"),
    });
  });
  const { registerWithPassword } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(
    registerWithPassword({
      username: "trail_bob",
      email: "bob@example.com",
      password: "OutdoorPass123!",
      confirm_password: "OutdoorPass123!",
      email_verification_code: "123456",
    }),
  );

  assert.equal(calls.length, 1);
  assert.equal(calls[0].method, "POST");
  assert.deepEqual(calls[0].data, {
    username: "trail_bob",
    email: "bob@example.com",
    password: "OutdoorPass123!",
    confirm_password: "OutdoorPass123!",
    email_verification_code: "123456",
  });
  assert.equal(storage.get("stellartrail_access_token"), "access-register");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-register");
});

test("email code and captcha helpers call public auth endpoints", async () => {
  const calls = [];
  installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    if (options.url.endsWith("/api/auth/email-verification-code")) {
      options.success({
        statusCode: 200,
        data: {
          email: "bob@example.com",
          expires_at: "2026-06-01T02:10:00Z",
          debug_code: "123456",
        },
      });
      return;
    }
    if (options.url.endsWith("/api/auth/captcha")) {
      options.success({
        statusCode: 200,
        data: {
          captcha_ticket: "ticket-1",
          captcha_type: "image",
          image_svg: "<svg />",
          expires_at: "2026-06-01T02:05:00Z",
          debug_answer: "7K2Q",
        },
      });
      return;
    }
    options.success({ statusCode: 404, data: { message: "not found" } });
  });
  const {
    createCaptcha,
    sendEmailVerificationCode,
  } = require("../.tmp-test/utils/api.js");

  const email = await sendEmailVerificationCode("bob@example.com");
  const captcha = await createCaptcha("trail_bob");

  assert.equal(email.debug_code, "123456");
  assert.equal(captcha.captcha_ticket, "ticket-1");
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/auth/email-verification-code",
      method: "POST",
      data: { email: "bob@example.com" },
      authorization: undefined,
    },
    {
      url: "https://api.example.test/api/auth/captcha",
      method: "POST",
      data: { account: "trail_bob" },
      authorization: undefined,
    },
  ]);
});

test("captcha required errors keep status code and response code", async () => {
  installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/auth/login");
    options.success({
      statusCode: 428,
      data: {
        code: "captcha_required",
        message: "请完成验证码后再试",
        captcha: { captcha_type: "image", endpoint: "/api/auth/captcha" },
      },
    });
  });
  const {
    isApiResponseError,
    isCaptchaRequiredError,
    loginWithPassword,
  } = require("../.tmp-test/utils/api.js");

  await assert.rejects(
    () => loginWithPassword({ account: "trail_bob", password: "bad-pass" }),
    (error) => {
      assert.equal(isApiResponseError(error), true);
      assert.equal(isCaptchaRequiredError(error), true);
      assert.equal(error.statusCode, 428);
      assert.equal(error.code, "captcha_required");
      assert.equal(error.captcha.endpoint, "/api/auth/captcha");
      return true;
    },
  );
});

test("not found API errors can be identified without exposing raw messages", async () => {
  const storage = installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/me/profile");
    options.success({
      statusCode: 404,
      data: { code: "not_found", message: "resource not found" },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  const {
    getCurrentUser,
    isNotFoundApiError,
  } = require("../.tmp-test/utils/api.js");

  await assert.rejects(
    () => getCurrentUser(),
    (error) => {
      assert.equal(isNotFoundApiError(error), true);
      assert.equal(error.statusCode, 404);
      return true;
    },
  );
});

test("cacheable GET responses are reused after network failure", async () => {
  let callCount = 0;
  installWxMock((options) => {
    callCount += 1;
    assert.equal(
      options.url,
      "https://api.example.test/api/skills/knots/list?offset=0&limit=2",
    );
    if (callCount === 1) {
      options.success({
        statusCode: 200,
        data: {
          locale: "zh-CN",
          items: [{ id: "bowline", title: "布林结", media: [] }],
          page: { limit: 2, offset: 0, next_offset: null },
        },
      });
      return;
    }
    options.fail({ errMsg: "request:fail" });
  });
  const {
    consumeOfflineCacheNotice,
    listKnots,
  } = require("../.tmp-test/utils/api.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  const online = await listKnots({ offset: 0, limit: 2 });
  const offline = await listKnots({ offset: 0, limit: 2 });

  assert.deepEqual(offline, online);
  assert.equal(
    consumeOfflineCacheNotice(),
    "当前离线，正在显示已缓存内容",
  );
});

test("cacheable GET rejects with an offline miss when no cache exists", async () => {
  installWxMock((options) => {
    options.fail({ errMsg: "request:fail" });
  });
  const {
    getErrorMessage,
    isOfflineCacheMissError,
    listKnots,
  } = require("../.tmp-test/utils/api.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  await assert.rejects(
    () => listKnots({ offset: 20, limit: 2 }),
    (error) => {
      assert.equal(isOfflineCacheMissError(error), true);
      assert.equal(getErrorMessage(error), "当前离线且暂无已缓存内容");
      return true;
    },
  );
});

test("API errors do not fall back to stale offline cache", async () => {
  let callCount = 0;
  installWxMock((options) => {
    callCount += 1;
    if (callCount === 1) {
      options.success({
        statusCode: 200,
        data: {
          locale: "zh-CN",
          items: [{ id: "figure-eight", title: "八字结", media: [] }],
          page: { limit: 1, offset: 0, next_offset: null },
        },
      });
      return;
    }
    options.success({
      statusCode: 500,
      data: { code: "server_error", message: "服务暂不可用" },
    });
  });
  const {
    isApiResponseError,
    listKnots,
  } = require("../.tmp-test/utils/api.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  await listKnots({ offset: 0, limit: 1 });

  await assert.rejects(
    () => listKnots({ offset: 0, limit: 1 }),
    (error) => {
      assert.equal(isApiResponseError(error), true);
      assert.equal(error.statusCode, 500);
      assert.equal(error.message, "服务暂不可用");
      return true;
    },
  );
});

test("clearLoginState removes user-scoped offline caches", async () => {
  const storage = installWxMock((options) => {
    assert.equal(
      options.url,
      "https://api.example.test/api/me/gears/stats?tab=available",
    );
    options.success({
      statusCode: 200,
      data: {
        current_count: 1,
        archived_count: 0,
        total_value_cents: 0,
        total_weight_g: 1200,
        by_category: [],
        by_status: [],
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-cache", nickname: "缓存用户" });
  const {
    clearLoginState,
    getGearStats,
  } = require("../.tmp-test/utils/api.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  await getGearStats("available");
  assert.equal(hasOfflineCacheStorage(storage), true);

  clearLoginState();

  assert.equal(hasOfflineCacheStorage(storage), false);
});

test("media cache returns saved files and ignores missing files", async () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    undefined,
    {
      downloadFile(options) {
        assert.equal(options.url, "https://assets.example.test/knot.gif");
        options.success({ statusCode: 200, tempFilePath: "/tmp/knot.gif" });
        options.complete();
      },
      saveFile(options) {
        assert.equal(options.tempFilePath, "/tmp/knot.gif");
        options.success({ savedFilePath: "wxfile://saved/knot.gif" });
      },
      getFileInfo(options) {
        if (options.filePath === "wxfile://saved/knot.gif") {
          options.success({ size: 128 });
          return;
        }
        options.fail({ errMsg: "file missing" });
      },
    },
  );
  const {
    cacheMediaUrl,
    resolveCachedMediaUrl,
  } = require("../.tmp-test/utils/media-cache.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  cacheMediaUrl("https://assets.example.test/knot.gif");
  assert.equal(
    await resolveCachedMediaUrl("https://assets.example.test/knot.gif"),
    "wxfile://saved/knot.gif",
  );

  const cachedKey = [...storage.keys()].find((key) =>
    String(key).startsWith("stellartrail_media_cache_v1:"),
  );
  storage.set(cachedKey, {
    url: "https://assets.example.test/knot.gif",
    filePath: "wxfile://missing/knot.gif",
    cachedAt: "2026-05-19T00:00:00.000Z",
  });

  assert.equal(
    await resolveCachedMediaUrl("https://assets.example.test/knot.gif"),
    "https://assets.example.test/knot.gif",
  );
});

function hasOfflineCacheStorage(storage) {
  return [...storage.keys()].some((key) =>
    String(key).startsWith("stellartrail_offline_cache_v1:"),
  );
}
