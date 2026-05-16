#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

export COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-stellartrail_it}"
export API_HOST_PORT="${API_HOST_PORT:-18080}"
export POSTGRES_HOST_PORT="${POSTGRES_HOST_PORT:-15432}"
export REDIS_HOST_PORT="${REDIS_HOST_PORT:-16379}"
export BASE_URL="${BASE_URL:-http://127.0.0.1:${API_HOST_PORT}}"

if docker info >/dev/null 2>&1; then
  DOCKER_COMPOSE=(docker compose)
elif sudo -n docker info >/dev/null 2>&1; then
  DOCKER_COMPOSE=(
    sudo -n env
    "COMPOSE_PROJECT_NAME=$COMPOSE_PROJECT_NAME"
    "API_HOST_PORT=$API_HOST_PORT"
    "POSTGRES_HOST_PORT=$POSTGRES_HOST_PORT"
    "REDIS_HOST_PORT=$REDIS_HOST_PORT"
    docker compose
  )
else
  echo "[integration] error: Docker is not accessible. Add the user to the docker group or allow passwordless sudo for docker." >&2
  exit 1
fi

cleanup() {
  local status=$?

  echo "[integration] shutting down compose stack"
  if ! "${DOCKER_COMPOSE[@]}" -f "$COMPOSE_FILE" down -v --remove-orphans; then
    echo "[integration] warning: compose cleanup failed" >&2
    if [ "$status" -eq 0 ]; then
      status=1
    fi
  fi

  exit "$status"
}
trap cleanup EXIT

echo "[integration] resetting compose stack"
"${DOCKER_COMPOSE[@]}" -f "$COMPOSE_FILE" down -v --remove-orphans

echo "[integration] starting PostgreSQL + Redis + API"
"${DOCKER_COMPOSE[@]}" -f "$COMPOSE_FILE" up -d --build postgres redis api

echo "[integration] waiting for API health at $BASE_URL/healthz"
for attempt in $(seq 1 60); do
  if curl -fsS "$BASE_URL/healthz" >/dev/null 2>&1; then
    break
  fi

  if [ "$attempt" -eq 60 ]; then
    echo "[integration] API did not become healthy" >&2
    "${DOCKER_COMPOSE[@]}" -f "$COMPOSE_FILE" ps >&2 || true
    "${DOCKER_COMPOSE[@]}" -f "$COMPOSE_FILE" logs --tail=120 api >&2 || true
    exit 1
  fi

  sleep 2
done

echo "[integration] running curl smoke test"
bash "$SCRIPT_DIR/smoke-test.sh"
