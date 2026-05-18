const test = require("node:test");
const assert = require("node:assert/strict");

const {
  buildGearPayload,
  combineSpecValue,
  createGearTagSuggestionViews,
  createGearTagViews,
  formatGearPrice,
  formatGearWeight,
  GEAR_SPEC_FIELDS,
  GEAR_TAG_COLOR_OPTIONS,
  GEAR_WEIGHT_UNIT_OPTIONS,
  getGearSpecFieldViews,
  addGearTagViews,
  parseTagsInput,
  PURCHASE_LOCATION_OPTIONS,
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
  assert.equal(formatGearPrice(1299, "USD"), "USD 12.99");
  assert.equal(formatGearPrice(1200, "JPY"), "JPY 1200");
});

test("parseTagsInput accepts comma, Chinese comma and semicolon separators", () => {
  assert.deepEqual(parseTagsInput(" 冬季,电子， 备用；EDC;  "), [
    "冬季",
    "电子",
    "备用",
    "EDC",
  ]);
});

test("purchase location options include ecommerce and outdoor stores", () => {
  assert.ok(PURCHASE_LOCATION_OPTIONS.includes("亚马逊"));
  assert.ok(PURCHASE_LOCATION_OPTIONS.includes("迪卡侬"));
  assert.ok(PURCHASE_LOCATION_OPTIONS.includes("三夫户外"));
  assert.ok(PURCHASE_LOCATION_OPTIONS.includes("REI"));
  assert.ok(PURCHASE_LOCATION_OPTIONS.includes("Backcountry"));
});

test("buildGearPayload trims text and converts UI inputs to API units", () => {
  assert.deepEqual(
    buildGearPayload({
      category: "electronics_system",
      name: "  NITECORE 头灯  ",
      brand: " NITECORE奈特科尔 ",
      model: "",
      description: " 冬季徒步备用电源 ",
      weightText: "0.315",
      purchaseDate: "2026-01-22",
      officialPriceText: "699",
      officialPriceCurrency: "CNY",
      purchasePriceText: "639.5",
      purchasePriceCurrency: "CNY",
      purchaseLocation: "京东",
      status: "available",
      storageLocation: "装备柜 A1",
      specs: {
        battery_capacity: "20000 mAh",
        waterproof_rating: "IPX8",
        opening_style: "拉链",
      },
      tags: createGearTagViews(["冬季", "电子", "备用"], {
        冬季: "blue",
        电子: "teal",
        备用: "amber",
      }),
      shareEnabled: true,
      notes: " 充满电后入库 ",
    }),
    {
      category: "electronics_system",
      name: "NITECORE 头灯",
      brand: "NITECORE奈特科尔",
      model: null,
      description: "冬季徒步备用电源",
      weight_g: 315,
      purchase_date: "2026-01-22",
      official_price_cents: 69900,
      official_price_currency: "CNY",
      purchase_price_cents: 63950,
      purchase_price_currency: "CNY",
      purchase_location: "京东",
      status: "available",
      storage_location: "装备柜 A1",
      specs: {
        battery_capacity: "20000 mAh",
        waterproof_rating: "IPX8",
      },
      tags: ["冬季", "电子", "备用"],
      tag_colors: {
        冬季: "blue",
        电子: "teal",
        备用: "amber",
      },
      share_enabled: true,
      notes: "充满电后入库",
    },
  );
});

test("gear tag helpers create colored tag views and payload maps", () => {
  assert.deepEqual(
    GEAR_TAG_COLOR_OPTIONS.map((item) => item.value),
    ["teal", "blue", "violet", "rose", "orange", "amber", "green", "slate"],
  );
  const tags = createGearTagViews([" 冬季 ", "电子", "冬季"], {
    冬季: "blue",
    电子: "teal",
  });
  assert.deepEqual(tags, [
    { name: "冬季", color: "blue", colorClass: "tag-color-blue" },
    { name: "电子", color: "teal", colorClass: "tag-color-teal" },
  ]);
  assert.deepEqual(
    createGearTagSuggestionViews([{ tag: "冬季", color: "blue" }]),
    [{ name: "冬季", color: "blue", colorClass: "tag-color-blue" }],
  );
  assert.deepEqual(
    addGearTagViews(tags, "备用，冬季", "amber").map((item) => ({
      name: item.name,
      color: item.color,
    })),
    [
      { name: "冬季", color: "blue" },
      { name: "电子", color: "teal" },
      { name: "备用", color: "amber" },
    ],
  );
});

