const test = require("node:test");
const assert = require("node:assert/strict");
const crypto = require("node:crypto");

function installWxMock(handler, uploadHandler, extraHandlers = {}) {
  clearCompiledUtilityModules();
  const storage = new Map();
  const globalData = {
    apiBaseUrl: "https://api.example.test",
    assetsBaseUrl: "https://assets.example.test",
    ...(extraHandlers.globalData ?? {}),
  };
  global.getApp = () => ({
    globalData,
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
    getRandomValues(array) {
      if (extraHandlers.getRandomValues) {
        return extraHandlers.getRandomValues(array);
      }
      for (let index = 0; index < array.length; index += 1) {
        array[index] = index + 1;
      }
      return array;
    },
    request(options) {
      handler(options, storage);
    },
    getNetworkType(options) {
      options.success({ networkType: "wifi" });
    },
    onNetworkStatusChange() {},
    getFileSystemManager() {
      return {
        readFile(options) {
          if (!extraHandlers.readFile) {
            throw new Error("unexpected file read");
          }
          extraHandlers.readFile(options, storage);
        },
      };
    },
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
    removeSavedFile(options) {
      if (!extraHandlers.removeSavedFile) {
        options.success?.();
        options.complete?.();
        return;
      }
      extraHandlers.removeSavedFile(options, storage);
    },
  };
  return storage;
}

function clearCompiledUtilityModules() {
  for (const key of Object.keys(require.cache)) {
    if (key.includes("/.tmp-test/utils/")) {
      delete require.cache[key];
    }
  }
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

function productionDomainCandidates() {
  return [
    {
      id: "stellartrail",
      apiBaseUrl: "https://api.example.invalid",
      assetsBaseUrl: "https://assets.example.invalid",
    },
    {
      id: "stellaris",
      apiBaseUrl: "https://api-alt1.example.invalid",
      assetsBaseUrl: "https://assets-alt1.example.invalid",
    },
    {
      id: "iwx",
      apiBaseUrl: "https://api-alt2.example.invalid",
      assetsBaseUrl: "https://assets-alt2.example.invalid",
    },
  ];
}

function parseRequestUrl(url) {
  const pathWithQuery = url.replace("https://api.example.test", "");
  const [path, query = ""] = pathWithQuery.split("?", 2);
  return { path, query };
}

function verifyRequestSignature({
  url,
  method,
  bodyHash,
  appId,
  appSecret,
  nonce,
  signature,
}) {
  const { path, query } = parseRequestUrl(url);
  const canonical = [
    "STELLARTRAIL-HMAC-SHA256",
    method,
    path,
    canonicalQueryWithoutSignature(query),
    bodyHash,
    appId,
    nonce,
  ].join("\n");
  assert.equal(
    signature,
    crypto.createHmac("sha256", appSecret).update(canonical).digest("hex"),
  );
}

function canonicalQueryWithoutSignature(query) {
  return query
    .split("&")
    .filter(Boolean)
    .map((pair) => {
      const [key, value = ""] = pair.split("=", 2);
      return [key, value];
    })
    .filter(([key]) => key !== "signature")
    .sort(
      ([leftKey, leftValue], [rightKey, rightValue]) =>
        compareCanonicalText(leftKey, rightKey) ||
        compareCanonicalText(leftValue, rightValue),
    )
    .map(([key, value]) => `${key}=${value}`)
    .join("&");
}

function compareCanonicalText(left, right) {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

function stableJson(value) {
  if (value === null || typeof value === "boolean") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    return JSON.stringify(Number.isFinite(value) ? value : null);
  }
  if (typeof value === "string") {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableJson(item)).join(",")}]`;
  }
  const entries = Object.keys(value)
    .sort()
    .map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`);
  return `{${entries.join(",")}}`;
}

function sha256Hex(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

test("loginWithWechat persists access and refresh tokens", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
    });
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/auth/wechat-login",
    );
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

test("loginWithWechat uses configured local code without wx.login", async () => {
  const calls = [];
  installWxMock(
    (options) => {
      calls.push(options.data);
      options.success({
        statusCode: 200,
        data: loginResponse("access-local-code", "refresh-local-code"),
      });
    },
    null,
    {
      globalData: {
        wechatLoginCode: " local-dev-user ",
      },
    },
  );
  global.wx.login = () => {
    throw new Error("wx.login should not be called");
  };
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(loginWithWechat());

  assert.deepEqual(calls, [{ code: "local-dev-user" }]);
});

test("request signature is injected into JSON requests without mutating business fields", async () => {
  const calls = [];
  installWxMock(
    (options) => {
      calls.push({
        url: options.url,
        method: options.method,
        data: options.data,
      });
      const { path, query } = parseRequestUrl(options.url);
      assert.equal(path, "/api/v1/auth/wechat-login");
      assert.equal(query, "");
      assert.equal(options.data.code, "wx-login-code");
      assert.equal(options.data.app_id, "wechat-client");
      assert.match(options.data.nonce, /^[a-z0-9]+-[0-9a-f]{32}$/);
      assert.match(options.data.signature, /^[0-9a-f]{64}$/);
      verifyRequestSignature({
        url: options.url,
        method: "POST",
        bodyHash: sha256Hex(stableJson({ code: "wx-login-code" })),
        appId: options.data.app_id,
        appSecret: "wechat-secret",
        nonce: options.data.nonce,
        signature: options.data.signature,
      });
      options.success({
        statusCode: 200,
        data: loginResponse("access-signed", "refresh-signed"),
      });
    },
    null,
    {
      globalData: {
        requestSignature: {
          app_id: "wechat-client",
          app_secret: "wechat-secret",
        },
      },
    },
  );
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(loginWithWechat());

  assert.deepEqual(
    Object.fromEntries(
      Object.entries(calls[0].data).filter(
        ([key]) => !["app_id", "nonce", "signature"].includes(key),
      ),
    ),
    { code: "wx-login-code" },
  );
});

test("request signature is injected into GET query parameters", async () => {
  installWxMock(
    (options) => {
      const { path, query } = parseRequestUrl(options.url);
      const params = new URLSearchParams(query);
      assert.equal(path, "/api/v1/client-versions");
      assert.equal(params.get("client_key"), "wechat_miniprogram");
      assert.equal(params.get("limit"), "20");
      assert.equal(params.get("app_id"), "wechat-client");
      assert.match(params.get("nonce"), /^[a-z0-9]+-[0-9a-f]{32}$/);
      assert.match(params.get("signature"), /^[0-9a-f]{64}$/);
      assert.equal(options.data, undefined);
      verifyRequestSignature({
        url: options.url,
        method: "GET",
        bodyHash: sha256Hex(Buffer.alloc(0)),
        appId: params.get("app_id"),
        appSecret: "wechat-secret",
        nonce: params.get("nonce"),
        signature: params.get("signature"),
      });
      options.success({
        statusCode: 200,
        data: { next_cursor: null, items: [] },
      });
    },
    null,
    {
      globalData: {
        requestSignature: {
          app_id: "wechat-client",
          app_secret: "wechat-secret",
        },
      },
    },
  );
  const { listClientVersions } = require("../.tmp-test/utils/api.js");

  const response = await listClientVersions("wechat_miniprogram", {
    limit: 20,
  });

  assert.deepEqual(response.items, []);
});

test("request signature nonce stays unique when random source returns zeros", async () => {
  const originalDateNow = Date.now;
  const originalMathRandom = Math.random;
  const originalCryptoDescriptor = Object.getOwnPropertyDescriptor(
    globalThis,
    "crypto",
  );
  const nonces = [];

  try {
    Date.now = () => 1780000000000;
    Math.random = () => 0;
    Object.defineProperty(globalThis, "crypto", {
      value: undefined,
      configurable: true,
    });
    installWxMock(
      (options) => {
        const { query } = parseRequestUrl(options.url);
        const params = new URLSearchParams(query);
        const nonce = params.get("nonce");
        nonces.push(nonce);
        assert.notEqual(
          nonce,
          "mppy1i4g-00000000000000000000000000000000",
        );
        verifyRequestSignature({
          url: options.url,
          method: "GET",
          bodyHash: sha256Hex(Buffer.alloc(0)),
          appId: params.get("app_id"),
          appSecret: "wechat-secret",
          nonce,
          signature: params.get("signature"),
        });
        options.success({
          statusCode: 200,
          data: { next_cursor: null, items: [] },
        });
      },
      null,
      {
        globalData: {
          requestSignature: {
            app_id: "wechat-client",
            app_secret: "wechat-secret",
          },
        },
        getRandomValues(array) {
          return array;
        },
      },
    );
    const { listClientVersions } = require("../.tmp-test/utils/api.js");

    await listClientVersions("wechat_miniprogram");
    await listClientVersions("wechat_miniprogram");
  } finally {
    Date.now = originalDateNow;
    Math.random = originalMathRandom;
    if (originalCryptoDescriptor) {
      Object.defineProperty(globalThis, "crypto", originalCryptoDescriptor);
    } else {
      delete globalThis.crypto;
    }
  }

  assert.equal(new Set(nonces).size, 2);
});

test("updateWechatNickname keeps the stored avatar in the profile payload", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({ url: options.url, data: options.data });
    options.success({
      statusCode: 200,
      data: {
        ...loginResponse("access-nickname", "refresh-nickname"),
        user: {
          id: "u1",
          nickname: "新昵称",
          avatar_url: "https://assets.example.test/users/u1/avatar/custom.png",
        },
      },
    });
  });
  storage.set("stellartrail_user", {
    id: "u1",
    nickname: "旧昵称",
    avatar_url: "https://assets.example.test/users/u1/avatar/custom.png",
  });
  const { updateWechatNickname } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(updateWechatNickname(" 新昵称 "));

  assert.deepEqual(calls[0].data, {
    code: "wx-login-code",
    profile: {
      nickname: "新昵称",
      avatar_url: "https://assets.example.test/users/u1/avatar/custom.png",
    },
  });
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "新昵称",
    avatar_url: "https://assets.example.test/users/u1/avatar/custom.png",
  });
});

