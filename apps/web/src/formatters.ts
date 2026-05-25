export interface GearNameParts {
  brand?: string | null;
  name: string;
  model?: string | null;
}

const currencyFormatter = new Intl.NumberFormat("zh-CN", {
  style: "currency",
  currency: "CNY",
  minimumFractionDigits: 2,
});

const compactCurrencyFormatter = new Intl.NumberFormat("zh-CN", {
  style: "currency",
  currency: "CNY",
  maximumFractionDigits: 0,
});

export function formatCurrency(cents?: number | null): string {
  if (cents === undefined || cents === null) {
    return "—";
  }
  return currencyFormatter.format(cents / 100).replace("CN¥", "¥");
}

export function formatCompactCurrency(cents?: number | null): string {
  if (cents === undefined || cents === null) {
    return "—";
  }
  return compactCurrencyFormatter.format(cents / 100).replace("CN¥", "¥");
}

export function formatWeight(weightG?: number | null): string {
  if (weightG === undefined || weightG === null) {
    return "—";
  }
  if (weightG >= 1000) {
    return `${(weightG / 1000).toFixed(1)} kg`;
  }
  return `${weightG} g`;
}

export function formatDate(date?: string | null): string {
  return date || "—";
}

export function joinGearName(parts: GearNameParts): string {
  const brand = parts.brand?.trim();
  const name = parts.name.trim();
  const model = parts.model?.trim();
  const prefix =
    brand && !name.toLowerCase().includes(brand.toLowerCase())
      ? `${brand} ${name}`
      : name;
  return model ? `${prefix} · ${model}` : prefix;
}

export function toPriceCents(value: string): number | null {
  const normalized = value.trim();
  if (!normalized) {
    return null;
  }
  const parsed = Number(normalized);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return null;
  }
  return Math.round(parsed * 100);
}

export function fromPriceCents(cents?: number | null): string {
  if (cents === undefined || cents === null) {
    return "";
  }
  return String(cents / 100);
}
