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
    options.success({ statusCode: 200, data: loginResponse("access-pass", "refresh-pass") });
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
    calls.push({ url: options.url, method: options.method, data: options.data });
    assert.equal(options.url, "https://api.example.test/api/auth/register");
    options.success({ statusCode: 200, data: loginResponse("access-register", "refresh-register") });
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
  const { createCaptcha, sendEmailVerificationCode } = require("../.tmp-test/utils/api.js");

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
