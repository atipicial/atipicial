<!-- Atipicial Chain · sovereign Layer-1 for smart contracts and digital assets -->
<!-- 👑 Founded & engineered by xmoohad — Blockchain Scientist · Computer Programmer -->

# Metrics Conventions

This document records the naming conventions for the Prometheus-style metrics
that atipicial-rs exposes, and the two deliberate deviations that are frozen for
compatibility. It is descriptive: it documents the conventions the codebase
already follows, so new metrics stay consistent and existing scrape tooling
keeps working.

## Metric prefix ownership

Every metric name begins with a crate-scoped prefix. The prefix tells you which
crate owns and emits the metric.

| Prefix | Owning crate / subsystem | Examples |
|--------|--------------------------|----------|
| `atipicial_node_` | `atipicial-node` (daemon, indexer, mempool gauges, task supervision) | `atipicial_node_ledger_height`, `atipicial_node_daemon_task_spawned_total`, `atipicial_node_mempool_transactions` |
| `atipicial_sync_` | `atipicial-node` sync/persistence metrics (`atipicial-node/src/node/sync_metrics`) | `atipicial_sync_height`, `atipicial_sync_blocks_persisted`, `atipicial_sync_avg_commit_us` |
| `atipicial_state_service_` | `atipicial-state-service` (MPT apply pipeline) | `atipicial_state_service_mpt_apply_height`, `atipicial_state_service_mpt_apply_avg_total_us` |
| `atipicial_storage_mdbx_` | `atipicial-storage` MDBX backend (production default environment diagnostics) | `atipicial_storage_mdbx_map_size_bytes`, `atipicial_storage_mdbx_reader_slots_used` |
| `atipicial_rpc_` | `atipicial-rpc` (JSON-RPC request/error counters) | `atipicial_rpc_requests_total`, `atipicial_rpc_errors_total` |

When adding a metric, pick the prefix that matches the emitting crate/subsystem
and keep the rest of the name descriptive.

## Naming rules

- **Counters end in `_total`.** A monotonically increasing counter should carry
  the `_total` suffix (e.g. `atipicial_rpc_requests_total`,
  `atipicial_node_daemon_task_spawned_total`).

  Known deviation: `atipicial_sync_blocks_persisted` is declared `# TYPE ... counter`
  but does **not** carry the `_total` suffix. Do **not** rename it — see
  "Frozen names" below.

- **Gauges** carry no `_total` suffix (e.g. `atipicial_sync_height`,
  `atipicial_node_ledger_height`, `atipicial_storage_mdbx_map_size_bytes`).

- **Duration metrics use the `_us` suffix** (microseconds). This is the
  established convention across the sync and state-service metrics
  (e.g. `atipicial_sync_avg_commit_us`, `atipicial_sync_avg_verify_us`,
  `atipicial_state_service_mpt_apply_avg_total_us`).

  Note: the Prometheus base-unit convention would use `_seconds` instead. atipicial-rs
  deliberately keeps `_us` — see "Frozen names" below.

## Frozen names

Two conventions deviate from strict Prometheus base-unit / naming guidance and
are intentionally frozen, because `scripts/run-bounded-mainnet-replay.py` scrapes
metrics by exact name and matching a fixed list breaks if the names change:

- **`_us` duration suffix is not renamed to `_seconds`.** The replay script's
  `DEFAULT_METRIC_NAMES` list matches names such as `atipicial_sync_avg_total_us` and
  `atipicial_state_service_mpt_apply_avg_us` verbatim. Renaming to the Prometheus
  base-unit `_seconds` would break that scrape parsing, so `_us` is frozen.

- **`atipicial_sync_blocks_persisted` keeps its non-`_total` name.** The same replay
  script matches `atipicial_sync_blocks_persisted` by exact name. It is a known
  deviation from the counter naming rule, flagged here rather than renamed.

Before renaming any scraped metric, update
`scripts/run-bounded-mainnet-replay.py` (and any other consumers) in the same
change.

---

> **Atipicial Chain** — sovereign Layer-1 for smart contracts and digital assets.
> 👑 Founded & engineered by **xmoohad** — Blockchain Scientist · Computer Programmer.
> `ATC` Atipicial Coin · `ATD` AtipicialDollar · addresses begin with **A**
