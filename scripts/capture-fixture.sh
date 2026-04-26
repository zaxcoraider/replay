#!/usr/bin/env bash
# Capture a real Helius getTransaction response for use as a test fixture.
#
# Usage:
#   HELIUS_API_KEY=... ./scripts/capture-fixture.sh <signature> [output-path]
#
# Default output:
#   crates/replay-core/tests/fixtures/jupiter-swap-response.json
#
# After capture, also write the signature into demo-signature.txt so the
# live test can find it.

set -euo pipefail

if [[ -z "${HELIUS_API_KEY:-}" ]]; then
    echo "error: HELIUS_API_KEY is not set" >&2
    exit 1
fi

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <signature> [output-path]" >&2
    exit 2
fi

SIG="$1"
OUT="${2:-crates/replay-core/tests/fixtures/jupiter-swap-response.json}"

mkdir -p "$(dirname "$OUT")"

curl -sS -X POST "https://mainnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}" \
    -H 'Content-Type: application/json' \
    -d "$(cat <<EOF
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTransaction",
  "params": [
    "${SIG}",
    { "encoding": "base64", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }
  ]
}
EOF
)" | python3 -m json.tool > "$OUT"

echo "$SIG" > "$(dirname "$OUT")/demo-signature.txt"

echo "Captured: $OUT"
echo "Wrote signature to: $(dirname "$OUT")/demo-signature.txt"
