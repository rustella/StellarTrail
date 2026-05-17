#!/usr/bin/env bash
# Run the production API compose stack with config.yaml as the single config source.
#
# Docker Compose still needs a few environment variables for PostgreSQL, Redis,
# and MinIO containers. This wrapper derives those values from the same
# config.yaml mounted into the API container, so production no longer needs an
# infra/production/api/.env file.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="${STELLARTAIL_DEPLOY_ROOT:-/www/service/stellartail}"
CONFIG_FILE="${CONFIG_PATH:-${DEPLOY_ROOT}/config.yaml}"
PYTHON_BIN="${PYTHON_BIN:-python3}"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "config.yaml not found: $CONFIG_FILE" >&2
  echo "Set CONFIG_PATH=/path/to/config.yaml or STELLARTAIL_DEPLOY_ROOT=/deploy/root." >&2
  exit 1
fi

while IFS= read -r assignment; do
  export "$assignment"
done < <("$PYTHON_BIN" - "$CONFIG_FILE" <<'PY'
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")


def strip_comment(value: str) -> str:
    quote = None
    escaped = False
    out = []
    for ch in value:
        if quote:
            out.append(ch)
            if escaped:
                escaped = False
            elif ch == "\\" and quote == '"':
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in ("'", '"'):
            quote = ch
            out.append(ch)
            continue
        if ch == "#":
            break
        out.append(ch)
    return "".join(out).strip()


def parse_scalar(value: str) -> str:
    value = strip_comment(value).strip()
    if value in ("", "null", "~"):
        return ""
    if (value.startswith('"') and value.endswith('"')) or (
        value.startswith("'") and value.endswith("'")
    ):
        value = value[1:-1]
    return value.replace("''", "'")


def parse_simple_yaml(source: str):
    data = {}
    section = None
    for raw in source.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        line = strip_comment(raw.strip())
        if not line:
            continue
        if indent == 0 and line.endswith(":"):
            section = line[:-1].strip()
            data.setdefault(section, {})
            continue
        if indent == 2 and section and ":" in line:
            key, value = line.split(":", 1)
            data.setdefault(section, {})[key.strip()] = parse_scalar(value)
    return data


try:
    import yaml  # type: ignore
except ImportError:
    config = parse_simple_yaml(text)
else:
    loaded = yaml.safe_load(text) or {}
    config = {
        key: value
        for key, value in loaded.items()
        if isinstance(value, dict)
    }


def get(section: str, key: str, default: str = "") -> str:
    value = config.get(section, {}).get(key, default)
    if value is None:
        return ""
    return str(value).strip()


def require(section: str, key: str) -> str:
    value = get(section, key)
    if not value:
        raise SystemExit(f"config.yaml missing required value: {section}.{key}")
    return value


def emit(key: str, value: str) -> None:
    if "\n" in value or "\0" in value:
        raise SystemExit(f"unsupported newline/NUL in {key}")
    print(f"{key}={value}")


database_url = require("database", "url")
db = urlparse(database_url)
if db.scheme not in {"postgres", "postgresql"}:
    raise SystemExit("production database.url must use postgres:// or postgresql://")
postgres_user = unquote(db.username or "")
postgres_password = unquote(db.password or "")
postgres_db = unquote(db.path.lstrip("/").split("?", 1)[0] or "")
if not postgres_user or not postgres_password or not postgres_db:
    raise SystemExit("database.url must include user, password, and database name")
emit("POSTGRES_USER", postgres_user)
emit("POSTGRES_PASSWORD", postgres_password)
emit("POSTGRES_DB", postgres_db)

redis_url = get("redis", "url")
redis_password = ""
if redis_url:
    redis = urlparse(redis_url)
    redis_password = unquote(redis.password or "")
emit("REDIS_PASSWORD", redis_password)

emit("MINIO_ROOT_USER", require("minio", "access_key_id"))
emit("MINIO_ROOT_PASSWORD", require("minio", "secret_access_key"))
emit("OBJECT_STORAGE_BUCKET", get("object_storage", "bucket", "stellartrail-uploads"))
emit("KNOTS_MEDIA_BUCKET", get("knots_media_storage", "bucket", "stellartrail-knots-media"))
PY
)

export STELLARTAIL_DEPLOY_ROOT="$DEPLOY_ROOT"

if [[ "${1:-}" == "--print-derived-env" ]]; then
  for key in \
    STELLARTAIL_DEPLOY_ROOT \
    POSTGRES_USER \
    POSTGRES_PASSWORD \
    POSTGRES_DB \
    REDIS_PASSWORD \
    MINIO_ROOT_USER \
    MINIO_ROOT_PASSWORD \
    OBJECT_STORAGE_BUCKET \
    KNOTS_MEDIA_BUCKET; do
    value="${!key:-}"
    case "$key" in
      *PASSWORD*)
        if [[ -n "$value" ]]; then
          value="<set>"
        else
          value="<empty>"
        fi
        ;;
    esac
    printf '%s=%s\n' "$key" "$value"
  done
  exit 0
fi

if [[ $# -eq 0 ]]; then
  set -- up -d
fi

exec docker compose -f "$SCRIPT_DIR/docker-compose.yml" "$@"
