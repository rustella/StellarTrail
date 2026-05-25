#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/apps/macos"
OUT_DIR="${1:-$ROOT/.hermes/local/macos-screenshots}"
DERIVED_DATA="$APP_DIR/DerivedData/Screenshots"
SCHEME="StellarTrailMac"
PROJECT="$APP_DIR/StellarTrailMac.xcodeproj"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS screenshot capture must run on macOS." >&2
  exit 1
fi
for cmd in xcodegen xcodebuild screencapture osascript; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
done

mkdir -p "$OUT_DIR"
cd "$APP_DIR"
xcodegen generate
xcodebuild build -project "$PROJECT" -scheme "$SCHEME" -destination 'platform=macOS' -derivedDataPath "$DERIVED_DATA"
APP_PATH="$DERIVED_DATA/Build/Products/Debug/$SCHEME.app"

capture_page() {
  local page="$1"
  local file="$2"
  pkill -x "$SCHEME" >/dev/null 2>&1 || true
  open -na "$APP_PATH" --args --stellartrail-screenshot-fixtures --stellartrail-screenshot-page "$page"
  sleep 2
  osascript -e 'tell application "System Events" to set frontmost of first process whose name is "StellarTrailMac" to true' || true
  sleep 1
  screencapture -x -T 0 "$OUT_DIR/$file"
}

capture_page home 01_home.png
capture_page gear 02_gear.png
capture_page skills 03_skills.png
capture_page profile 04_profile.png
capture_page auth-login 05_auth_login.png
capture_page auth-register 06_auth_register.png

pkill -x "$SCHEME" >/dev/null 2>&1 || true
echo "Screenshots saved to $OUT_DIR"