test("buildGearPayload converts selectable weight units to grams", () => {
  assert.equal(
    buildGearPayload({
      category: "electronics_system",
      name: "充电宝",
      weightText: "0.315",
      weightUnit: "kg",
      status: "available",
    }).weight_g,
    315,
  );
  assert.equal(
    buildGearPayload({
      category: "electronics_system",
      name: "充电宝",
      weightText: "315",
      weightUnit: "g",
      status: "available",
    }).weight_g,
    315,
  );
  assert.equal(
    buildGearPayload({
      category: "electronics_system",
      name: "充电宝",
      weightText: "1",
      weightUnit: "lb",
      status: "available",
    }).weight_g,
    454,
  );
});

test("buildGearPayload keeps purchase location nullable or selected", () => {
  assert.equal(
    buildGearPayload({
      category: "electronics_system",
      name: "充电宝",
      purchaseLocation: "  ",
      status: "available",
    }).purchase_location,
    null,
  );
  assert.equal(
    buildGearPayload({
      category: "electronics_system",
      name: "充电宝",
      purchaseLocation: " 亚马逊 ",
      status: "available",
    }).purchase_location,
    "亚马逊",
  );
  assert.equal(
    buildGearPayload({
      category: "electronics_system",
      name: "充电宝",
      purchaseLocation: " 迪卡侬 ",
      status: "available",
    }).purchase_location,
    "迪卡侬",
  );
});

test("gear form unit options are ordered by common outdoor usage", () => {
  assert.deepEqual(
    GEAR_WEIGHT_UNIT_OPTIONS.map((item) => item.value),
    ["kg", "g", "lb", "oz"],
  );
  assert.deepEqual(GEAR_SPEC_FIELDS.backpack_system[0].units, [
    "L",
    "ml",
    "fl oz",
  ]);
  assert.deepEqual(GEAR_SPEC_FIELDS.backpack_system[1].units, [
    "kg",
    "g",
    "lb",
  ]);
  assert.equal(GEAR_SPEC_FIELDS.backpack_system[2].key, "back_length");
  assert.deepEqual(GEAR_SPEC_FIELDS.backpack_system[2].units, ["cm", "in"]);
  assert.equal(GEAR_SPEC_FIELDS.backpack_system[3].key, "backpack_size");
  assert.deepEqual(GEAR_SPEC_FIELDS.backpack_system[3].units, [
    "",
    "XS",
    "S",
    "M",
    "L",
    "XL",
    "XXL",
    "均码",
  ]);
  assert.deepEqual(GEAR_SPEC_FIELDS.backpack_system[3].unitLabels, [
    "选择尺码",
    "XS",
    "S",
    "M",
    "L",
    "XL",
    "XXL",
    "均码",
  ]);
  assert.equal(GEAR_SPEC_FIELDS.backpack_system[3].choiceOnly, true);
  assert.deepEqual(GEAR_SPEC_FIELDS.kitchen_system[1].units, [
    "L",
    "ml",
    "fl oz",
  ]);
  assert.deepEqual(GEAR_SPEC_FIELDS.electronics_system[0].units, ["mAh", "Wh"]);
  assert.deepEqual(GEAR_SPEC_FIELDS.consumable[1].units, [
    "g",
    "ml",
    "kg",
    "L",
    "oz",
  ]);
  assert.deepEqual(GEAR_SPEC_FIELDS.walking_system[0].units, [
    "cm",
    "EU",
    "US",
    "UK",
    "in",
  ]);
});

test("spec unit helpers preserve selected common units", () => {
  assert.equal(combineSpecValue("45", "L"), "45 L");
  assert.equal(combineSpecValue("500", "ml"), "500 ml");
  assert.equal(combineSpecValue("20", "fl oz"), "20 fl oz");
  assert.equal(combineSpecValue("42", "EU"), "42 EU");
  assert.equal(combineSpecValue("120", "cm"), "120 cm");
  assert.equal(combineSpecValue("", "M"), "M");
});

test("getGearSpecFieldViews splits value and unit for category specs", () => {
  const fields = getGearSpecFieldViews("electronics_system", {
    battery_capacity: "20000 mAh",
  });
  const battery = fields.find((field) => field.key === "battery_capacity");
  assert.equal(battery.valueText, "20000");
  assert.equal(battery.unitLabel, "mAh");
});

test("getGearSpecFieldViews orders ranked category keys first", () => {
  const fields = getGearSpecFieldViews(
    "electronics_system",
    {
      battery_capacity: "20000 mAh",
      waterproof_rating: "IPX4",
    },
    ["waterproof_rating", "unknown_key", "battery_capacity"],
  );
  assert.deepEqual(
    fields.slice(0, 3).map((field) => field.key),
    ["waterproof_rating", "battery_capacity", "rated_energy"],
  );
});

test("getGearSpecFieldViews preserves default order without rankings", () => {
  const fields = getGearSpecFieldViews("electronics_system", {});
  assert.deepEqual(
    fields.slice(0, 3).map((field) => field.key),
    ["battery_capacity", "rated_energy", "output_power"],
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
