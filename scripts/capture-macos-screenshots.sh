#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/apps/macos"
OUT_DIR="${1:-$ROOT/.hermes/local/macos-screenshots}"
DERIVED_DATA="$APP_DIR/DerivedData/Screenshots"
SCHEME="StellarTrailMac"
APP_NAME="StellarTrail"
BUNDLE_ID="com.rustella.stellartrail.macos"
PROJECT="$APP_DIR/StellarTrailMac.xcodeproj"
WINDOW_ORIGIN_X=340
WINDOW_ORIGIN_Y=140
WINDOW_WIDTH=1320
WINDOW_HEIGHT=860

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
APP_PATH="$DERIVED_DATA/Build/Products/Debug/$APP_NAME.app"

wait_for_window() {
  for _ in {1..60}; do
    if osascript -e "tell application \"System Events\" to tell (first application process whose bundle identifier is \"$BUNDLE_ID\") to count of windows" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done

  echo "Timed out waiting for $APP_NAME window." >&2
  exit 1
}

focus_and_frame_window() {
  osascript \
    -e "tell application id \"$BUNDLE_ID\" to activate" \
    -e "tell application \"System Events\"" \
    -e "  tell (first application process whose bundle identifier is \"$BUNDLE_ID\")" \
    -e "    set frontmost to true" \
    -e "    tell window 1" \
    -e "      set position to {$WINDOW_ORIGIN_X, $WINDOW_ORIGIN_Y}" \
    -e "      set size to {$WINDOW_WIDTH, $WINDOW_HEIGHT}" \
    -e "    end tell" \
    -e "  end tell" \
    -e "end tell"
}

capture_page() {
  local page="$1"
  local file="$2"
  local rect="$WINDOW_ORIGIN_X,$WINDOW_ORIGIN_Y,$WINDOW_WIDTH,$WINDOW_HEIGHT"

  pkill -x "$APP_NAME" >/dev/null 2>&1 || true
  sleep 0.5
  open -na "$APP_PATH" --args --stellartrail-screenshot-fixtures --stellartrail-screenshot-page "$page"
  wait_for_window
  focus_and_frame_window
  sleep 0.8
  screencapture -x -T 0 -R "$rect" "$OUT_DIR/$file"
}

capture_page home 01_home.png
capture_page gear 02_gear.png
capture_page skills 03_skills.png
capture_page profile 04_profile.png
capture_page auth-login 05_auth_login.png
capture_page auth-register 06_auth_register.png

pkill -x "$APP_NAME" >/dev/null 2>&1 || true
echo "Screenshots saved to $OUT_DIR"
