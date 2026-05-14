#!/usr/bin/env bash
set -euo pipefail

CONTENT_DIR="${1:-content}"
echo "Content import placeholder. Validating directory exists: ${CONTENT_DIR}"
test -d "${CONTENT_DIR}"
echo "Next step: wire crates/importer into a CLI that writes to the selected database."
