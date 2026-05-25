#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CORE_MEDIA = [
  ["local_thumbnail", "thumbnail", "image/webp"],
  ["local_preview", "preview", "image/webp"],
  ["local_draw_gif", "draw_gif", "image/gif"],
  ["local_360_gif", "turntable_gif", "image/gif"],
  ["local_draw_mp4", "draw_mp4", "video/mp4"],
  ["local_360_mp4", "turntable_mp4", "video/mp4"],
];

const DEFAULT_METADATA =
  ".hermes/local/knots3d/metadata/knots3d_bilingual_metadata.json";
const DEFAULT_API_BASE_URL = "http://127.0.0.1:8080";

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const apiBaseUrl = trimTrailingSlash(
    options.apiBaseUrl ??
      process.env.STELLARTRAIL_API_BASE_URL ??
      DEFAULT_API_BASE_URL,
  );
  const metadataPath = path.resolve(
    options.metadata ?? process.env.KNOTS3D_METADATA_PATH ?? DEFAULT_METADATA,
  );
  const mediaRoot = path.resolve(
    options.mediaRoot ??
      process.env.KNOTS3D_MEDIA_ROOT ??
      path.dirname(path.dirname(metadataPath)),
  );
  const token = options.token ?? process.env.STELLARTRAIL_ADMIN_TOKEN;
  const concurrency = Number(
    options.concurrency ?? process.env.KNOTS3D_UPLOAD_CONCURRENCY ?? 4,
  );
  if (!Number.isInteger(concurrency) || concurrency < 1 || concurrency > 16) {
    throw new Error("--concurrency must be an integer in 1..=16");
  }
  const dryRun = Boolean(options.dryRun);
  const verifyOnly = Boolean(options.verifyOnly);
  if (!token && !dryRun && !verifyOnly) {
    throw new Error(
      "missing admin token: pass --token or STELLARTRAIL_ADMIN_TOKEN",
    );
  }

  const plan = await buildUploadPlan(metadataPath, mediaRoot);
  const limit =
    options.limit === undefined ? plan.length : Number(options.limit);
  const selected = Number.isFinite(limit) ? plan.slice(0, limit) : plan;
  const expectedKnots = new Set(plan.map((item) => item.knotId)).size;
  const expectedCoreMedia = expectedKnots * CORE_MEDIA.length;
  console.log(
    JSON.stringify({
      metadataPath,
      mediaRoot,
      plannedUploads: plan.length,
      selectedUploads: selected.length,
      expectedKnots,
      expectedCoreMedia,
      apiBaseUrl,
      dryRun,
      verifyOnly,
    }),
  );

  if (dryRun) {
    return;
  }

  if (!verifyOnly) {
    const uploadResults = await mapLimit(selected, concurrency, (item) =>
      uploadOne(apiBaseUrl, token, item),
    );
    const failed = uploadResults.filter((result) => !result.ok);
    console.log(
      JSON.stringify({
        uploaded: uploadResults.length - failed.length,
        failed: failed.length,
      }),
    );
    if (failed.length > 0) {
      console.error(JSON.stringify(failed.slice(0, 20), null, 2));
      process.exitCode = 1;
      return;
    }
  }

  const verified = await verifyPublicReadApi(apiBaseUrl, token, plan);
  console.log(JSON.stringify(verified));
  if (verified.missing.length > 0 || verified.assetsUrls > 0) {
    process.exitCode = 1;
  }
}

export async function buildUploadPlan(metadataPath, mediaRoot) {
  const raw = JSON.parse(await readFile(metadataPath, "utf8"));
  const items = Array.isArray(raw.items) ? raw.items : [];
  const plan = [];
  for (const item of items) {
    const knotId = item.id ?? item.english_slug;
    if (!knotId || !item.local_media) continue;
    for (const [sourceKey, assetId, mimeType] of CORE_MEDIA) {
      const sourcePath = item.local_media[sourceKey];
      if (!sourcePath) continue;
      plan.push({
        knotId,
        assetId,
        mediaType: assetId,
        mimeType,
        filePath: resolveMediaPath(mediaRoot, sourcePath),
        relativeSourcePath: normalizeSlash(sourcePath),
      });
    }
  }
  return plan;
}