test("updateWechatNickname lets a default avatar become a text avatar", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({ url: options.url, data: options.data });
    options.success({
      statusCode: 200,
      data: {
        ...loginResponse("access-nickname-text", "refresh-nickname-text"),
        user: {
          id: "u1",
          nickname: "新昵称",
          avatar_url: null,
        },
      },
    });
  });
  storage.set("stellartrail_user", {
    id: "u1",
    nickname: "旧昵称",
    avatar_url: "https://thirdwx.qlogo.cn/mmopen/default-avatar/132",
  });
  const { updateWechatNickname } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(updateWechatNickname(" 新昵称 "));

  assert.deepEqual(calls[0].data, {
    code: "wx-login-code",
    profile: { nickname: "新昵称" },
  });
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "新昵称",
    avatar_url: null,
  });
});

test("loginWithWechat reports request timeout instead of hanging", async () => {
  installWxMock((options) => {
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/auth/wechat-login",
    );
    assert.equal(options.timeout, 15000);
    options.fail({ errMsg: "request:fail timeout" });
  });
  const {
    getErrorMessage,
    loginWithWechat,
  } = require("../.tmp-test/utils/api.js");

  await assert.rejects(
    () => loginWithWechat(),
    (error) => {
      assert.equal(getErrorMessage(error), "网络请求超时，请稍后再试");
      return true;
    },
  );
  require("../.tmp-test/utils/network-state.js").initNetworkState();
});

test("loginWithWechat can retry after a transient request failure", async () => {
  let callCount = 0;
  installWxMock((options) => {
    callCount += 1;
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/auth/wechat-login",
    );
    if (callCount === 1) {
      options.fail({ errMsg: "request:fail timeout" });
      return;
    }
    options.success({
      statusCode: 200,
      data: loginResponse("access-retry", "refresh-retry"),
    });
  });
  const {
    getErrorMessage,
    loginWithWechat,
  } = require("../.tmp-test/utils/api.js");
  const {
    isOffline,
    initNetworkState,
  } = require("../.tmp-test/utils/network-state.js");
  initNetworkState();

  await assert.rejects(
    () => loginWithWechat(),
    (error) => {
      assert.equal(getErrorMessage(error), "网络请求超时，请稍后再试");
      return true;
    },
  );
  assert.equal(isOffline(), false);
  await assert.doesNotReject(loginWithWechat());
  assert.equal(callCount, 2);
});

test("stored offline state is not treated as current before system confirmation", async () => {
  const storage = installWxMock((options) => {
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/auth/wechat-login",
    );
    options.success({
      statusCode: 200,
      data: loginResponse("access-stale", "refresh-stale"),
    });
  });
  storage.set("stellartrail_network_state", {
    isOffline: true,
    networkType: "none",
    updatedAt: "2026-05-19T00:00:00.000Z",
  });
  clearCompiledUtilityModules();
  const { isOffline } = require("../.tmp-test/utils/network-state.js");
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  assert.equal(isOffline(), false);
  await assert.doesNotReject(loginWithWechat());
  clearCompiledUtilityModules();
});

test("profile refresh marker is consumed once", () => {
  const storage = installWxMock(() => {
    throw new Error("unexpected wx.request call");
  });
  const {
    consumeProfileShouldRefresh,
    markProfileShouldRefresh,
  } = require("../.tmp-test/utils/profile-refresh.js");

  assert.equal(consumeProfileShouldRefresh(), false);
  markProfileShouldRefresh();
  assert.equal(storage.get("stellartrail_profile_should_refresh"), true);
  assert.equal(consumeProfileShouldRefresh(), true);
  assert.equal(consumeProfileShouldRefresh(), false);
});

test("stale local API base URL is ignored for login requests", async () => {
  const storage = installWxMock((options) => {
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/auth/wechat-login",
    );
    options.success({
      statusCode: 200,
      data: loginResponse("access-clean-url", "refresh-clean-url"),
    });
  });
  storage.set(
    "stellartrail_api_base_url",
    "https://fixture.stellartrail.local",
  );
  clearCompiledUtilityModules();
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(loginWithWechat());
  assert.equal(storage.get("stellartrail_api_base_url"), undefined);
  clearCompiledUtilityModules();
});

test("production domain probe keeps the first healthy domain family", async () => {
  const calls = [];
  const storage = installWxMock(
    (options) => {
      calls.push(options.url);
      if (options.url === "https://api.example.invalid/healthz") {
        options.success({ statusCode: 200, data: { status: "ok" } });
        return;
      }
      assert.equal(options.url, "https://api.example.invalid/api/v1/skills");
      options.success({ statusCode: 200, data: { items: [] } });
    },
    null,
    {
      globalData: {
        domainCandidates: productionDomainCandidates(),
      },
    },
  );
  const {
    getApiBaseUrl,
    getAssetsBaseUrl,
    listSkills,
  } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(listSkills());

  assert.deepEqual(calls, [
    "https://api.example.invalid/healthz",
    "https://api.example.invalid/api/v1/skills",
  ]);
  assert.equal(getApiBaseUrl(), "https://api.example.invalid");
  assert.equal(getAssetsBaseUrl(), "https://assets.example.invalid");
  assert.equal(
    storage.get("stellartrail_api_base_url"),
    "https://api.example.invalid",
  );
  assert.equal(
    storage.get("stellartrail_assets_base_url"),
    "https://assets.example.invalid",
  );
});

test("production domain probe uses the next healthy family and rewrites known asset urls", async () => {
  const calls = [];
  installWxMock(
    (options) => {
      calls.push(options.url);
      if (options.url === "https://api.example.invalid/healthz") {
        options.success({ statusCode: 503, data: {} });
        return;
      }
      if (options.url === "https://api-alt1.example.invalid/healthz") {
        options.success({ statusCode: 200, data: { status: "ok" } });
        return;
      }
      assert.equal(
        options.url,
        "https://api-alt1.example.invalid/api/v1/skills",
      );
      options.success({ statusCode: 200, data: { items: [] } });
    },
    null,
    {
      globalData: {
        domainCandidates: productionDomainCandidates(),
      },
    },
  );
  const {
    getApiBaseUrl,
    getAssetsBaseUrl,
    listSkills,
    resolveAssetUrl,
  } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(listSkills());

  assert.deepEqual(calls, [
    "https://api.example.invalid/healthz",
    "https://api-alt1.example.invalid/healthz",
    "https://api-alt1.example.invalid/api/v1/skills",
  ]);
  assert.equal(getApiBaseUrl(), "https://api-alt1.example.invalid");
  assert.equal(getAssetsBaseUrl(), "https://assets-alt1.example.invalid");
  assert.equal(
    resolveAssetUrl("https://assets.example.invalid/knots/bowline.png"),
    "https://assets-alt1.example.invalid/knots/bowline.png",
  );
  assert.equal(
    resolveAssetUrl("https://cdn.example.test/knots/bowline.png"),
    "https://cdn.example.test/knots/bowline.png",
  );
});

test("production domain probe falls back to the first family when all health checks fail", async () => {
  const calls = [];
  installWxMock(
    (options) => {
      calls.push(options.url);
      if (options.url.endsWith("/healthz")) {
        options.fail({ errMsg: "request:fail timeout" });
        return;
      }
      assert.equal(
        options.url,
        "https://api.example.invalid/api/v1/auth/wechat-login",
      );
      options.success({
        statusCode: 200,
        data: loginResponse("access-fallback", "refresh-fallback"),
      });
    },
    null,
    {
      globalData: {
        domainCandidates: productionDomainCandidates(),
      },
    },
  );
  const { loginWithWechat } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(loginWithWechat());

  assert.deepEqual(calls, [
    "https://api.example.invalid/healthz",
    "https://api-alt1.example.invalid/healthz",
    "https://api-alt2.example.invalid/healthz",
    "https://api.example.invalid/api/v1/auth/wechat-login",
  ]);
});

test("production domain probe is shared by concurrent GET requests", async () => {
  const calls = [];
  const pendingHealth = [];
  installWxMock(
    (options) => {
      calls.push(options.url);
      if (options.url === "https://api.example.invalid/healthz") {
        pendingHealth.push(options);
        return;
      }
      if (options.url === "https://api.example.invalid/api/v1/skills") {
        options.success({ statusCode: 200, data: { items: [] } });
        return;
      }
      assert.equal(
        options.url,
        "https://api.example.invalid/api/v1/skills/knots/filters",
      );
      options.success({
        statusCode: 200,
        data: { categories: [], types: [], total_count: 0 },
      });
    },
    null,
    {
      globalData: {
        domainCandidates: productionDomainCandidates(),
      },
    },
  );
  const { getKnotFilters, listSkills } = require("../.tmp-test/utils/api.js");

  const requests = Promise.all([listSkills(), getKnotFilters()]);
  await Promise.resolve();
  assert.equal(pendingHealth.length, 1);
  pendingHealth[0].success({ statusCode: 200, data: { status: "ok" } });
  await assert.doesNotReject(requests);

  assert.equal(
    calls.filter((url) => url === "https://api.example.invalid/healthz").length,
    1,
  );
});

test("domain allowlist request failures use a safe connection message", async () => {
  installWxMock((options) => {
    options.fail({
      errMsg:
        "request:fail url not in domain list https://fixture.stellartrail.local",
    });
  });
  clearCompiledUtilityModules();
  const {
    getErrorMessage,
    loginWithWechat,
  } = require("../.tmp-test/utils/api.js");

  await assert.rejects(
    () => loginWithWechat(),
    (error) => {
      assert.equal(getErrorMessage(error), "服务连接配置异常，请稍后再试");
      return true;
    },
  );
  clearCompiledUtilityModules();
});

