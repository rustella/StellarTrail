#!/usr/bin/env node

import fs from "node:fs";

const DEFAULT_URL =
  "https://api.example.invalid/api/v1/skills/knots/offline-manifest";

const RISK_PATTERNS = [
  ["human_safety", /承载人体|人身|救命|救生|life\s*sav|lifesav|life\s*safety/i],
  ["rescue", /救援|消防|急救|搜寻|援救|rescue|search\s*and\s*rescue|SAR/i],
  [
    "climbing_height",
    /攀岩|攀登|攀爬|高空|探洞|树艺|登山|保护系统|地锚|登山扣|climb|climbing|caving|arborist|height/i,
  ],
  [
    "industrial_load",
    /吊装|吊桶|起重|工业|承重|载荷|负载|承载|悬挂|load|weight.?bearing|bearing|hoist|rigging/i,
  ],
  [
    "water_marine",
    /航海|船|小船|划船|海事|帆船|锚|boating|sailing|marine|nautical/i,
  ],
  [
    "strong_safety_claim",
    /安全|可靠|牢固|稳定|结实|高强度|防止.*滑动|不会.*滑脱|secure|safe|safety|reliable|strong|stable/i,
  ],
  ["essential_claim", /必备|essential|must.?have/i],
];

const CRITICAL_PATTERNS = [
  /救命/,
  /救援安全带/,
  /承载人体/,
  /吊装/,
  /系在攀岩安全带/,
  /支撑人员/,
];

const args = process.argv.slice(2);

function argValue(name) {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
}

function hasFlag(name) {
  return args.includes(name);
}

async function readManifest() {
  const file = argValue("--file");
  if (file) {
    return fs.readFileSync(file, "utf8");
  }
  if (hasFlag("--stdin")) {
    return fs.readFileSync(0, "utf8");
  }
  const url = argValue("--url") ?? DEFAULT_URL;
  const response = await fetch(url, {
    headers: { "X-StellarTrail-Locale": "zh-CN" },
  });
  if (!response.ok) {
    throw new Error(
      `request failed: ${response.status} ${response.statusText}`,
    );
  }
  return response.text();
}

function pushField(fields, field, value) {
  if (typeof value === "string" && value.trim()) {
    fields.push([field, value.trim()]);
  }
}

function publicFields(item, includeTaxonomy) {
  const fields = [];
  pushField(fields, "title", item.title);
  pushField(fields, "slug", item.slug);
  pushField(fields, "summary", item.summary);
  pushField(fields, "description", item.description);
  if (Array.isArray(item.steps)) {
    item.steps.forEach((step, index) =>
      pushField(fields, `steps[${index}]`, step),
    );
  }
  if (includeTaxonomy && Array.isArray(item.categories)) {
    item.categories.forEach((category, index) => {
      pushField(fields, `categories[${index}].id`, category.id);
      pushField(fields, `categories[${index}].title`, category.title);
    });
  }
  if (includeTaxonomy && Array.isArray(item.types)) {
    item.types.forEach((type, index) => {
      pushField(fields, `types[${index}].id`, type.id);
      pushField(fields, `types[${index}].title`, type.title);
    });
  }
  return fields;
}

function scanItems(items, includeTaxonomy) {
  const byPattern = Object.fromEntries(RISK_PATTERNS.map(([key]) => [key, []]));
  const critical = [];
  for (const item of items) {
    for (const [field, value] of publicFields(item, includeTaxonomy)) {
      for (const [key, pattern] of RISK_PATTERNS) {
        if (pattern.test(value)) {
          byPattern[key].push({ id: item.id, title: item.title, field, value });
        }
      }
      if (CRITICAL_PATTERNS.some((pattern) => pattern.test(value))) {
        critical.push({ id: item.id, title: item.title, field, value });
      }
    }
  }
  return { byPattern, critical };
}

function summarize(data) {
  const items = Array.isArray(data.items) ? data.items : [];
  const withTaxonomy = scanItems(items, true);
  const textOnly = scanItems(items, false);
  return {
    locale: data.locale,
    item_count: items.length,
    reported_item_count: data.item_count,
    media_count: data.media_count,
    risk_item_counts: Object.fromEntries(
      Object.entries(withTaxonomy.byPattern).map(([key, matches]) => [
        key,
        new Set(matches.map((match) => match.id)).size,
      ]),
    ),
    text_risk_item_counts: Object.fromEntries(
      Object.entries(textOnly.byPattern).map(([key, matches]) => [
        key,
        new Set(matches.map((match) => match.id)).size,
      ]),
    ),
    critical_count: textOnly.critical.length,
    critical_examples: textOnly.critical.slice(0, 40),
    examples: Object.fromEntries(
      Object.entries(textOnly.byPattern).map(([key, matches]) => [
        key,
        matches.slice(0, 12),
      ]),
    ),
  };
}

const manifest = JSON.parse(await readManifest());
const summary = summarize(manifest);
console.log(JSON.stringify(summary, null, 2));

if (hasFlag("--fail-on-critical") && summary.critical_count > 0) {
  process.exitCode = 2;
}