async function uploadOne(apiBaseUrl, token, item) {
  try {
    const bytes = await readFile(item.filePath);
    const form = new FormData();
    form.set("media_type", item.mediaType);
    form.set("source_name", "Knots 3D");
    form.set("source_path", item.relativeSourcePath);
    form.set("attribution", "Knots 3D");
    form.set("license_note", "Use only after authorization is confirmed.");
    form.set(
      "file",
      new Blob([bytes], { type: item.mimeType }),
      path.basename(item.filePath),
    );
    const response = await fetch(
      `${apiBaseUrl}/api/admin/skills/knots/${encodeURIComponent(item.knotId)}/media/${encodeURIComponent(item.assetId)}`,
      {
        method: "PUT",
        headers: { authorization: `Bearer ${token}` },
        body: form,
      },
    );
    if (!response.ok) {
      return {
        ok: false,
        item,
        status: response.status,
        body: await response.text(),
      };
    }
    const body = await response.json();
    return { ok: true, item, url: body.media?.url };
  } catch (error) {
    return { ok: false, item, error: String(error?.message ?? error) };
  }
}

async function verifyPublicReadApi(apiBaseUrl, token, plan) {
  const expected = new Map();
  for (const item of plan) {
    const set = expected.get(item.knotId) ?? new Set();
    set.add(item.assetId);
    expected.set(item.knotId, set);
  }
  const missing = [];
  let verifiedAssets = 0;
  let assetsUrls = 0;
  for (const [knotId, assetIds] of expected.entries()) {
    const response = await fetch(
      `${apiBaseUrl}/api/skills/knots/detail/${encodeURIComponent(knotId)}`,
      {
        headers: token ? { authorization: `Bearer ${token}` } : undefined,
      },
    );
    if (!response.ok) {
      missing.push({
        knotId,
        status: response.status,
        reason: "detail_failed",
      });
      continue;
    }
    const detail = await response.json();
    const media = Array.isArray(detail.media) ? detail.media : [];
    const found = new Set(media.map((asset) => asset.id));
    for (const assetId of assetIds) {
      if (!found.has(assetId)) missing.push({ knotId, assetId });
    }
    verifiedAssets += media.length;
    assetsUrls += media.filter(
      (asset) =>
        typeof asset.url === "string" && asset.url.includes("/assets/"),
    ).length;
  }
  return {
    verifiedKnots: expected.size,
    verifiedAssets,
    expectedAssets: plan.length,
    missing,
    assetsUrls,
  };
}

async function mapLimit(items, concurrency, worker) {
  const results = new Array(items.length);
  let index = 0;
  async function run() {
    while (index < items.length) {
      const current = index++;
      results[current] = await worker(items[current]);
      if ((current + 1) % 50 === 0) {
        console.log(
          JSON.stringify({ progress: current + 1, total: items.length }),
        );
      }
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(concurrency, items.length) }, run),
  );
  return results;
}

function parseArgs(args) {
  const options = {};
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--dry-run") options.dryRun = true;
    else if (arg === "--verify-only") options.verifyOnly = true;
    else if (arg === "--api-base-url")
      options.apiBaseUrl = requiredValue(args, ++index, arg);
    else if (arg === "--token")
      options.token = requiredValue(args, ++index, arg);
    else if (arg === "--metadata")
      options.metadata = requiredValue(args, ++index, arg);
    else if (arg === "--media-root")
      options.mediaRoot = requiredValue(args, ++index, arg);
    else if (arg === "--concurrency")
      options.concurrency = requiredValue(args, ++index, arg);
    else if (arg === "--limit")
      options.limit = requiredValue(args, ++index, arg);
    else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return options;
}

function requiredValue(args, index, flag) {
  const value = args[index];
  if (!value || value.startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return value;
}

function resolveMediaPath(mediaRoot, sourcePath) {
  if (path.isAbsolute(sourcePath)) return sourcePath;
  const normalized = sourcePath
    .replace(/^\.hermes\/local\/knots3d\/?/, "")
    .replace(/^knots3d\/?/, "");
  return path.resolve(mediaRoot, normalized);
}

function normalizeSlash(value) {
  return String(value).replaceAll("\\", "/");
}

function trimTrailingSlash(value) {
  return value.replace(/\/$/, "");
}

function printHelp() {
  const command = path.basename(fileURLToPath(import.meta.url));
  console.log(`Usage: node scripts/${command} [options]

Uploads Knots3D core media through PUT /api/admin/skills/knots/:knot_id/media/:asset_id.

Options:
  --api-base-url <url>   API base URL (default: STELLARTRAIL_API_BASE_URL or ${DEFAULT_API_BASE_URL})
  --token <token>        Admin bearer token (default: STELLARTRAIL_ADMIN_TOKEN)
  --metadata <path>      Knots3D metadata JSON (default: ${DEFAULT_METADATA})
  --media-root <path>    Local Knots3D media root (default: parent of metadata/)
  --concurrency <n>      Parallel uploads, 1..16 (default: 4)
  --limit <n>            Upload only first n planned media items for smoke tests
  --dry-run              Print upload plan summary without HTTP writes
  --verify-only          Skip upload and verify public read API completeness
`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
