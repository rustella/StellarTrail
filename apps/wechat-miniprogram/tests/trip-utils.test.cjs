const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const {
  buildTripInvitationShareTitle,
  buildTripInvitationText,
  buildTripJoinPath,
  extractTripInvitationToken,
  formatTripDurationText,
} = require("../.tmp-test/utils/trip-utils.js");

const miniRoot = path.resolve(__dirname, "..", "miniprogram");

function plan(overrides) {
  return {
    itinerary_day_count: 0,
    start_date: null,
    end_date: null,
    ...overrides,
  };
}

function read(rel) {
  return fs.readFileSync(path.join(miniRoot, rel), "utf8");
}

test("trip duration prefers itinerary day count", () => {
  assert.equal(
    formatTripDurationText(
      plan({
        itinerary_day_count: 3,
        start_date: "2026-05-26",
        end_date: "2026-05-27",
      }),
    ),
    "3天2夜",
  );
  assert.equal(
    formatTripDurationText(plan({ itinerary_day_count: 1 })),
    "1天",
  );
});

test("trip duration falls back to inclusive date range", () => {
  assert.equal(
    formatTripDurationText(
      plan({
        start_date: "2026-05-26",
        end_date: "2026-05-29",
      }),
    ),
    "4天3夜",
  );
});

test("trip duration handles missing or invalid dates", () => {
  assert.equal(formatTripDurationText(plan({})), "未设置天数");
  assert.equal(
    formatTripDurationText(
      plan({
        start_date: "2026-05-29",
        end_date: "2026-05-26",
      }),
    ),
    "未设置天数",
  );
  assert.equal(
    formatTripDurationText(
      plan({
        start_date: "2026-02-30",
        end_date: "2026-03-01",
      }),
    ),
    "未设置天数",
  );
});

test("trip list card shows duration instead of enabled section count", () => {
  const wxml = read("pages/trips/index.wxml");
  const ts = read("pages/trips/index.ts");
  const source = `${wxml}\n${ts}`;

  assert.match(source, /durationText/);
  assert.match(source, /formatTripDurationText/);
  assert.doesNotMatch(source, /sectionText/);
  assert.doesNotMatch(source, /个板块/);
});

test("trip invitation helpers build share content and extract tokens", () => {
  const token = "c215d1a7-b7d1-4735-bcc4-38a08cbbccf1";

  assert.equal(
    buildTripJoinPath(token),
    `/pages/trips/join/index?token=${token}`,
  );
  assert.equal(
    buildTripInvitationShareTitle("银湖山"),
    "邀请你加入「银湖山」多人行程",
  );
  assert.equal(
    buildTripInvitationText("银湖山", token),
    [
      "邀请你加入「银湖山」多人行程",
      `邀请口令：${token}`,
      "打开小程序「寻径星野」- 行程 - 加入，粘贴口令加入。",
    ].join("\n"),
  );
  assert.equal(extractTripInvitationToken(token.toUpperCase()), token);
  assert.equal(
    extractTripInvitationToken(
      `邀请口令：${token}\n/pages/trips/join/index?token=${token}`,
    ),
    token,
  );
  assert.equal(extractTripInvitationToken("没有口令"), "");
});
