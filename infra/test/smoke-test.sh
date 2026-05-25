#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:18080}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

RUN_ID="$(date +%s)-$$"
USERNAME="${ACCOUNT_USERNAME:-smoke_$RUN_ID}"
EMAIL="${ACCOUNT_EMAIL:-smoke_$RUN_ID@example.test}"
ACCOUNT_PASS="${ACCOUNT_PASS:-OutdoorPass123!}"
NEW_ACCOUNT_PASS="${NEW_ACCOUNT_PASS:-OutdoorPass456!}"

expect_status() {
  local expected="$1"
  local actual="$2"
  local body_file="$3"
  if [ "$actual" != "$expected" ]; then
    echo "[smoke] expected HTTP $expected, got $actual" >&2
    cat "$body_file" >&2 || true
    exit 1
  fi
}

echo "[smoke] healthz"
curl -fsS "$BASE_URL/healthz"   | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["status"] == "ok", data; print(data)'

echo "[smoke] meta"
curl -fsS "$BASE_URL/api/v1/meta"   | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["name"] == "StellarTrail", data; assert data["env"] == "local", data; assert data["database_kind"] == "postgres", data; print(data)'


echo "[smoke] public skills and knots are readable without login"
SKILLS_HEADERS="$TMP_DIR/public-skills.headers"
SKILLS_JSON="$TMP_DIR/public-skills.json"
curl -fsS -D "$SKILLS_HEADERS" -H 'X-StellarTrail-Locale: zh-CN' "$BASE_URL/api/v1/skills" -o "$SKILLS_JSON"
python3 - "$SKILLS_HEADERS" "$SKILLS_JSON" <<'PY'
import json
import sys
headers = open(sys.argv[1]).read().lower()
data = json.load(open(sys.argv[2]))
assert any(item.get("id") == "knots" for item in data.get("items", [])), data
assert "content-language: zh-cn" in headers, headers
assert "cache-control:" in headers, headers
assert "etag:" in headers, headers
print({"public_skill_categories": len(data["items"])})
PY

KNOTS_HEADERS="$TMP_DIR/public-knots.headers"
KNOTS_JSON="$TMP_DIR/public-knots.json"
curl -fsS -D "$KNOTS_HEADERS" -H 'X-StellarTrail-Locale: zh-CN' "$BASE_URL/api/v1/skills/knots/list?offset=0&limit=20" -o "$KNOTS_JSON"
KNOT_ID="$(python3 - "$KNOTS_JSON" <<'PY'
import json
import sys
data = json.load(open(sys.argv[1]))
assert data.get("locale") == "zh-CN", data
items = data.get("items", [])
assert all("source_slug_en" not in item and "source_slug_zh" not in item for item in items), data
print(items[0]["id"] if items else "")
PY
)"
KNOT_ETAG="$(python3 - "$KNOTS_HEADERS" <<'PY'
import sys
for line in open(sys.argv[1]):
    if line.lower().startswith("etag:"):
        print(line.split(":", 1)[1].strip())
        break
PY
)"
if [ -n "$KNOT_ID" ]; then
  DETAIL_JSON="$TMP_DIR/public-knot-detail.json"
  curl -fsS -H 'X-StellarTrail-Locale: zh-CN' "$BASE_URL/api/v1/skills/knots/detail/$KNOT_ID" -o "$DETAIL_JSON"
  python3 - "$DETAIL_JSON" "$KNOT_ID" <<'PY'
import json
import sys
data = json.load(open(sys.argv[1]))
assert data.get("id") == sys.argv[2], data
assert data.get("title"), data
assert "source_slug_en" not in data and "source_slug_zh" not in data, data
print({"public_knot_detail": data["id"]})
PY
else
  echo "[smoke] no seeded public knots; skipping knot detail smoke"
fi
if [ -n "$KNOT_ETAG" ]; then
  NOT_MODIFIED_STATUS="$(curl -sS -o /dev/null -w '%{http_code}' -H "If-None-Match: $KNOT_ETAG" "$BASE_URL/api/v1/skills/knots/list?offset=0&limit=20")"
  expect_status 304 "$NOT_MODIFIED_STATUS" "$KNOTS_JSON"
