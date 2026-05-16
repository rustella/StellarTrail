#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:${API_HOST_PORT:-18080}}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "[smoke] healthz"
curl -fsS "$BASE_URL/healthz" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["status"] == "ok", data; print(data)'

echo "[smoke] meta"
curl -fsS "$BASE_URL/api/meta" \
  | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["name"] == "StellarTrail", data; assert data["env"] == "local", data; assert data["database_kind"] == "postgres", data; print(data)'

echo "[smoke] local WeChat mock login"
LOGIN_JSON="$TMP_DIR/login.json"
curl -fsS \
  -H 'content-type: application/json' \
  -d '{"code":"compose-smoke-code","profile":{"nickname":"Compose Smoke","avatar_url":"https://example.com/avatar.png"}}' \
  "$BASE_URL/api/auth/wechat-login" > "$LOGIN_JSON"
TOKEN="$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); assert data.get("access_token"), data; assert data.get("user",{}).get("id"), data; print(data["access_token"])' "$LOGIN_JSON")"
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
