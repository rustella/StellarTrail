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

detect_deploy_timezone() {
  local timezone=""
  if [[ -n "${TZ:-}" ]]; then
    printf '%s\n' "$TZ"
    return
  fi
  if command -v timedatectl >/dev/null 2>&1; then
    timezone="$(timedatectl show --property=Timezone --value 2>/dev/null || true)"
    if [[ -n "$timezone" ]]; then
      printf '%s\n' "$timezone"
      return
    fi
  fi
  if [[ -f /etc/timezone ]]; then
    timezone="$(head -n 1 /etc/timezone | tr -d '[:space:]')"
    if [[ -n "$timezone" ]]; then
      printf '%s\n' "$timezone"
      return
    fi
  fi
  if [[ -L /etc/localtime ]]; then
    timezone="$(readlink /etc/localtime)"
    case "$timezone" in
      *zoneinfo/*)
        printf '%s\n' "${timezone#*zoneinfo/}"
        return
        ;;
    esac
  fi
}

assignments="$("$PYTHON_BIN" - "$CONFIG_FILE" <<'PY'
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
avatar_bucket = get("avatar_storage", "bucket", "stellartrail-avatars")
emit("AVATAR_STORAGE_BUCKET", avatar_bucket)
emit(
    "AVATAR_STORAGE_PUBLIC_BASE_URL",
    get(
        "avatar_storage",
        "public_base_url",
        f"https://assets.example.invalid/{avatar_bucket}",
    ),
)
emit("AVATAR_STORAGE_MAX_IMAGE_BYTES", get("avatar_storage", "max_image_bytes", "2000000"))
emit("KNOTS_MEDIA_BUCKET", get("knots_media_storage", "bucket", "stellartrail-knots-media"))
PY
)"

while IFS= read -r assignment; do
  export "$assignment"
done <<< "$assignments"

export STELLARTAIL_DEPLOY_ROOT="$DEPLOY_ROOT"
export STELLARTAIL_DEPLOY_TIMEZONE="${STELLARTAIL_DEPLOY_TIMEZONE:-$(detect_deploy_timezone)}"
export APP_COMMIT_HASH="${APP_COMMIT_HASH:-$(git -C "$DEPLOY_ROOT" rev-parse HEAD 2>/dev/null || true)}"
export COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-stellartail-api}"

if [[ "${1:-}" == "--print-derived-env" ]]; then
  for key in \
    COMPOSE_PROJECT_NAME \
    STELLARTAIL_DEPLOY_ROOT \
    STELLARTAIL_DEPLOY_TIMEZONE \
    APP_COMMIT_HASH \
    POSTGRES_USER \
    POSTGRES_PASSWORD \
    POSTGRES_DB \
    REDIS_PASSWORD \
    MINIO_ROOT_USER \
    MINIO_ROOT_PASSWORD \
    OBJECT_STORAGE_BUCKET \
    AVATAR_STORAGE_BUCKET \
    AVATAR_STORAGE_PUBLIC_BASE_URL \
    AVATAR_STORAGE_MAX_IMAGE_BYTES \
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

command_requires_volume_pins() {
  local command=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --ansi|--compatibility|--env-file|--file|-f|--parallel|--profile|--progress|--project-directory|--project-name|-p)
        if [[ $# -gt 1 ]]; then
          shift 2
        else
          shift
        fi
        ;;
      --dry-run|--verbose)
        shift
        ;;
      -*)
        shift
        ;;
      *)
        command="$1"
        break
        ;;
    esac
  done

  case "$command" in
    up|create|start|run)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

validate_stateful_volume_pins() {
  local override_file="$1"
  if [[ "${STELLARTAIL_ALLOW_UNPINNED_PRODUCTION_VOLUMES:-}" == "1" ]]; then
    echo "warning: skipping production external volume pin validation" >&2
    return
  fi
  if [[ ! -f "$override_file" ]]; then
    cat >&2 <<EOF
Production stateful volume pins are required before running this compose command.
Create $override_file with external volume names for postgres-data, redis-data, and minio-data.
For a brand-new production bootstrap only, set STELLARTAIL_ALLOW_UNPINNED_PRODUCTION_VOLUMES=1.
EOF
    exit 1
  fi

  "$PYTHON_BIN" - "$override_file" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")

try:
    import yaml  # type: ignore
except ImportError:
    yaml = None


def parse_bool(value: str) -> bool:
    return value.strip().lower() in {"true", "yes", "on", "1"}


def parse_scalar(value: str) -> str:
    value = value.split("#", 1)[0].strip()
    if (value.startswith('"') and value.endswith('"')) or (
        value.startswith("'") and value.endswith("'")
    ):
        return value[1:-1]
    return value


def fallback_parse_volumes(source: str):
    volumes = {}
    in_volumes = False
    current = None
    for raw in source.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        line = raw.strip()
        if indent == 0:
            in_volumes = line == "volumes:"
            current = None
            continue
        if not in_volumes:
            continue
        if indent == 2 and line.endswith(":"):
            current = line[:-1].strip()
            volumes[current] = {}
            continue
        if indent == 4 and current and ":" in line:
            key, value = line.split(":", 1)
            key = key.strip()
            value = parse_scalar(value)
            volumes[current][key] = parse_bool(value) if key == "external" else value
    return volumes


if yaml is None:
    volumes = fallback_parse_volumes(text)
else:
    loaded = yaml.safe_load(text) or {}
    volumes = loaded.get("volumes") or {}

required = ("postgres-data", "redis-data", "minio-data")
missing = []
for volume in required:
    config = volumes.get(volume)
    if not isinstance(config, dict):
        missing.append(f"{volume}: missing")
        continue
    if config.get("external") is not True:
        missing.append(f"{volume}: external must be true")
    name = str(config.get("name") or "").strip()
    if not name:
        missing.append(f"{volume}: name is required")

if missing:
    details = "\n".join(f"- {item}" for item in missing)
    raise SystemExit(
        "Production stateful volume pins are incomplete:\n"
        f"{details}\n"
        "Pin postgres-data, redis-data, and minio-data in the server-local "
        "docker-compose.production-local.override.yml before deploying."
    )
PY
}

compose_files=(-f "$SCRIPT_DIR/docker-compose.yml")
production_local_override="$SCRIPT_DIR/docker-compose.production-local.override.yml"
if [[ -f "$SCRIPT_DIR/docker-compose.production-local.override.yml" ]]; then
  compose_files+=( -f "$production_local_override" )
elif [[ -f "$SCRIPT_DIR/docker-compose.backend-external.override.yml" ]]; then
  compose_files+=( -f "$SCRIPT_DIR/docker-compose.backend-external.override.yml" )
fi

if command_requires_volume_pins "$@"; then
  validate_stateful_volume_pins "$production_local_override"
fi

exec docker compose "${compose_files[@]}" "$@"