test("uploadWechatAvatar uploads with bearer token and stores returned user", async () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    (options) => {
      assert.equal(
        options.url,
        "https://api.example.test/api/v1/me/profile/avatar",
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

test("uploadFeedbackImage uploads an authenticated feedback attachment", async () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    (options) => {
      assert.equal(options.url, "https://api.example.test/api/v1/me/uploads");
      assert.equal(options.filePath, "/tmp/feedback.png");
      assert.equal(options.name, "file");
      assert.equal(options.formData.purpose, "feedback");
      assert.equal(options.header.authorization, "Bearer access-old");
      options.success({
        statusCode: 201,
        data: JSON.stringify({
          id: "upload-1",
          purpose: "feedback",
          original_filename: "feedback.png",
          image_type: "png",
          content_type: "image/png",
          size_bytes: 1234,
          sha256: "abc123",
          download_url: "/api/v1/me/uploads/upload-1",
          created_at: "2026-05-20T00:00:00Z",
        }),
      });
    },
  );
  storage.set("stellartrail_access_token", "access-old");
  const { uploadFeedbackImage } = require("../.tmp-test/utils/api.js");

  const upload = await uploadFeedbackImage("/tmp/feedback.png");

  assert.equal(upload.id, "upload-1");
  assert.equal(upload.purpose, "feedback");
  assert.equal(upload.download_url, "/api/v1/me/uploads/upload-1");
});

test("uploadFeedbackImage signs the raw multipart body when request signing is configured", async () => {
  const fileBytes = Buffer.from([0x89, 0x50, 0x4e, 0x47]);
  const storage = installWxMock(
    (options) => {
      const { path, query } = parseRequestUrl(options.url);
      const params = new URLSearchParams(query);
      const body = Buffer.from(new Uint8Array(options.data));
      assert.equal(path, "/api/v1/me/uploads");
      assert.equal(options.method, "POST");
      assert.equal(options.header.authorization, "Bearer access-old");
      assert.match(
        options.header["content-type"],
        /^multipart\/form-data; boundary=StellarTrailBoundary/,
      );
      assert.match(body.toString("binary"), /name="purpose"\r\n\r\nfeedback/);
      assert.match(
        body.toString("binary"),
        /name="file"; filename="feedback.png"/,
      );
      assert.equal(params.get("app_id"), "wechat-client");
      assert.match(params.get("nonce"), /^[a-z0-9]+-[0-9a-f]{32}$/);
      assert.match(params.get("signature"), /^[0-9a-f]{64}$/);
      verifyRequestSignature({
        url: options.url,
        method: "POST",
        bodyHash: sha256Hex(body),
        appId: params.get("app_id"),
        appSecret: "wechat-secret",
        nonce: params.get("nonce"),
        signature: params.get("signature"),
      });
      options.success({
        statusCode: 201,
        data: {
          id: "upload-signed",
          purpose: "feedback",
          original_filename: "feedback.png",
          image_type: "png",
          content_type: "image/png",
          size_bytes: 4,
          sha256: "abc123",
          download_url: "/api/v1/me/uploads/upload-signed",
          created_at: "2026-05-20T00:00:00Z",
        },
      });
    },
    null,
    {
      globalData: {
        requestSignature: {
          app_id: "wechat-client",
          app_secret: "wechat-secret",
        },
      },
      readFile(options) {
        assert.equal(options.filePath, "/tmp/feedback.png");
        options.success({ data: fileBytes.buffer.slice(0) });
      },
    },
  );
  storage.set("stellartrail_access_token", "access-old");
  const { uploadFeedbackImage } = require("../.tmp-test/utils/api.js");

  const upload = await uploadFeedbackImage("/tmp/feedback.png");

  assert.equal(upload.id, "upload-signed");
});

test("uploadFeedbackImage exposes readable quota validation errors", async () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    (options) => {
      assert.equal(options.url, "https://api.example.test/api/v1/me/uploads");
      options.success({
        statusCode: 422,
        data: {
          code: "validation_failed",
          message: "request validation failed",
          fields: [
            {
              field: "image_quota",
              message: "反馈图片已达到 100 张上限",
            },
          ],
        },
      });
    },
  );
  storage.set("stellartrail_access_token", "access-old");
  const {
    getErrorMessage,
    uploadFeedbackImage,
  } = require("../.tmp-test/utils/api.js");

  await assert.rejects(
    () => uploadFeedbackImage("/tmp/feedback.png"),
    (error) => {
      assert.equal(getErrorMessage(error), "反馈图片已达到 100 张上限");
      return true;
    },
  );
});

test("getCurrentUser refreshes stored profile from backend", async () => {
  const storage = installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/v1/me/profile");
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

test("outdoor profile helpers use authenticated profile outdoor endpoints", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
      auth: options.header.authorization,
    });
    if (
      options.url.endsWith("/api/v1/me/profile/outdoor") &&
      options.method === "GET"
    ) {
      options.success({
        statusCode: 200,
        data: {
          profile: {
            user_id: "u1",
            outdoor_id: "星星",
            height_cm: 176,
            blood_type: "O",
          },
        },
      });
      return;
    }
    if (
      options.url.endsWith("/api/v1/me/profile/outdoor") &&
      options.method === "PATCH"
    ) {
      options.success({
        statusCode: 200,
        data: {
          profile: {
            user_id: "u1",
            ...options.data,
          },
        },
      });
      return;
    }
    throw new Error(`unexpected request ${options.method} ${options.url}`);
  });
  storage.set("stellartrail_access_token", "access-old");
  const {
    getOutdoorProfile,
    updateOutdoorProfile,
  } = require("../.tmp-test/utils/api.js");

  const read = await getOutdoorProfile();
  const saved = await updateOutdoorProfile({
    height_cm: 177,
    blood_type: null,
  });

  assert.equal(read.profile.outdoor_id, "星星");
  assert.equal(saved.profile.height_cm, 177);
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/me/profile/outdoor",
      method: "GET",
      data: undefined,
      auth: "Bearer access-old",
    },
    {
      url: "https://api.example.test/api/v1/me/profile/outdoor",
      method: "PATCH",
      data: { height_cm: 177, blood_type: null },
      auth: "Bearer access-old",
    },
  ]);
});

test("outdoor experience helpers use authenticated experience endpoints", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push({
      path,
      method: options.method || "GET",
      data: options.data,
      auth: options.header && options.header.authorization,
    });
    if (path === "/api/v1/me/outdoor-experiences" && options.method === "GET") {
      options.success({
        statusCode: 200,
        data: {
          items: [
            {
              id: "exp-1",
              user_id: "u1",
              source_trip_id: "trip-1",
              trip_type: "team",
              title: "罗浮山重装",
              start_date: "2026-05-29",
              end_date: "2026-05-31",
              day_count: 3,
              companion_count: 2,
              route_summary: "环线",
              created_at: "2026-05-31T00:00:00Z",
              updated_at: "2026-05-31T00:00:00Z",
            },
          ],
        },
      });
      return;
    }
    if (
      path === "/api/v1/me/outdoor-experiences" &&
      options.method === "POST"
    ) {
      options.success({
        statusCode: 201,
        data: {
          id: "exp-2",
          user_id: "u1",
          trip_type: "solo",
          title: options.data.title,
          created_at: "2026-06-01T00:00:00Z",
          updated_at: "2026-06-01T00:00:00Z",
        },
      });
      return;
    }
    if (
      path === "/api/v1/me/outdoor-experiences/exp-1" &&
      options.method === "PATCH"
    ) {
      options.success({
        statusCode: 200,
        data: {
          id: "exp-1",
          user_id: "u1",
          trip_type: "team",
          title: options.data.title,
          created_at: "2026-05-31T00:00:00Z",
          updated_at: "2026-06-01T00:00:00Z",
        },
      });
      return;
    }
    if (
      path === "/api/v1/me/outdoor-experiences/exp-1" &&
      options.method === "DELETE"
    ) {
      options.success({ statusCode: 204, data: undefined });
      return;
    }
    if (
      path === "/api/v1/me/trips/trip-1/convert-to-outdoor-experience" &&
      options.method === "POST"
    ) {
      options.success({
        statusCode: 201,
        data: {
          id: "exp-3",
          user_id: "u1",
          source_trip_id: "trip-1",
          trip_type: "team",
          title: "历史行程",
          created_at: "2026-06-02T00:00:00Z",
          updated_at: "2026-06-02T00:00:00Z",
        },
      });
      return;
    }
    throw new Error(`unexpected request ${options.method} ${options.url}`);
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u1", nickname: "星" });
  const {
    listOutdoorExperiences,
    createOutdoorExperience,
    updateOutdoorExperience,
    deleteOutdoorExperience,
    convertTripToOutdoorExperience,
  } = require("../.tmp-test/utils/api.js");

  const list = await listOutdoorExperiences();
  const created = await createOutdoorExperience({ title: "手动经历" });
  const updated = await updateOutdoorExperience("exp-1", { title: "更新经历" });
  await deleteOutdoorExperience("exp-1");
  const converted = await convertTripToOutdoorExperience("trip-1");

  assert.equal(list.items[0].title, "罗浮山重装");
  assert.equal(created.title, "手动经历");
  assert.equal(updated.title, "更新经历");
  assert.equal(converted.source_trip_id, "trip-1");
  assert.deepEqual(calls, [
    {
      path: "/api/v1/me/outdoor-experiences",
      method: "GET",
      data: undefined,
      auth: "Bearer access-old",
    },
    {
      path: "/api/v1/me/outdoor-experiences",
      method: "POST",
      data: { title: "手动经历" },
      auth: "Bearer access-old",
    },
    {
      path: "/api/v1/me/outdoor-experiences/exp-1",
      method: "PATCH",
      data: { title: "更新经历" },
      auth: "Bearer access-old",
    },
    {
      path: "/api/v1/me/outdoor-experiences/exp-1",
      method: "DELETE",
      data: undefined,
      auth: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/trip-1/convert-to-outdoor-experience",
      method: "POST",
      data: undefined,
      auth: "Bearer access-old",
    },
  ]);
});