fi
LEGACY_BODY="$TMP_DIR/public-skills-legacy.json"
LEGACY_STATUS="$(curl -sS -o "$LEGACY_BODY" -w '%{http_code}' "$BASE_URL/api/v1/skills?category=knot")"
expect_status 404 "$LEGACY_STATUS" "$LEGACY_BODY"
OLD_PREFIX_BODY="$TMP_DIR/old-api-prefix.json"
OLD_PREFIX_STATUS="$(curl -sS -o "$OLD_PREFIX_BODY" -w '%{http_code}' "$BASE_URL/api/skills")"
expect_status 404 "$OLD_PREFIX_STATUS" "$OLD_PREFIX_BODY"

echo "[smoke] username/password account registration and login"
EMAIL_CODE_REQUEST="$TMP_DIR/email-code-request.json"
EMAIL_CODE_JSON="$TMP_DIR/email-code.json"
python3 - "$EMAIL" > "$EMAIL_CODE_REQUEST" <<'PY'
import json
import sys

json.dump({"email": sys.argv[1]}, sys.stdout)
PY
curl -fsS   -H 'content-type: application/json'   -d "@$EMAIL_CODE_REQUEST"   "$BASE_URL/api/v1/auth/email-verification-code" > "$EMAIL_CODE_JSON"
VERIFY_CODE="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); code=data.get("debug_code"); assert code, data; print(code)' "$EMAIL_CODE_JSON")"

REGISTER_REQUEST="$TMP_DIR/register-request.json"
REGISTER_JSON="$TMP_DIR/register.json"
python3 - "$USERNAME" "$EMAIL" "$ACCOUNT_PASS" "$VERIFY_CODE" > "$REGISTER_REQUEST" <<'PY'
import json
import sys

username, email, password, code = sys.argv[1:5]
json.dump(
    {
        "username": username,
        "email": email,
        "password": password,
        "confirm_password": password,
        "email_verification_code": code,
    },
    sys.stdout,
)
PY
curl -fsS   -H 'content-type: application/json'   -d "@$REGISTER_REQUEST"   "$BASE_URL/api/v1/auth/register" > "$REGISTER_JSON"
python3 - "$REGISTER_JSON" "$USERNAME" "$EMAIL" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
assert data.get("access_token"), data
assert data.get("refresh_token"), data
assert data.get("refresh_expires_at"), data
user = data.get("user", {})
assert user.get("username") == sys.argv[2], data
assert user.get("email") == sys.argv[3], data
print({"registered_username": user["username"]})
PY

LOGIN_REQUEST="$TMP_DIR/login-request.json"
LOGIN_JSON="$TMP_DIR/login.json"
python3 - "$USERNAME" "$ACCOUNT_PASS" > "$LOGIN_REQUEST" <<'PY'
import json
import sys

account, password = sys.argv[1:3]
json.dump({"account": account, "password": password}, sys.stdout)
PY
curl -fsS   -H 'content-type: application/json'   -d "@$LOGIN_REQUEST"   "$BASE_URL/api/v1/auth/login" > "$LOGIN_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("refresh_token"), data; assert data.get("refresh_expires_at"), data; assert data.get("user",{}).get("username") == sys.argv[2], data; print(data["access_token"])' "$LOGIN_JSON" "$USERNAME")"
REFRESH_TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["refresh_token"])' "$LOGIN_JSON")"

REFRESH_REQUEST="$TMP_DIR/refresh-request.json"
REFRESH_JSON="$TMP_DIR/refresh.json"
python3 - "$REFRESH_TOKEN" > "$REFRESH_REQUEST" <<'PY'
import json
import sys

json.dump({"refresh_token": sys.argv[1]}, sys.stdout)
PY
curl -fsS   -H 'content-type: application/json'   -d "@$REFRESH_REQUEST"   "$BASE_URL/api/v1/auth/refresh" > "$REFRESH_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("refresh_token"), data; assert data["refresh_token"] != sys.argv[2], data; assert data.get("user",{}).get("username") == sys.argv[3], data; print(data["access_token"])' "$REFRESH_JSON" "$REFRESH_TOKEN" "$USERNAME")"
REFRESH_REPLAY_BODY="$TMP_DIR/refresh-replay.json"
REFRESH_REPLAY_STATUS="$(curl -sS -o "$REFRESH_REPLAY_BODY" -w '%{http_code}'   -H 'content-type: application/json'   -d "@$REFRESH_REQUEST"   "$BASE_URL/api/v1/auth/refresh")"
expect_status 401 "$REFRESH_REPLAY_STATUS" "$REFRESH_REPLAY_BODY"
AUTH_HEADER="authorization: Bearer $TOKEN"

