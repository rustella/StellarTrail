const test = require("node:test");
const assert = require("node:assert/strict");

const {
  buildGearPayload,
  formatGearPrice,
  formatGearWeight,
  parseTagsInput,
} = require("../.tmp-test/utils/gear-utils.js");

test("formatGearWeight formats grams and kilograms for mobile cards", () => {
  assert.equal(formatGearWeight(null), "未记录");
  assert.equal(formatGearWeight(undefined), "未记录");
  assert.equal(formatGearWeight(315), "315 g");
  assert.equal(formatGearWeight(1200), "1.2 kg");
});

test("formatGearPrice formats cents as Chinese Yuan", () => {
  assert.equal(formatGearPrice(null), "未记录");
  assert.equal(formatGearPrice(undefined), "未记录");
  assert.equal(formatGearPrice(63900), "¥639");
  assert.equal(formatGearPrice(63950), "¥639.50");
});

test("parseTagsInput accepts comma, Chinese comma and semicolon separators", () => {
  assert.deepEqual(parseTagsInput(" 冬季,电子， 备用；EDC;  "), [
    "冬季",
    "电子",
    "备用",
    "EDC",
  ]);
});

test("buildGearPayload trims text and converts UI inputs to API units", () => {
  assert.deepEqual(
    buildGearPayload({
      category: "electronics_system",
      name: "  NITECORE 头灯  ",
      brand: " NITECORE奈特科尔 ",
      model: "",
      color: "黑色",
      material: "",
      capacity: "20000mAh",
      size: "",
      description: " 冬季徒步备用电源 ",
      weightText: "0.315",
      warmthIndex: "",
      waterproofIndex: "IPX8",
      purchaseDate: "2026-01-22",
      purchasePriceText: "639.5",
      expiryOrWarrantyDate: "",
      purchaseLocation: "京东",
      status: "available",
      storageLocation: "装备柜 A1",
      tagsText: "冬季, 电子；备用",
      shareEnabled: true,
      notes: " 充满电后入库 ",
    }),
    {
      category: "electronics_system",
      name: "NITECORE 头灯",
      brand: "NITECORE奈特科尔",
      model: null,
      color: "黑色",
      material: null,
      capacity: "20000mAh",
      size: null,
      description: "冬季徒步备用电源",
      weight_g: 315,
      warmth_index: null,
      waterproof_index: "IPX8",
      purchase_date: "2026-01-22",
      purchase_price_cents: 63950,
      expiry_or_warranty_date: null,
      purchase_location: "京东",
      status: "available",
      storage_location: "装备柜 A1",
      tags: ["冬季", "电子", "备用"],
      share_enabled: true,
      notes: "充满电后入库",
    },
  );
});

test("buildGearPayload rejects missing names and invalid numbers", () => {
  assert.throws(
    () =>
      buildGearPayload({
        category: "electronics_system",
        name: " ",
        status: "available",
      }),
    /装备名称不能为空/,
  );
  assert.throws(
    () =>
      buildGearPayload({
        category: "electronics_system",
        name: "头灯",
        status: "available",
        weightText: "abc",
      }),
    /重量必须是数字/,
  );
  assert.throws(
    () =>
      buildGearPayload({
        category: "electronics_system",
        name: "头灯",
        status: "available",
        purchasePriceText: "-1",
      }),
    /价格不能为负数/,
  );
});