test("email binding sends authenticated code and stores bound email", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    if (options.url.endsWith("/api/v1/me/email-binding-code")) {
      options.success({
        statusCode: 200,
        data: {
          email: "bound@example.com",
          expires_at: "2026-06-01T02:10:00Z",
        },
      });
      return;
    }
    if (options.url.endsWith("/api/v1/me/email-binding")) {
      options.success({
        statusCode: 200,
        data: {
          user: {
            id: "u1",
            nickname: "微信用户",
            email: "bound@example.com",
            avatar_url: null,
          },
        },
      });
      return;
    }
    options.success({ statusCode: 404, data: { message: "not found" } });
  });
  storage.set("stellartrail_access_token", "access-old");
  const {
    bindEmailToCurrentAccount,
    sendBindEmailCode,
  } = require("../.tmp-test/utils/api.js");

  await sendBindEmailCode("bound@example.com");
  const user = await bindEmailToCurrentAccount({
    email: "bound@example.com",
    email_verification_code: "123456",
  });

  assert.equal(user.email, "bound@example.com");
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "微信用户",
    email: "bound@example.com",
    avatar_url: null,
  });
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/me/email-binding-code",
      method: "POST",
      data: { email: "bound@example.com" },
      authorization: "Bearer access-old",
    },
    {
      url: "https://api.example.test/api/v1/me/email-binding",
      method: "POST",
      data: {
        email: "bound@example.com",
        email_verification_code: "123456",
      },
      authorization: "Bearer access-old",
    },
  ]);
});

test("createFeedback posts authenticated user feedback", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 201,
      data: {
        id: "feedback-1",
        category: "suggestion",
        content: "希望增加设置区",
        contact: "trail@example.com",
        page: "/pages/profile/index",
        client_platform: "wechat_miniprogram",
        client_version: "dev",
        device_model: "iPhone",
        status: "open",
        images: [],
        created_at: "2026-05-20T00:00:00Z",
        updated_at: "2026-05-20T00:00:00Z",
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  const { createFeedback } = require("../.tmp-test/utils/api.js");

  const feedback = await createFeedback({
    category: "suggestion",
    content: "希望增加设置区",
    contact: "trail@example.com",
    page: "/pages/profile/index",
    client_platform: "wechat_miniprogram",
    client_version: "dev",
    device_model: "iPhone",
    image_ids: ["upload-1"],
  });

  assert.equal(feedback.id, "feedback-1");
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/me/feedback",
      method: "POST",
      data: {
        category: "suggestion",
        content: "希望增加设置区",
        contact: "trail@example.com",
        page: "/pages/profile/index",
        client_platform: "wechat_miniprogram",
        client_version: "dev",
        device_model: "iPhone",
        image_ids: ["upload-1"],
      },
      authorization: "Bearer access-old",
    },
  ]);
});

test("listClientVersions fetches public WeChat version history", async () => {
  const calls = [];
  installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: {
        next_cursor: null,
        items: [
          {
            id: "version-1",
            client_key: "wechat_miniprogram",
            version: "0.1.0",
            title: "0.1.0 初始版本",
            release_notes: ["装备库上线"],
            release_note_sections: [
              {
                key: "feature",
                title: "Feature",
                items: ["装备库上线"],
              },
            ],
            status: "published",
            published_at: "2026-05-23T00:00:00Z",
            created_at: "2026-05-23T00:00:00Z",
            updated_at: "2026-05-23T00:00:00Z",
          },
        ],
      },
    });
  });
  const { listClientVersions } = require("../.tmp-test/utils/api.js");

  const response = await listClientVersions("wechat_miniprogram", {
    limit: 20,
  });

  assert.equal(response.items[0].version, "0.1.0");
  assert.equal(response.items[0].release_note_sections[0].title, "Feature");
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/client-versions?limit=20&client_key=wechat_miniprogram",
      method: "GET",
      authorization: undefined,
    },
  ]);
});

test("authenticated requests refresh once on 401 and retry with the new access token", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      authorization: options.header && options.header.authorization,
      data: options.data,
    });
    if (options.url.endsWith("/api/v1/me/gears/stats") && calls.length === 1) {
      options.success({ statusCode: 401, data: { code: "unauthorized" } });
      return;
    }
    if (options.url.endsWith("/api/v1/auth/refresh")) {
      assert.deepEqual(options.data, { refresh_token: "refresh-old" });
      options.success({
        statusCode: 200,
        data: {
          ...loginResponse("access-new", "refresh-new"),
          expires_at: "2026-07-01T00:00:00Z",
        },
      });
      return;
    }
    if (options.url.endsWith("/api/v1/me/gears/stats")) {
      options.success({
        statusCode: 200,
        data: {
          current_count: 0,
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
  storage.set("stellartrail_access_token_expires_at", "2026-07-01T00:00:00Z");
  storage.set("stellartrail_refresh_token", "refresh-old");
  const { getGearStats } = require("../.tmp-test/utils/api.js");

  await assert.doesNotReject(getGearStats());
  assert.deepEqual(
    calls.map((call) => call.url.replace("https://api.example.test", "")),
    [
      "/api/v1/me/gears/stats",
      "/api/v1/auth/refresh",
      "/api/v1/me/gears/stats",
    ],
  );
  assert.equal(calls[0].authorization, "Bearer access-old");
  assert.equal(calls[2].authorization, "Bearer access-new");
  assert.equal(storage.get("stellartrail_access_token"), "access-new");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-new");
});

test("expired access token is kept when refresh fails because of network", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push(options.url.replace("https://api.example.test", ""));
    assert.equal(options.url, "https://api.example.test/api/v1/auth/refresh");
    options.fail({ errMsg: "request:fail timeout" });
  });
  storage.set("stellartrail_access_token", "access-offline");
  storage.set("stellartrail_access_token_expires_at", "2020-01-01T00:00:00Z");
  storage.set("stellartrail_refresh_token", "refresh-offline");
  storage.set("stellartrail_user", { id: "u1", nickname: "离线用户" });
  const { ensureAccessToken } = require("../.tmp-test/utils/api.js");

  const token = await ensureAccessToken();

  assert.equal(token, "access-offline");
  assert.deepEqual(calls, ["/api/v1/auth/refresh"]);
  assert.equal(storage.get("stellartrail_access_token"), "access-offline");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-offline");
  assert.deepEqual(storage.get("stellartrail_user"), {
    id: "u1",
    nickname: "离线用户",
  });
});

test("expired access token is cleared when refresh is explicitly unauthorized", async () => {
  const storage = installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/v1/auth/refresh");
    options.success({
      statusCode: 401,
      data: { code: "unauthorized", message: "refresh token expired" },
    });
  });
  storage.set("stellartrail_access_token", "access-expired");
  storage.set("stellartrail_access_token_expires_at", "2020-01-01T00:00:00Z");
  storage.set("stellartrail_refresh_token", "refresh-expired");
  storage.set("stellartrail_user", { id: "u1", nickname: "过期用户" });
  const {
    ensureAccessToken,
    isLoginRequiredError,
  } = require("../.tmp-test/utils/api.js");

  await assert.rejects(
    () => ensureAccessToken(),
    (error) => {
      assert.equal(isLoginRequiredError(error), true);
      return true;
    },
  );

  assert.equal(storage.get("stellartrail_access_token"), undefined);
  assert.equal(storage.get("stellartrail_refresh_token"), undefined);
  assert.equal(storage.get("stellartrail_user"), undefined);
});

test("authenticated GET falls back to user offline cache when refresh retry has network failure", async () => {
  let statsCallCount = 0;
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    if (path === "/api/v1/me/gears/stats") {
      statsCallCount += 1;
      if (statsCallCount === 1) {
        options.success({
          statusCode: 200,
          data: {
            current_count: 3,
            archived_count: 0,
            total_value_cents: 0,
            total_weight_g: 900,
            by_category: [],
            by_status: [],
          },
        });
        return;
      }
      options.success({
        statusCode: 401,
        data: { code: "unauthorized", message: "access token expired" },
      });
      return;
    }
    if (path === "/api/v1/auth/refresh") {
      options.fail({ errMsg: "request:fail timeout" });
      return;
    }
    throw new Error(`unexpected request ${path}`);
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_refresh_token", "refresh-old");
  storage.set("stellartrail_user", { id: "u-cache", nickname: "缓存用户" });
  const {
    consumeOfflineCacheNotice,
    getGearStats,
  } = require("../.tmp-test/utils/api.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  const online = await getGearStats();
  const offline = await getGearStats();

  assert.deepEqual(offline, online);
  assert.equal(storage.get("stellartrail_access_token"), "access-old");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-old");
  assert.equal(consumeOfflineCacheNotice(), "当前离线，正在显示已缓存内容");
});

test("knot disclaimer reads and writes authenticated acceptance", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      authorization: options.header && options.header.authorization,
      data: options.data,
    });
    if (options.url.endsWith("/api/v1/me/skills/knots/disclaimer")) {
      options.success({
        statusCode: 200,
        data: {
          key: "knot_tutorial_disclaimer",
          version: "v1",
          title: "绳结教程免责声明",
          content: "仅供参考",
          accepted: false,
          accepted_at: null,
        },
      });
      return;
    }
    if (options.url.endsWith("/api/v1/me/skills/knots/disclaimer/acceptance")) {
      options.success({
        statusCode: 200,
        data: {
          key: "knot_tutorial_disclaimer",
          version: "v1",
          title: "绳结教程免责声明",
          content: "仅供参考",
          accepted: true,
          accepted_at: "2026-05-24T00:00:00Z",
        },
      });
      return;
    }
    options.success({ statusCode: 404, data: { message: "not found" } });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u1" });
  const {
    acceptKnotDisclaimer,
    getKnotDisclaimer,
    hasLocalKnotDisclaimerAcceptance,
  } = require("../.tmp-test/utils/api.js");

  const disclaimer = await getKnotDisclaimer();
  const accepted = await acceptKnotDisclaimer({
    client_platform: "wechat_miniprogram",
    client_version: "dev",
    device_model: "iPhone",
  });

  assert.equal(disclaimer.accepted, false);
  assert.equal(accepted.accepted, true);
  assert.equal(hasLocalKnotDisclaimerAcceptance(), true);
  assert.deepEqual(storage.get("stellartrail_knot_disclaimer_acceptance_v1"), {
    userId: "u1",
    version: "v1",
    acceptedAt: "2026-05-24T00:00:00Z",
  });
  assert.deepEqual(
    calls.map((call) => ({
      path: call.url.replace("https://api.example.test", ""),
      method: call.method,
      authorization: call.authorization,
      data: call.data,
    })),
    [
      {
        path: "/api/v1/me/skills/knots/disclaimer",
        method: "GET",
        authorization: "Bearer access-old",
        data: undefined,
      },
      {
        path: "/api/v1/me/skills/knots/disclaimer/acceptance",
        method: "POST",
        authorization: "Bearer access-old",
        data: {
          client_platform: "wechat_miniprogram",
          client_version: "dev",
          device_model: "iPhone",
        },
      },
    ],
  );
});

