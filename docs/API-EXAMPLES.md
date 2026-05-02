# Replay API — curl Examples

All examples assume `replay-api` is running on `localhost:8787`.

## Health / Version

```bash
curl http://localhost:8787/health
# ok

curl http://localhost:8787/version
# {"name":"replay-api","version":"0.1.0"}
```

## One-shot replay

```bash
curl -X POST http://localhost:8787/replay \
  -H 'content-type: application/json' \
  -d '{"signature":"<BASE58_SIG>"}'
```

Returns a `Trace` JSON with `frames`, `account_deltas`, `total_cu`, and `log_divergence`.

## Fork → Mutate → Execute → Diff

### 1. Fork

```bash
SESSION=$(curl -s -X POST http://localhost:8787/fork \
  -H 'content-type: application/json' \
  -d '{"signature":"<BASE58_SIG>"}' \
  | jq -r .session_id)
echo "session: $SESSION"
```

### 2. Mutate — lamports

```bash
curl -X POST "http://localhost:8787/session/$SESSION/mutate" \
  -H 'content-type: application/json' \
  -d '{
    "pubkey": "<ACCOUNT_PUBKEY>",
    "mutation": { "type": "lamports", "new_value": 1000000000 }
  }'
```

### 3. Mutate — raw bytes splice

```bash
curl -X POST "http://localhost:8787/session/$SESSION/mutate" \
  -H 'content-type: application/json' \
  -d '{
    "pubkey": "<ACCOUNT_PUBKEY>",
    "mutation": { "type": "raw_bytes", "offset": 8, "bytes": "0f270000", "extend": false }
  }'
```

### 4. Re-execute

```bash
curl -X POST "http://localhost:8787/session/$SESSION/execute"
```

Returns the new `Trace`.

### 5. Diff

```bash
curl "http://localhost:8787/session/$SESSION/diff"
```

Returns:

```json
{
  "result_changed": true,
  "total_cu_delta": 12345,
  "changed_accounts": ["<PUBKEY_A>", "<PUBKEY_B>"],
  "baseline": { ... },
  "latest": { ... }
}
```

## Error shape

All errors follow:

```json
{
  "error": {
    "code": "tx_not_found",
    "message": "transaction not found on-chain"
  }
}
```

HTTP status codes:
- `400 Bad Request` — invalid signature or malformed body
- `404 Not Found` — transaction or session not found
- `422 Unprocessable Entity` — state reconstruction failure, bad mutation path
- `502 Bad Gateway` — Helius RPC error
- `500 Internal Server Error` — unexpected engine error
- `429 Too Many Requests` — rate limit exceeded (20 req/min per IP)
