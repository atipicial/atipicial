<!-- Atipicial Chain · sovereign Layer-1 for smart contracts and digital assets -->
<!-- 👑 Founded & engineered by xmoohad — Blockchain Scientist · Computer Programmer -->

# Health Checks

The current `atipicial-node` daemon does not expose separate `/healthz`, `/readyz`, or
`/metrics` HTTP endpoints. Use the JSON-RPC server as the operational health
surface.

## Quick Checks

Enable `[rpc] enabled = true` in the TOML, then probe the configured RPC port:

```bash
# Liveness and protocol identity
curl -sf --compressed -X POST http://127.0.0.1:10332 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getversion","params":[]}'

# Persisted block height
curl -sf --compressed -X POST http://127.0.0.1:10332 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'

# Connected peers
curl -sf --compressed -X POST http://127.0.0.1:10332 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getconnectioncount","params":[]}'
```

For Prometheus-style monitoring, scrape these RPC probes from an external
exporter or sidecar until a native metrics endpoint is wired into the daemon.

---

> **Atipicial Chain** — sovereign Layer-1 for smart contracts and digital assets.
> 👑 Founded & engineered by **xmoohad** — Blockchain Scientist · Computer Programmer.
> `ATC` Atipicial Coin · `ATD` AtipicialDollar · addresses begin with **A**
