#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:${API_HOST_PORT:-18080}}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

SMOKE_ID="${SMOKE_ID:-$(python3 -c 'import secrets; print(secrets.token_hex(4))')}"
USERNAME="compose_${SMOKE_ID}"
EMAIL="${USERNAME}@example.test"
ACCOUNT_PASS="${ACCOUNT_PASS:-OutdoorPass123!}"

echo "[smoke] healthz"
curl -fsS "$BASE_URL/healthz" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["status"] == "ok", data; print(data)'

echo "[smoke] meta"
curl -fsS "$BASE_URL/api/meta" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["name"] == "StellarTrail", data; assert data["env"] == "local", data; assert data["database_kind"] == "postgres", data; print(data)'

echo "[smoke] username/password account registration and login"
EMAIL_CODE_REQUEST="$TMP_DIR/email-code-request.json"
EMAIL_CODE_JSON="$TMP_DIR/email-code.json"
python3 - "$EMAIL" > "$EMAIL_CODE_REQUEST" <<'PY'
import json
import sys

json.dump({"email": sys.argv[1]}, sys.stdout)
PY
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$EMAIL_CODE_REQUEST" \
  "$BASE_URL/api/auth/email-verification-code" > "$EMAIL_CODE_JSON"
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
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$REGISTER_REQUEST" \
  "$BASE_URL/api/auth/register" > "$REGISTER_JSON"
python3 - "$REGISTER_JSON" "$USERNAME" "$EMAIL" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
assert data.get("access_token"), data
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
curl -fsS \
  -H 'content-type: application/json' \
  -d "@$LOGIN_REQUEST" \
  "$BASE_URL/api/auth/login" > "$LOGIN_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("user",{}).get("username") == sys.argv[2], data; print(data["access_token"])' "$LOGIN_JSON" "$USERNAME")"
AUTH_HEADER="Authorization: Bearer $TOKEN"

echo "[smoke] gear categories through Redis-backed cache"
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/me/gears/categories" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert isinstance(data.get("items"), list), data; print({"items": len(data["items"])})'
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/me/gears/categories" >/dev/null

echo "[smoke] create gear and verify list/stats"
GEAR_JSON="$TMP_DIR/gear.json"
curl -fsS \
  -H "$AUTH_HEADER" \
  -H 'content-type: application/json' \
  -d '{"category":"lighting_system","name":"Compose Smoke Headlamp","brand":"StellarTrail","model":"IT-1","weight_g":88,"purchase_date":"2026-05-16","purchase_price_cents":9900,"status":"available","storage_location":"compose","tags":["smoke","redis"]}' \
  "$BASE_URL/api/me/gears" > "$GEAR_JSON"
GEAR_ID="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("id"), data; assert data.get("name") == "Compose Smoke Headlamp", data; print(data["id"])' "$GEAR_JSON")"
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/me/gears?limit=5" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert any(item.get("name") == "Compose Smoke Headlamp" for item in data.get("items", [])), data; print({"items": len(data.get("items", []))})'
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/me/gears/stats" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data.get("current_count", 0) >= 1, data; print({"current_count": data["current_count"], "total_weight_g": data["total_weight_g"]})'
curl -fsS -H "$AUTH_HEADER" "$BASE_URL/api/me/gears/$GEAR_ID" >/dev/null

echo "[smoke] ok"