echo "[smoke] email-code login"
EMAIL_LOGIN_CODE_REQUEST="$TMP_DIR/email-login-code-request.json"
EMAIL_LOGIN_CODE_JSON="$TMP_DIR/email-login-code.json"
python3 - "$EMAIL" > "$EMAIL_LOGIN_CODE_REQUEST" <<'PY'
import json
import sys

json.dump({"email": sys.argv[1]}, sys.stdout)
PY
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$EMAIL_LOGIN_CODE_REQUEST" \
  "$BASE_URL/api/v1/auth/email-login-code" > "$EMAIL_LOGIN_CODE_JSON"
EMAIL_LOGIN_CODE="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); code=data.get("debug_code"); assert code, data; print(code)' "$EMAIL_LOGIN_CODE_JSON")"

EMAIL_LOGIN_REQUEST="$TMP_DIR/email-login-request.json"
EMAIL_LOGIN_JSON="$TMP_DIR/email-login.json"
python3 - "$EMAIL" "$EMAIL_LOGIN_CODE" > "$EMAIL_LOGIN_REQUEST" <<'PY'
import json
import sys

email, code = sys.argv[1:3]
json.dump({"email": email, "email_verification_code": code}, sys.stdout)
PY
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$EMAIL_LOGIN_REQUEST" \
  "$BASE_URL/api/v1/auth/email-login" > "$EMAIL_LOGIN_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("refresh_token"), data; assert data.get("user",{}).get("email") == sys.argv[2], data; print(data["access_token"])' "$EMAIL_LOGIN_JSON" "$EMAIL")"
REFRESH_TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["refresh_token"])' "$EMAIL_LOGIN_JSON")"

EMAIL_LOGIN_REUSE_BODY="$TMP_DIR/email-login-reuse.json"
EMAIL_LOGIN_REUSE_STATUS="$(curl -sS -o "$EMAIL_LOGIN_REUSE_BODY" -w '%{http_code}' \
  -H 'content-type: application/json' \
  -d "@$EMAIL_LOGIN_REQUEST" \
  "$BASE_URL/api/v1/auth/email-login")"
expect_status 401 "$EMAIL_LOGIN_REUSE_STATUS" "$EMAIL_LOGIN_REUSE_BODY"

echo "[smoke] password reset invalidates old password and old refresh token"
OLD_REFRESH_TOKEN="$REFRESH_TOKEN"
PASSWORD_RESET_CODE_REQUEST="$TMP_DIR/password-reset-code-request.json"
PASSWORD_RESET_CODE_JSON="$TMP_DIR/password-reset-code.json"
python3 - "$EMAIL" > "$PASSWORD_RESET_CODE_REQUEST" <<'PY'
import json
import sys

json.dump({"email": sys.argv[1]}, sys.stdout)
PY
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$PASSWORD_RESET_CODE_REQUEST" \
  "$BASE_URL/api/v1/auth/password-reset-code" > "$PASSWORD_RESET_CODE_JSON"
PASSWORD_RESET_CODE="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); code=data.get("debug_code"); assert code, data; print(code)' "$PASSWORD_RESET_CODE_JSON")"

PASSWORD_RESET_REQUEST="$TMP_DIR/password-reset-request.json"
PASSWORD_RESET_JSON="$TMP_DIR/password-reset.json"
python3 - "$EMAIL" "$NEW_ACCOUNT_PASS" "$PASSWORD_RESET_CODE" > "$PASSWORD_RESET_REQUEST" <<'PY'
import json
import sys

email, password, code = sys.argv[1:4]
json.dump(
    {
        "email": email,
        "password": password,
        "confirm_password": password,
        "email_verification_code": code,
    },
    sys.stdout,
)
PY
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$PASSWORD_RESET_REQUEST" \
  "$BASE_URL/api/v1/auth/password-reset" > "$PASSWORD_RESET_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("refresh_token"), data; assert data.get("user",{}).get("email") == sys.argv[2], data; print(data["access_token"])' "$PASSWORD_RESET_JSON" "$EMAIL")"
REFRESH_TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["refresh_token"])' "$PASSWORD_RESET_JSON")"
AUTH_HEADER="Authorization: Bearer $TOKEN"

OLD_REFRESH_REQUEST="$TMP_DIR/old-refresh-request.json"
OLD_REFRESH_BODY="$TMP_DIR/old-refresh.json"
python3 - "$OLD_REFRESH_TOKEN" > "$OLD_REFRESH_REQUEST" <<'PY'
import json
import sys

