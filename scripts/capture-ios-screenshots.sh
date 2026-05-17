#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IOS_DIR="$ROOT_DIR/apps/ios"
STAMP="${STELLARTRAIL_SCREENSHOT_STAMP:-$(date +%Y-%m-%d_%H%M%S)}"
OUT_DIR="${STELLARTRAIL_SCREENSHOT_DIR:-$ROOT_DIR/.hermes/local/ios-screenshots/$STAMP}"
RESULT_BUNDLE="$OUT_DIR/StellarTrailScreenshots.xcresult"
DEVICE="${STELLARTRAIL_SCREENSHOT_DEVICE:-iPhone 15 Pro}"

mkdir -p "$OUT_DIR"
cd "$IOS_DIR"

if ! command -v xcodegen >/dev/null 2>&1; then
  echo "xcodegen is required. Install it on macOS before running screenshot capture." >&2
  exit 1
fi

xcodegen generate

xcodebuild \
  -project StellarTrail.xcodeproj \
  -scheme StellarTrail \
  -destination "platform=iOS Simulator,name=$DEVICE" \
  -resultBundlePath "$RESULT_BUNDLE" \
  -only-testing:StellarTrailUITests/ScreenshotFlowUITests \
  test

if xcrun xcresulttool export attachments --path "$RESULT_BUNDLE" --output-path "$OUT_DIR/attachments" >/dev/null 2>&1; then
  echo "Screenshot attachments exported to $OUT_DIR/attachments"
else
  echo "Screenshot result bundle saved to $RESULT_BUNDLE"
  echo "Open it in Xcode Organizer or export attachments with the xcresulttool version installed on this Mac."
fi
