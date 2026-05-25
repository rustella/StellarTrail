import { describe, expect, it } from "vitest";

import {
  formatCurrency,
  formatDate,
  formatWeight,
  joinGearName,
} from "./formatters";

describe("formatters", () => {
  it("formats cents as RMB values", () => {
    expect(formatCurrency(3106442)).toBe("¥31,064.42");
    expect(formatCurrency(null)).toBe("—");
  });

  it("formats weights with kilogram fallback", () => {
    expect(formatWeight(860)).toBe("860 g");
    expect(formatWeight(16085)).toBe("16.1 kg");
    expect(formatWeight(undefined)).toBe("—");
  });

  it("joins gear brand, name, and model without duplicates", () => {
    expect(
      joinGearName({
        brand: "NITECORE",
        name: "充电宝",
        model: "SUMMIT 20000",
      }),
    ).toBe("NITECORE 充电宝 · SUMMIT 20000");
    expect(
      joinGearName({ brand: "NITECORE", name: "NITECORE 充电宝", model: null }),
    ).toBe("NITECORE 充电宝");
  });

  it("keeps empty dates readable", () => {
    expect(formatDate("2026-01-22")).toBe("2026-01-22");
    expect(formatDate(null)).toBe("—");
  });
});