test("accepted knot disclaimer read is saved for offline fallback", async () => {
  const storage = installWxMock((options) => {
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/me/skills/knots/disclaimer",
    );
    options.success({
      statusCode: 200,
      data: {
        key: "knot_tutorial_disclaimer",
        version: "v1",
        title: "绳结教程免责声明",
        content: "仅供参考",
        accepted: true,
        accepted_at: "2026-05-31T00:00:00Z",
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u1" });
  const {
    getKnotDisclaimer,
    hasLocalKnotDisclaimerAcceptance,
  } = require("../.tmp-test/utils/api.js");

  await getKnotDisclaimer();

  assert.equal(hasLocalKnotDisclaimerAcceptance(), true);
  assert.deepEqual(storage.get("stellartrail_knot_disclaimer_acceptance_v1"), {
    userId: "u1",
    version: "v1",
    acceptedAt: "2026-05-31T00:00:00Z",
  });
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
      url: "https://api.example.test/api/v1/me/gears/spec-key-rankings?category=electronics_system",
      authorization: "Bearer access-old",
    },
  ]);
});

test("getGearOverview calls the authenticated first-screen aggregate endpoint", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: {
        categories: { items: [{ id: "all", label: "全部装备", count: 1 }] },
        stats: {
          current_count: 1,
          total_value_cents: 63900,
          total_weight_g: 315,
          by_category: [
            {
              category: "electronics_system",
              label: "电子系统",
              count: 1,
              total_weight_g: 315,
              total_value_cents: 63900,
            },
          ],
          by_status: [
            {
              status: "available",
              label: "可用",
              count: 1,
              total_weight_g: 315,
              total_value_cents: 63900,
            },
          ],
        },
        list: {
          items: [{ id: "gear-1", name: "头灯", tags: [], tag_colors: {} }],
          next_cursor: null,
        },
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-overview", nickname: "星" });
  const { getGearOverview } = require("../.tmp-test/utils/api.js");

  const response = await getGearOverview({
    limit: 2,
    sort: "created_at_desc",
  });

  assert.equal(response.stats.current_count, 1);
  assert.equal(response.stats.by_category[0].total_weight_g, 315);
  assert.equal(response.stats.by_category[0].total_value_cents, 63900);
  assert.equal(response.stats.by_status[0].total_weight_g, 315);
  assert.equal(response.stats.by_status[0].total_value_cents, 63900);
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/me/gears/overview?limit=2&sort=created_at_desc",
      method: "GET",
      authorization: "Bearer access-old",
    },
  ]);
});