json.dump({"refresh_token": sys.argv[1]}, sys.stdout)
PY
OLD_REFRESH_STATUS="$(curl -sS -o "$OLD_REFRESH_BODY" -w '%{http_code}' \
  -H 'content-type: application/json' \
  -d "@$OLD_REFRESH_REQUEST" \
  "$BASE_URL/api/v1/auth/refresh")"
expect_status 401 "$OLD_REFRESH_STATUS" "$OLD_REFRESH_BODY"

OLD_PASSWORD_REQUEST="$TMP_DIR/old-password-login-request.json"
OLD_PASSWORD_BODY="$TMP_DIR/old-password-login.json"
python3 - "$USERNAME" "$ACCOUNT_PASS" > "$OLD_PASSWORD_REQUEST" <<'PY'
import json
import sys

account, password = sys.argv[1:3]
json.dump({"account": account, "password": password}, sys.stdout)
PY
OLD_PASSWORD_STATUS="$(curl -sS -o "$OLD_PASSWORD_BODY" -w '%{http_code}' \
  -H 'content-type: application/json' \
  -d "@$OLD_PASSWORD_REQUEST" \
  "$BASE_URL/api/v1/auth/login")"
expect_status 401 "$OLD_PASSWORD_STATUS" "$OLD_PASSWORD_BODY"

NEW_PASSWORD_REQUEST="$TMP_DIR/new-password-login-request.json"
NEW_PASSWORD_JSON="$TMP_DIR/new-password-login.json"
python3 - "$EMAIL" "$NEW_ACCOUNT_PASS" > "$NEW_PASSWORD_REQUEST" <<'PY'
import json
import sys

account, password = sys.argv[1:3]
json.dump({"account": account, "password": password}, sys.stdout)
PY
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$NEW_PASSWORD_REQUEST" \
  "$BASE_URL/api/v1/auth/login" > "$NEW_PASSWORD_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("refresh_token"), data; assert data.get("user",{}).get("email") == sys.argv[2], data; print(data["access_token"])' "$NEW_PASSWORD_JSON" "$EMAIL")"
REFRESH_TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["refresh_token"])' "$NEW_PASSWORD_JSON")"
AUTH_HEADER="Authorization: Bearer $TOKEN"


echo "[smoke] gear categories through Redis-backed cache"
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/v1/me/gears/categories"   | python3 -c 'import json,sys; data=json.load(sys.stdin); assert isinstance(data.get("items"), list), data; print({"items": len(data["items"])})'
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/v1/me/gears/categories" >/dev/null

echo "[smoke] create gear and verify list/stats"
GEAR_JSON="$TMP_DIR/gear.json"
curl -fsS   -H "$AUTH_HEADER"   -H 'content-type: application/json'   -d '{"category":"lighting_system","name":"Compose Smoke Headlamp","brand":"StellarTrail","model":"IT-1","weight_g":88,"purchase_date":"2026-05-16","purchase_price_cents":9900,"status":"available","storage_location":"compose","tags":["smoke","redis"]}'   "$BASE_URL/api/v1/me/gears" > "$GEAR_JSON"
GEAR_ID="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("id"), data; assert data.get("name") == "Compose Smoke Headlamp", data; print(data["id"])' "$GEAR_JSON")"
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/v1/me/gears?limit=5"   | python3 -c 'import json,sys; data=json.load(sys.stdin); assert any(item.get("name") == "Compose Smoke Headlamp" for item in data.get("items", [])), data; print({"items": len(data.get("items", []))})'
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/v1/me/gears/stats"   | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data.get("current_count", 0) >= 1, data; print({"current_count": data["current_count"], "total_weight_g": data["total_weight_g"]})'
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/v1/me/gears/$GEAR_ID" >/dev/null

echo "[smoke] upload valid PNG feedback image to private MinIO bucket"
PNG_FILE="$TMP_DIR/screen.png"
python3 - "$PNG_FILE" <<'PY'
import base64
import sys

