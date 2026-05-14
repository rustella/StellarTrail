#!/usr/bin/env bash
set -euo pipefail

export APP_ENV="${APP_ENV:-local}"
export APP_HOST="${APP_HOST:-127.0.0.1}"
export APP_PORT="${APP_PORT:-8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite://stellartrail.db}"
export RUST_LOG="${RUST_LOG:-stellartrail_api=debug,tower_http=debug,info}"

cargo run -p stellartrail-api
