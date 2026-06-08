const test = require("node:test");
const assert = require("node:assert/strict");

function loadClientVersionUtils() {
  for (const key of Object.keys(require.cache)) {
    if (key.includes("/.tmp-test/utils/client-version.js")) {
      delete require.cache[key];
    }
  }
  return require("../.tmp-test/utils/client-version.js");
}

test("semantic version comparison uses numeric parts", () => {
  const { compareSemanticVersions, isSemanticVersionLessThanOrEqual } =
    loadClientVersionUtils();

  assert.equal(compareSemanticVersions("0.10.0", "0.2.9") > 0, true);
  assert.equal(isSemanticVersionLessThanOrEqual("0.2.2", "0.2.2"), true);
  assert.equal(isSemanticVersionLessThanOrEqual("0.2.1", "0.2.2"), true);
  assert.equal(isSemanticVersionLessThanOrEqual("0.2.3", "0.2.2"), false);
});

test("client identity version filters future WeChat releases", () => {
  const { filterClientVersionsForCurrentVersion } = loadClientVersionUtils();
  const versions = [
    { version: "0.3.0" },
    { version: "0.2.3" },
    { version: "0.2.2" },
    { version: "0.2.1" },
    { version: "0.1.0" },
  ];

  assert.deepEqual(
    filterClientVersionsForCurrentVersion(versions, "0.2.2").map(
      (version) => version.version,
    ),
    ["0.2.2", "0.2.1", "0.1.0"],
  );
});

test("current WeChat version prefers client identity over native version", () => {
  global.getApp = () => ({
    globalData: {
      clientIdentity: "wechat/0.2.2",
    },
  });
  global.wx = {
    getAccountInfoSync() {
      return { miniProgram: { version: "0.2.1" } };
    },
  };
  const { getCurrentWechatClientVersion } = loadClientVersionUtils();

  assert.equal(getCurrentWechatClientVersion(), "0.2.2");
});

test("resolver falls back to native version and ignores dev versions", () => {
  const { resolveWechatClientVersion } = loadClientVersionUtils();

  assert.equal(
    resolveWechatClientVersion({
      appClientIdentity: "wechat/dev",
      configClientIdentity: "wechat/beta",
      nativeVersion: "0.2.1",
    }),
    "0.2.1",
  );
  assert.equal(
    resolveWechatClientVersion({
      appClientIdentity: "",
      configClientIdentity: "wechat/dev",
      nativeVersion: "dev",
    }),
    undefined,
  );
});

test("invalid current version keeps the original list visible", () => {
  const { filterClientVersionsForCurrentVersion } = loadClientVersionUtils();
  const versions = [{ version: "0.3.0" }, { version: "0.2.2" }];

  assert.deepEqual(filterClientVersionsForCurrentVersion(versions, "dev"), [
    { version: "0.3.0" },
    { version: "0.2.2" },
  ]);
});