png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg=="
open(sys.argv[1], "wb").write(base64.b64decode(png))
PY
UPLOAD_JSON="$TMP_DIR/upload.json"
HTTP_CODE="$(curl -sS -o "$UPLOAD_JSON" -w '%{http_code}'   -H "$AUTH_HEADER"   -F 'purpose=feedback'   -F "file=@$PNG_FILE;type=image/png;filename=screen.png"   "$BASE_URL/api/v1/me/uploads")"
expect_status 201 "$HTTP_CODE" "$UPLOAD_JSON"
UPLOAD_ID="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("id"), data; assert data.get("content_type") == "image/png", data; assert data.get("download_url", "").startswith("/api/v1/me/uploads/"), data; print(data["id"])' "$UPLOAD_JSON")"

DOWNLOAD_FILE="$TMP_DIR/download.png"
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/v1/me/uploads/$UPLOAD_ID" -o "$DOWNLOAD_FILE"
python3 - "$PNG_FILE" "$DOWNLOAD_FILE" <<'PY'
import sys

expected = open(sys.argv[1], "rb").read()
actual = open(sys.argv[2], "rb").read()
assert actual == expected, {"expected": len(expected), "actual": len(actual)}
print({"downloaded_bytes": len(actual)})
PY
NOAUTH_BODY="$TMP_DIR/noauth-download.json"
NOAUTH_STATUS="$(curl -sS -o "$NOAUTH_BODY" -w '%{http_code}' "$BASE_URL/api/v1/me/uploads/$UPLOAD_ID")"
expect_status 401 "$NOAUTH_STATUS" "$NOAUTH_BODY"

echo "[smoke] reject spoofed image payloads without storing objects"
FAKE_FILE="$TMP_DIR/payload.jpg"
printf '<script>alert(1)</script>' > "$FAKE_FILE"
FAKE_BODY="$TMP_DIR/fake-upload.json"
FAKE_STATUS="$(curl -sS -o "$FAKE_BODY" -w '%{http_code}'   -H "$AUTH_HEADER"   -F 'purpose=feedback'   -F "file=@$FAKE_FILE;type=image/jpeg;filename=payload.jpg"   "$BASE_URL/api/v1/me/uploads")"
expect_status 422 "$FAKE_STATUS" "$FAKE_BODY"
python3 - "$FAKE_BODY" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
assert data.get("code") == "validation_failed", data
print({"fake_upload_rejected": data["code"]})
PY

LARGE_FILE="$TMP_DIR/large.png"
python3 - "$PNG_FILE" "$LARGE_FILE" <<'PY'
import sys

small = open(sys.argv[1], "rb").read()
payload = small + b"\0" * (8_000_001 - len(small))
open(sys.argv[2], "wb").write(payload)
PY
LARGE_BODY="$TMP_DIR/large-upload.json"
LARGE_STATUS="$(curl -sS -o "$LARGE_BODY" -w '%{http_code}'   -H "$AUTH_HEADER"   -F 'purpose=feedback'   -F "file=@$LARGE_FILE;type=image/png;filename=large.png"   "$BASE_URL/api/v1/me/uploads")"
expect_status 413 "$LARGE_STATUS" "$LARGE_BODY"
python3 - "$LARGE_BODY" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
assert data.get("code") == "payload_too_large", data
print({"large_upload_rejected": data["code"]})
PY

echo "[smoke] submit feedback with uploaded image"
FEEDBACK_REQUEST="$TMP_DIR/feedback-request.json"
FEEDBACK_JSON="$TMP_DIR/feedback.json"
python3 - "$UPLOAD_ID" > "$FEEDBACK_REQUEST" <<'PY'
import json
import sys

json.dump(
    {
        "category": "bug",
        "content": "Compose smoke feedback for upload flow",
        "contact": "smoke@example.test",
        "page": "/pages/routes/detail/index?id=smoke",
        "client_platform": "wechat_miniprogram",
        "client_version": "0.1.0",
        "device_model": "compose-smoke",
        "image_ids": [sys.argv[1]],
    },
    sys.stdout,
)
PY
HTTP_CODE="$(curl -sS -o "$FEEDBACK_JSON" -w '%{http_code}'   -H "$AUTH_HEADER"   -H 'content-type: application/json'   -d "@$FEEDBACK_REQUEST"   "$BASE_URL/api/v1/me/feedback")"
expect_status 201 "$HTTP_CODE" "$FEEDBACK_JSON"
python3 - "$FEEDBACK_JSON" "$UPLOAD_ID" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
assert data.get("status") == "open", data
images = data.get("images", [])
assert len(images) == 1 and images[0].get("id") == sys.argv[2], data
print({"feedback_id": data["id"], "image_count": len(images)})
PY

echo "[smoke] ok"
