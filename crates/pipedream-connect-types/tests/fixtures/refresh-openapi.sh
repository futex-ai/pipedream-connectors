#!/usr/bin/env bash
# Refresh the vendored schema from Pipedream. Run from any directory.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
curl --fail --location --output "$root/tests/fixtures/pipedream_openapi_swagger.json" \
  https://pipedream.com/docs/pipedream_openapi_swagger.json