test("gear packing list API utilities call authenticated endpoints", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push({
      path,
      method: options.method || "GET",
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    if (path === "/api/v1/me/packing-lists?limit=10") {
      options.success({
        statusCode: 200,
        data: { items: [], next_cursor: null },
      });
      return;
    }
    if (path === "/api/v1/me/packing-lists" && options.method === "POST") {
      options.success({
        statusCode: 201,
        data: {
          id: "pack-1",
          name: "武功山一日",
          stats: { item_count: 0, packed_count: 0, total_weight_g: 0 },
          items: [],
          created_at: "2026-05-24T00:00:00Z",
          updated_at: "2026-05-24T00:00:00Z",
        },
      });
      return;
    }
    if (path === "/api/v1/me/packing-lists/pack-1") {
      options.success({
        statusCode: 200,
        data: {
          id: "pack-1",
          name: "武功山一日",
          stats: { item_count: 1, packed_count: 0, total_weight_g: 800 },
          items: [],
          created_at: "2026-05-24T00:00:00Z",
          updated_at: "2026-05-24T00:00:00Z",
        },
      });
      return;
    }
    if (path === "/api/v1/me/packing-lists/pack-1/items") {
      options.success({
        statusCode: 200,
        data: {
          id: "pack-1",
          name: "武功山一日",
          stats: { item_count: 1, packed_count: 0, total_weight_g: 800 },
          items: [],
          created_at: "2026-05-24T00:00:00Z",
          updated_at: "2026-05-24T00:00:00Z",
        },
      });
      return;
    }
    if (path === "/api/v1/me/packing-lists/pack-1/items/item-1") {
      options.success({
        statusCode: 200,
        data: {
          id: "pack-1",
          name: "武功山一日",
          stats: { item_count: 1, packed_count: 1, total_weight_g: 800 },
          items: [],
          created_at: "2026-05-24T00:00:00Z",
          updated_at: "2026-05-24T00:00:00Z",
        },
      });
      return;
    }
    options.success({ statusCode: 204, data: undefined });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-packing", nickname: "星" });
  const {
    addGearPackingItems,
    createGearPackingList,
    deleteGearPackingList,
    getGearPackingList,
    listGearPackingLists,
    removeGearPackingItem,
    updateGearPackingItem,
    updateGearPackingList,
  } = require("../.tmp-test/utils/api.js");

  await listGearPackingLists({ limit: 10 });
  await createGearPackingList({
    name: "武功山一日",
  });
  await updateGearPackingList("pack-1", {
    name: "武功山一日",
    route_name: "武功山",
    duration_label: "一日",
  });
  await getGearPackingList("pack-1");
  await addGearPackingItems("pack-1", ["gear-1"]);
  await updateGearPackingItem("pack-1", "item-1", true);
  await removeGearPackingItem("pack-1", "item-1");
  await deleteGearPackingList("pack-1");

  assert.deepEqual(
    calls.map((call) => ({
      path: call.path,
      method: call.method,
      data: call.data,
      authorization: call.authorization,
    })),
    [
      {
        path: "/api/v1/me/packing-lists?limit=10",
        method: "GET",
        data: undefined,
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists",
        method: "POST",
        data: {
          name: "武功山一日",
        },
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists/pack-1",
        method: "PATCH",
        data: {
          name: "武功山一日",
          route_name: "武功山",
          duration_label: "一日",
        },
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists/pack-1",
        method: "GET",
        data: undefined,
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists/pack-1/items",
        method: "POST",
        data: { gear_ids: ["gear-1"] },
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists/pack-1/items/item-1",
        method: "PATCH",
        data: { packed: true },
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists/pack-1/items/item-1",
        method: "DELETE",
        data: undefined,
        authorization: "Bearer access-old",
      },
      {
        path: "/api/v1/me/packing-lists/pack-1",
        method: "DELETE",
        data: undefined,
        authorization: "Bearer access-old",
      },
    ],
  );
});

test("trip gear API utilities import packing lists and create trip-only gear", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push({
      path,
      method: options.method || "GET",
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: {
        plan: {
          id: "team-1",
          owner_user_id: "u-team",
          name: "端午重装计划",
          enabled_sections: ["members", "personal_gear"],
          itinerary_day_count: 0,
          field_versions: {},
          is_deleted: false,
          created_at: "2026-05-24T00:00:00Z",
          updated_at: "2026-05-24T00:00:00Z",
        },
        members: [],
        my_member_id: "member-1",
        personal_gear_items: [],
        shared_gear_items: [],
        itinerary_days: [],
        route_segments: [],
        food_meals: [],
        medical_items: [],
        gear_weight_summaries: [],
        member_gear_views: [],
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-team", nickname: "队长" });
  const {
    bindTripSharedGearDemandMyGear,
    createTripPersonalGearItem,
    createTripSharedGearDemand,
    deleteTripSharedGearDemand,
    fillTripSharedGearDemandConcreteGear,
    importTripPackingList,
    updateTripSharedGearDemand,
  } = require("../.tmp-test/utils/api.js");

  await importTripPackingList("team-1", { packing_list_id: "pack-1" });
  await createTripPersonalGearItem("team-1", {
    name: "防水袋",
    category: "other_gear",
    planned_quantity: 1,
    packed_quantity: 0,
    unit_weight_g: 50,
    notes: null,
  });
  await createTripSharedGearDemand("team-1", {
    name: "炉头",
    template_key: "common_stove_burner",
    demand_name: "炉头",
    concrete_name: "火枫炉头",
    source_gear_id: "gear-1",
    category: "kitchen_system",
    planned_quantity: 1,
    packed_quantity: 0,
    source_member_id: "member-1",
    responsible_member_id: "member-1",
  });
  await updateTripSharedGearDemand("team-1", "shared-1", {
    responsible_member_id: "member-2",
    base_field_versions: { responsible_member_id: 1 },
  });
  await bindTripSharedGearDemandMyGear("team-1", "shared-1", {
    source_gear_id: "gear-2",
    source_member_id: "member-1",
  });
  await fillTripSharedGearDemandConcreteGear("team-1", "shared-1", {
    concrete_name: "火枫 300T",
    unit_weight_g: 45,
  });
  await deleteTripSharedGearDemand("team-1", "shared-2");

  assert.deepEqual(calls, [
    {
      path: "/api/v1/me/trips/team-1/personal-gear/import-packing-list",
      method: "POST",
      data: { packing_list_id: "pack-1" },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/personal-gear",
      method: "POST",
      data: {
        name: "防水袋",
        category: "other_gear",
        planned_quantity: 1,
        packed_quantity: 0,
        unit_weight_g: 50,
        notes: null,
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/shared-gear-demands",
      method: "POST",
      data: {
        name: "炉头",
        template_key: "common_stove_burner",
        demand_name: "炉头",
        concrete_name: "火枫炉头",
        source_gear_id: "gear-1",
        category: "kitchen_system",
        planned_quantity: 1,
        packed_quantity: 0,
        source_member_id: "member-1",
        responsible_member_id: "member-1",
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/shared-gear-demands/shared-1",
      method: "PATCH",
      data: {
        responsible_member_id: "member-2",
        base_field_versions: { responsible_member_id: 1 },
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/shared-gear-demands/shared-1/bind-my-gear",
      method: "POST",
      data: {
        source_gear_id: "gear-2",
        source_member_id: "member-1",
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/shared-gear-demands/shared-1/fill-concrete-gear",
      method: "POST",
      data: {
        concrete_name: "火枫 300T",
        unit_weight_g: 45,
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/shared-gear-demands/shared-2",
      method: "DELETE",
      data: undefined,
      authorization: "Bearer access-old",
    },
  ]);
});

test("trip food API utilities edit meal ingredients and shared supplies", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push({
      path,
      method: options.method || "GET",
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: {
        plan: {
          id: "team-1",
          owner_user_id: "u-team",
          name: "端午重装计划",
          enabled_sections: ["members", "food_plan"],
          itinerary_day_count: 1,
          field_versions: {},
          is_deleted: false,
          created_at: "2026-05-24T00:00:00Z",
          updated_at: "2026-05-24T00:00:00Z",
        },
        members: [],
        my_member_id: "member-1",
        personal_gear_items: [],
        shared_gear_items: [],
        itinerary_days: [],
        route_segments: [],
        food_meals: [],
        food_supplies: [],
        medical_items: [],
        gear_weight_summaries: [],
        member_gear_views: [],
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-team", nickname: "队长" });
  const {
    createTripFoodItem,
    createTripFoodSupply,
    deleteTripFoodItem,
    deleteTripFoodSupply,
    updateTripFoodItem,
    updateTripFoodSupply,
  } = require("../.tmp-test/utils/api.js");

  await createTripFoodItem("team-1", "meal-1", {
    name: "米饭",
    amount_g: 500,
    total_price_cents: 1280,
    responsible_member_id: "member-1",
    notes: "早餐主食",
  });
  await updateTripFoodItem("team-1", "meal-1", "item-1", {
    amount_g: 450,
    total_price_cents: 1180,
    responsible_member_id: "member-2",
  });
  await deleteTripFoodItem("team-1", "meal-1", "item-1");
  await createTripFoodSupply("team-1", {
    name: "盐",
    supply_type: "调味",
    amount_g: 100,
    total_price_cents: 500,
    responsible_member_id: "member-1",
  });
  await updateTripFoodSupply("team-1", "supply-1", {
    amount_g: 80,
    total_price_cents: 450,
    notes: "公共调味",
  });
  await deleteTripFoodSupply("team-1", "supply-1");

  assert.deepEqual(calls, [
    {
      path: "/api/v1/me/trips/team-1/food-meals/meal-1/items",
      method: "POST",
      data: {
        name: "米饭",
        amount_g: 500,
        total_price_cents: 1280,
        responsible_member_id: "member-1",
        notes: "早餐主食",
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/food-meals/meal-1/items/item-1",
      method: "PATCH",
      data: {
        amount_g: 450,
        total_price_cents: 1180,
        responsible_member_id: "member-2",
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/food-meals/meal-1/items/item-1",
      method: "DELETE",
      data: undefined,
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/food-supplies",
      method: "POST",
      data: {
        name: "盐",
        supply_type: "调味",
        amount_g: 100,
        total_price_cents: 500,
        responsible_member_id: "member-1",
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/food-supplies/supply-1",
      method: "PATCH",
      data: {
        amount_g: 80,
        total_price_cents: 450,
        notes: "公共调味",
      },
      authorization: "Bearer access-old",
    },
    {
      path: "/api/v1/me/trips/team-1/food-supplies/supply-1",
      method: "DELETE",
      data: undefined,
      authorization: "Bearer access-old",
    },
  ]);
});

test("trip home highlight API calls authenticated dated endpoint", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push({
      path,
      method: options.method || "GET",
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    options.success({
      statusCode: 200,
      data: {
        item: {
          plan: {
            id: "team-1",
            owner_user_id: "u-team",
            name: "银湖山",
            start_date: "2026-05-26",
            end_date: "2026-05-29",
            enabled_sections: ["members", "personal_gear"],
            itinerary_day_count: 0,
            field_versions: {},
            is_deleted: false,
            created_at: "2026-05-24T00:00:00Z",
            updated_at: "2026-05-24T00:00:00Z",
          },
          status: "ongoing",
          days_until_start: 0,
          days_until_end: 2,
        },
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-team", nickname: "队长" });
  const { getTripHomeHighlight } = require("../.tmp-test/utils/api.js");

  const response = await getTripHomeHighlight("2026-05-27");

  assert.equal(response.item.plan.name, "银湖山");
  assert.deepEqual(calls, [
    {
      path: "/api/v1/me/trips/home-highlight?today=2026-05-27",
      method: "GET",
      data: undefined,
      authorization: "Bearer access-old",
    },
  ]);
});

test("roadmap API utilities call public and authenticated endpoints", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push({
      path,
      method: options.method || "GET",
      authorization: options.header && options.header.authorization,
      clientIdentity: options.header && options.header["X-StellarTrail-Client"],
    });
    options.success({
      statusCode: 200,
      data: {
        items: [
          {
            id: "smart-packing-template",
            client_key: "wechat_miniprogram",
            title: "智能打包清单模板",
            summary: "按路线和历史习惯生成建议清单",
            category: "gear",
            status: "planned",
            priority: 100,
            sort_order: 10,
            is_published: true,
            vote_count: 1,
            subscription_count: 0,
            is_voted: path.includes("/me/roadmap"),
            is_subscribed: false,
            created_at: "2026-05-24T00:00:00Z",
            updated_at: "2026-05-24T00:00:00Z",
          },
        ],
        next_cursor: null,
      },
    });
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-roadmap", nickname: "星" });
  const {
    listMyRoadmap,
    listRoadmap,
    subscribeRoadmapItem,
    unsubscribeRoadmapItem,
    unvoteRoadmapItem,
    voteRoadmapItem,
  } = require("../.tmp-test/utils/api.js");

  await listRoadmap({ client_key: "wechat_miniprogram", status: "planned" });
  await listMyRoadmap({ client_key: "wechat_miniprogram" });
  await voteRoadmapItem("smart-packing-template");
  await unvoteRoadmapItem("smart-packing-template");
  await subscribeRoadmapItem("smart-packing-template");
  await unsubscribeRoadmapItem("smart-packing-template");

  assert.deepEqual(calls, [
    {
      path: "/api/v1/roadmap?client_key=wechat_miniprogram&status=planned",
      method: "GET",
      authorization: undefined,
      clientIdentity: "wechat/0.2.2",
    },
    {
      path: "/api/v1/me/roadmap?client_key=wechat_miniprogram",
      method: "GET",
      authorization: "Bearer access-old",
      clientIdentity: "wechat/0.2.2",
    },
    {
      path: "/api/v1/me/roadmap/smart-packing-template/vote",
      method: "PUT",
      authorization: "Bearer access-old",
      clientIdentity: "wechat/0.2.2",
    },
    {
      path: "/api/v1/me/roadmap/smart-packing-template/vote",
      method: "DELETE",
      authorization: "Bearer access-old",
      clientIdentity: "wechat/0.2.2",
    },
    {
      path: "/api/v1/me/roadmap/smart-packing-template/subscription",
      method: "PUT",
      authorization: "Bearer access-old",
      clientIdentity: "wechat/0.2.2",
    },
    {
      path: "/api/v1/me/roadmap/smart-packing-template/subscription",
      method: "DELETE",
      authorization: "Bearer access-old",
      clientIdentity: "wechat/0.2.2",
    },
  ]);
});

test("identical GET requests share one in-flight wx.request", async () => {
  let requestCount = 0;
  const storage = installWxMock((options) => {
    requestCount += 1;
    assert.equal(
      options.url,
      "https://api.example.test/api/v1/me/gears/overview?limit=2",
    );
    setTimeout(() => {
      options.success({
        statusCode: 200,
        data: {
          categories: { items: [] },
          stats: {
            current_count: 0,
            total_value_cents: 0,
            total_weight_g: 0,
            by_category: [],
            by_status: [],
          },
          list: { items: [], next_cursor: null },
        },
      });
    }, 5);
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-dedupe", nickname: "星" });
  const { getGearOverview } = require("../.tmp-test/utils/api.js");

  const [first, second] = await Promise.all([
    getGearOverview({ limit: 2 }),
    getGearOverview({ limit: 2 }),
  ]);

  assert.equal(requestCount, 1);
  assert.equal(first, second);
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
      url: "https://api.example.test/api/v1/me/gears/tag-suggestions?limit=12",
      authorization: "Bearer access-old",
    },
  ]);
});

test("gear form caches spec rankings and tag suggestions briefly", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    const path = options.url.replace("https://api.example.test", "");
    calls.push(path);
    if (path.includes("/me/gears/spec-key-rankings")) {
      options.success({
        statusCode: 200,
        data: { keys: ["battery_capacity"] },
      });
      return;
    }
    if (path.includes("/me/gears/tag-suggestions")) {
      options.success({
        statusCode: 200,
        data: { items: [{ tag: "冬季", color: "blue" }] },
      });
      return;
    }
    throw new Error(`unexpected request ${path}`);
  });
  storage.set("stellartrail_access_token", "access-old");
  storage.set("stellartrail_user", { id: "u-cache", nickname: "星" });
  const {
    clearGearFormSuggestionCaches,
    getCachedGearSpecKeyRankings,
    getCachedGearTagSuggestions,
  } = require("../.tmp-test/utils/gear-form-cache.js");

  await getCachedGearSpecKeyRankings("electronics_system");
  await getCachedGearSpecKeyRankings("electronics_system");
  await getCachedGearTagSuggestions(20);
  await getCachedGearTagSuggestions(20);
  clearGearFormSuggestionCaches();
  await getCachedGearTagSuggestions(20);

  assert.deepEqual(calls, [
    "/api/v1/me/gears/spec-key-rankings?category=electronics_system",
    "/api/v1/me/gears/tag-suggestions?limit=20",
    "/api/v1/me/gears/tag-suggestions?limit=20",
  ]);
});

test("getKnotFilters calls the public knot filters endpoint with locale", async () => {
  const calls = [];
  installWxMock((options) => {
    calls.push({
      url: options.url,
      locale: options.header && options.header["X-StellarTrail-Locale"],
      clientIdentity: options.header && options.header["X-StellarTrail-Client"],
    });
    options.success({
      statusCode: 200,
      data: {
        locale: "zh-CN",
        categories: [{ id: "camping", slug: "camping", title: "露营" }],
      },
    });
  });
  const { getKnotFilters } = require("../.tmp-test/utils/api.js");

  const response = await getKnotFilters("zh-CN");

  assert.deepEqual(response.categories[0], {
    id: "camping",
    slug: "camping",
    title: "露营",
  });
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/skills/knots/filters",
      locale: "zh-CN",
      clientIdentity: "wechat/0.2.2",
    },
  ]);
});

test("favorite skill helpers call authenticated favorites endpoints", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      authorization: options.header && options.header.authorization,
      locale: options.header && options.header["X-StellarTrail-Locale"],
    });
    if (options.url.includes("/api/v1/me/skills/favorites?")) {
      options.success({
        statusCode: 200,
        data: {
          locale: "zh-CN",
          filters: [
            { id: "all", title: "全部收藏", count: 1 },
            { id: "knots", title: "绳结", count: 1 },
          ],
          items: [
            {
              skill_category: "knots",
              favorited_at: "2026-05-24T00:00:00Z",
              knot: {
                id: "bowline",
                slug: "bowline",
                title: "布林结",
                summary: "固定绳圈",
                categories: [],
                types: [],
                media: [],
                href: "/api/v1/skills/knots/detail/bowline",
              },
            },
          ],
          page: { limit: 20, offset: 0, next_offset: null },
        },
      });
      return;
    }
    options.success({
      statusCode: 200,
      data: {
        skill_category: "knots",
        knot_id: "bowline",
        is_favorited: options.method !== "DELETE",
        favorited_at:
          options.method === "DELETE" ? null : "2026-05-24T00:00:00Z",
      },
    });
  });
  storage.set("stellartrail_access_token", "access-favorite");
  const {
    favoriteKnot,
    getFavoriteKnotStatus,
    listFavoriteSkills,
    unfavoriteKnot,
  } = require("../.tmp-test/utils/api.js");

  const list = await listFavoriteSkills({
    skill_category: "knots",
    offset: 0,
    limit: 20,
  });
  const status = await getFavoriteKnotStatus("bowline");
  const favorited = await favoriteKnot("bowline");
  const unfavorited = await unfavoriteKnot("bowline");

  assert.equal(list.items[0].knot.id, "bowline");
  assert.equal(status.is_favorited, true);
  assert.equal(favorited.is_favorited, true);
  assert.equal(unfavorited.is_favorited, false);
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/me/skills/favorites?skill_category=knots&offset=0&limit=20",
      method: "GET",
      authorization: "Bearer access-favorite",
      locale: "zh-CN",
    },
    {
      url: "https://api.example.test/api/v1/me/skills/favorites/knots/bowline",
      method: "GET",
      authorization: "Bearer access-favorite",
      locale: undefined,
    },
    {
      url: "https://api.example.test/api/v1/me/skills/favorites/knots/bowline",
      method: "PUT",
      authorization: "Bearer access-favorite",
      locale: undefined,
    },
    {
      url: "https://api.example.test/api/v1/me/skills/favorites/knots/bowline",
      method: "DELETE",
      authorization: "Bearer access-favorite",
      locale: undefined,
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
    assert.equal(options.url, "https://api.example.test/api/v1/auth/login");
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

test("SMS login helpers call phone auth endpoints and persist tokens", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
      authorization: options.header && options.header.authorization,
    });
    if (options.url.endsWith("/api/v1/auth/sms-login-code")) {
      options.success({
        statusCode: 200,
        data: {
          phone: "13800138000",
          sms_ticket: "sms-ticket-1",
          expires_at: "2026-06-01T02:10:00Z",
          debug_code: "123456",
        },
      });
      return;
    }
    assert.equal(options.url, "https://api.example.test/api/v1/auth/sms-login");
    options.success({
      statusCode: 200,
      data: loginResponse("access-sms", "refresh-sms"),
    });
  });
  const { loginWithSmsCode, sendSmsLoginCode } = require("../.tmp-test/utils/api.js");

  const code = await sendSmsLoginCode("13800138000");
  await assert.doesNotReject(
    loginWithSmsCode({
      phone: code.phone,
      sms_ticket: code.sms_ticket,
      sms_verification_code: code.debug_code,
    }),
  );

  assert.equal(code.debug_code, "123456");
  assert.deepEqual(calls, [
    {
      url: "https://api.example.test/api/v1/auth/sms-login-code",
      method: "POST",
      data: { phone: "13800138000" },
      authorization: undefined,
    },
    {
      url: "https://api.example.test/api/v1/auth/sms-login",
      method: "POST",
      data: {
        phone: "13800138000",
        sms_ticket: "sms-ticket-1",
        sms_verification_code: "123456",
      },
      authorization: undefined,
    },
  ]);
  assert.equal(storage.get("stellartrail_access_token"), "access-sms");
  assert.equal(storage.get("stellartrail_refresh_token"), "refresh-sms");
});

test("registerWithPassword persists the returned session", async () => {
  const calls = [];
  const storage = installWxMock((options) => {
    calls.push({
      url: options.url,
      method: options.method,
      data: options.data,
    });
    assert.equal(options.url, "https://api.example.test/api/v1/auth/register");
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
    if (options.url.endsWith("/api/v1/auth/email-verification-code")) {
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
    if (options.url.endsWith("/api/v1/auth/captcha")) {
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
      url: "https://api.example.test/api/v1/auth/email-verification-code",
      method: "POST",
      data: { email: "bob@example.com" },
      authorization: undefined,
    },
    {
      url: "https://api.example.test/api/v1/auth/captcha",
      method: "POST",
      data: { account: "trail_bob" },
      authorization: undefined,
    },
  ]);
});

test("captcha required errors keep status code and response code", async () => {
  installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/v1/auth/login");
    options.success({
      statusCode: 428,
      data: {
        code: "captcha_required",
        message: "请完成验证码后再试",
        captcha: { captcha_type: "image", endpoint: "/api/v1/auth/captcha" },
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
      assert.equal(error.captcha.endpoint, "/api/v1/auth/captcha");
      return true;
    },
  );
});

test("not found API errors can be identified without exposing raw messages", async () => {
  const storage = installWxMock((options) => {
    assert.equal(options.url, "https://api.example.test/api/v1/me/profile");
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
      "https://api.example.test/api/v1/skills/knots/list?offset=0&limit=2",
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
  assert.equal(consumeOfflineCacheNotice(), "当前离线，正在显示已缓存内容");
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
    assert.equal(options.url, "https://api.example.test/api/v1/me/gears/stats");
    options.success({
      statusCode: 200,
      data: {
        current_count: 1,
        total_value_cents: 0,
        total_weight_g: 1200,
        by_category: [
          {
            category: "backpack_system",
            label: "背负系统",
            count: 1,
            total_weight_g: 1200,
            total_value_cents: 0,
          },
        ],
        by_status: [
          {
            status: "available",
            label: "可用",
            count: 1,
            total_weight_g: 1200,
            total_value_cents: 0,
          },
        ],
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

test("media cache returns saved files and memoizes validated file paths", async () => {
  let getFileInfoCount = 0;
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
        getFileInfoCount += 1;
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
  assert.equal(
    await resolveCachedMediaUrl("https://assets.example.test/knot.gif"),
    "wxfile://saved/knot.gif",
  );
  assert.equal(getFileInfoCount, 0);
  clearCompiledUtilityModules();
  const {
    resolveCachedMediaUrl: resolveCachedMediaUrlAfterReload,
  } = require("../.tmp-test/utils/media-cache.js");
  assert.equal(
    await resolveCachedMediaUrlAfterReload(
      "https://assets.example.test/knot.gif",
    ),
    "wxfile://saved/knot.gif",
  );
  assert.equal(getFileInfoCount, 1);
  assert.equal(
    await resolveCachedMediaUrlAfterReload(
      "https://assets.example.test/knot.gif",
    ),
    "wxfile://saved/knot.gif",
  );
  assert.equal(getFileInfoCount, 1);
});

test("media cache removes stale entries when the saved file is missing", async () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    undefined,
    {
      getFileInfo(options) {
        assert.equal(options.filePath, "wxfile://missing/knot.gif");
        options.fail({ errMsg: "file missing" });
      },
    },
  );

  const cachedKey = [...storage.keys()].find((key) =>
    String(key).startsWith("stellartrail_media_cache_v1:"),
  );
  const key =
    cachedKey ??
    `stellartrail_media_cache_v1:${encodeURIComponent(
      "https://assets.example.test/knot.gif",
    )}`;
  storage.set(key, {
    url: "https://assets.example.test/knot.gif",
    filePath: "wxfile://missing/knot.gif",
    cachedAt: "2026-05-19T00:00:00.000Z",
  });
  const {
    resolveCachedMediaUrl,
  } = require("../.tmp-test/utils/media-cache.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  assert.equal(
    await resolveCachedMediaUrl("https://assets.example.test/knot.gif"),
    "https://assets.example.test/knot.gif",
  );
});

test("opportunistic media cache queues downloads with concurrency and dedupe", async () => {
  const downloads = [];
  installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    undefined,
    {
      downloadFile(options) {
        downloads.push(options);
      },
      saveFile(options) {
        options.success({
          savedFilePath: `wxfile://saved/${downloads.length}`,
        });
      },
      getFileInfo(options) {
        options.fail({ errMsg: "missing" });
      },
    },
  );
  const { cacheMediaUrl } = require("../.tmp-test/utils/media-cache.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  cacheMediaUrl("https://assets.example.test/a.gif");
  cacheMediaUrl("https://assets.example.test/b.gif");
  cacheMediaUrl("https://assets.example.test/a.gif");
  cacheMediaUrl("https://assets.example.test/c.gif");
  cacheMediaUrl("https://assets.example.test/d.gif");

  assert.deepEqual(
    downloads.map((item) => item.url),
    ["https://assets.example.test/a.gif", "https://assets.example.test/b.gif"],
  );

  downloads[0].success({ statusCode: 200, tempFilePath: "/tmp/a.gif" });
  downloads[0].complete?.();
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.deepEqual(
    downloads.map((item) => item.url),
    [
      "https://assets.example.test/a.gif",
      "https://assets.example.test/b.gif",
      "https://assets.example.test/c.gif",
    ],
  );
});

test("media cache filters and removes cached urls with one index pass", () => {
  const storage = installWxMock(
    () => {
      throw new Error("unexpected wx.request call");
    },
    undefined,
    {
      removeSavedFile(options) {
        storage.set(`removed:${options.filePath}`, true);
        options.complete?.();
      },
    },
  );
  const urls = [
    "https://assets.example.test/a.gif",
    "https://assets.example.test/b.gif",
  ];
  storage.set("stellartrail_media_cache_index_v1", urls);
  urls.forEach((url, index) => {
    storage.set(`stellartrail_media_cache_v1:${encodeURIComponent(url)}`, {
      url,
      filePath: `wxfile://saved/${index}`,
      cachedAt: "2026-05-19T00:00:00.000Z",
    });
  });
  const {
    filterUncachedMediaUrls,
    removeCachedMediaUrls,
  } = require("../.tmp-test/utils/media-cache.js");

  assert.deepEqual(
    filterUncachedMediaUrls([
      "https://assets.example.test/a.gif",
      "https://assets.example.test/c.gif",
      "https://assets.example.test/c.gif",
    ]),
    ["https://assets.example.test/c.gif"],
  );
  assert.equal(removeCachedMediaUrls(urls), 2);
  assert.deepEqual(storage.get("stellartrail_media_cache_index_v1"), []);
  assert.equal(storage.get("removed:wxfile://saved/0"), true);
  assert.equal(storage.get("removed:wxfile://saved/1"), true);
});

test("cacheAllKnotsForOffline stores paged lists, details, and media resources", async () => {
  const requests = [];
  const downloads = [];
  const storage = installWxMock(
    (options) => {
      const path = options.url.replace("https://api.example.test", "");
      requests.push(path);
      if (path === "/api/v1/skills/knots/offline-manifest") {
        options.success({
          statusCode: 200,
          data: {
            locale: "zh-CN",
            item_count: 2,
            media_count: 3,
            estimated_bytes: 896,
            items: [
              {
                id: "bowline",
                slug: "bowline",
                title: "布林结",
                summary: "固定绳圈",
                aliases: ["单套结"],
                categories: [
                  {
                    id: "camping",
                    slug: "camping",
                    title: "露营",
                  },
                ],
                types: [],
                media: [
                  {
                    id: "thumb-bowline",
                    media_type: "thumbnail",
                    url: "/knots/bowline-thumb.png",
                    mime_type: "image/png",
                    size_bytes: 128,
                  },
                  {
                    id: "gif-bowline",
                    media_type: "draw_gif",
                    url: "/knots/bowline.gif",
                    mime_type: "image/gif",
                    size_bytes: 256,
                  },
                ],
                description: "绳圈详情",
                steps: [],
                locale: "zh-CN",
              },
              {
                id: "clove",
                slug: "clove",
                title: "丁香结",
                summary: "快速固定",
                aliases: [],
                categories: [],
                types: [],
                media: [
                  {
                    id: "gif-clove",
                    media_type: "draw_gif",
                    url: "https://cdn.example.test/knots/clove.gif",
                    mime_type: "image/gif",
                    size_bytes: 512,
                  },
                ],
                description: "固定详情",
                steps: [],
                locale: "zh-CN",
              },
            ],
          },
        });
        return;
      }
      throw new Error(`unexpected request ${path}`);
    },
    undefined,
    {
      downloadFile(options) {
        downloads.push(options.url);
        options.success({
          statusCode: 200,
          tempFilePath: `/tmp/media-${downloads.length}`,
        });
        options.complete();
      },
      saveFile(options) {
        options.success({
          savedFilePath: `wxfile://saved/${downloads.length}`,
        });
      },
      removeSavedFile(options) {
        storage.set(`removed:${options.filePath}`, true);
        options.complete?.();
      },
    },
  );
  const {
    cacheAllKnotsForOffline,
    clearKnotOfflineCache,
    deleteCachedKnot,
    getKnotOfflineCacheInventory,
    listCachedKnotPreviews,
  } = require("../.tmp-test/utils/knot-offline-cache.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  const progress = [];
  const result = await cacheAllKnotsForOffline({
    pageSize: 1,
    onProgress: (item) => progress.push(item.phase),
  });

  assert.deepEqual(requests, ["/api/v1/skills/knots/offline-manifest"]);
  assert.equal(result.items.length, 2);
  assert.deepEqual(result.items[0].aliases, ["单套结"]);
  assert.equal(result.detailCount, 2);
  assert.equal(result.mediaReadyCount, 3);
  assert.equal(result.mediaTotal, 3);
  assert.equal(result.estimatedBytes, 896);
  assert.equal(result.failedDetailCount, 0);
  assert.equal(result.failedMediaCount, 0);
  assert.deepEqual(downloads, [
    "https://assets.example.test/knots/bowline-thumb.png",
    "https://assets.example.test/knots/bowline.gif",
    "https://cdn.example.test/knots/clove.gif",
  ]);
  assert.deepEqual(progress.slice(0, 2), ["list", "media"]);
  assert.equal(hasOfflineCacheStorage(storage), true);
  assert.ok(
    offlineCacheKeys(storage).includes(
      "/api/v1/skills/knots/list?offset=0&limit=1|zh-CN|",
    ),
  );
  assert.ok(
    offlineCacheKeys(storage).includes(
      "/api/v1/skills/knots/detail/bowline|zh-CN|",
    ),
  );
  assert.deepEqual(getKnotOfflineCacheInventory("zh-CN"), {
    cachedCount: 2,
    totalCount: 2,
    uncachedCount: 0,
    items: listCachedKnotPreviews("zh-CN"),
  });
  assert.deepEqual(
    listCachedKnotPreviews("zh-CN").map((item) => ({
      id: item.id,
      title: item.title,
      categoryText: item.categoryText,
    })),
    [
      {
        id: "bowline",
        title: "布林结",
        categoryText: "露营",
      },
      {
        id: "clove",
        title: "丁香结",
        categoryText: "绳结",
      },
    ],
  );

  const afterDelete = deleteCachedKnot("bowline", "zh-CN");
  assert.equal(afterDelete.cachedCount, 1);
  assert.equal(afterDelete.totalCount, 2);
  assert.equal(afterDelete.uncachedCount, 1);
  assert.deepEqual(
    afterDelete.items.map((item) => item.id),
    ["clove"],
  );
  assert.ok(
    !offlineCacheKeys(storage).includes(
      "/api/v1/skills/knots/detail/bowline|zh-CN|",
    ),
  );
  assert.equal(storage.get("removed:wxfile://saved/1"), true);
  assert.equal(storage.get("removed:wxfile://saved/2"), true);

  const afterClear = clearKnotOfflineCache("zh-CN");
  assert.equal(afterClear.cachedCount, 0);
  assert.equal(afterClear.totalCount, 2);
  assert.equal(afterClear.uncachedCount, 2);
  assert.deepEqual(listCachedKnotPreviews("zh-CN"), []);
});

test("prepareAllKnotsOfflineCache only reads the manifest before confirmation", async () => {
  const requests = [];
  const downloads = [];
  const storage = installWxMock(
    (options) => {
      const path = options.url.replace("https://api.example.test", "");
      requests.push(path);
      assert.equal(path, "/api/v1/skills/knots/offline-manifest");
      options.success({
        statusCode: 200,
        data: {
          locale: "zh-CN",
          item_count: 1,
          media_count: 1,
          estimated_bytes: 128,
          items: [
            {
              id: "bowline",
              slug: "bowline",
              title: "布林结",
              summary: "固定绳圈",
              aliases: ["单套结"],
              categories: [],
              types: [],
              media: [
                {
                  id: "thumb-bowline",
                  media_type: "thumbnail",
                  url: "/knots/bowline-thumb.png",
                  mime_type: "image/png",
                  size_bytes: 128,
                },
              ],
              description: "绳圈详情",
              steps: [],
              locale: "zh-CN",
            },
          ],
        },
      });
    },
    undefined,
    {
      downloadFile(options) {
        downloads.push(options.url);
      },
    },
  );
  const {
    prepareAllKnotsOfflineCache,
  } = require("../.tmp-test/utils/knot-offline-cache.js");
  require("../.tmp-test/utils/network-state.js").initNetworkState();

  const plan = await prepareAllKnotsOfflineCache({ pageSize: 1 });

  assert.deepEqual(requests, ["/api/v1/skills/knots/offline-manifest"]);
  assert.equal(plan.items.length, 1);
  assert.equal(plan.estimatedBytes, 128);
  assert.deepEqual(downloads, []);
  assert.equal(hasOfflineCacheStorage(storage), false);
});

function hasOfflineCacheStorage(storage) {
  return [...storage.keys()].some((key) =>
    String(key).startsWith("stellartrail_offline_cache_v1:"),
  );
}

function offlineCacheKeys(storage) {
  return [...storage.entries()]
    .filter(([key]) => String(key).startsWith("stellartrail_offline_cache_v1:"))
    .map(([, value]) => value && value.key)
    .filter(Boolean);
}
