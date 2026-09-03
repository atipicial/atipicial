<!-- Atipicial Chain · sovereign Layer-1 for smart contracts and digital assets -->
<!-- 👑 Founded & engineered by xmoohad — Blockchain Scientist · Computer Programmer -->

# Atipicial Node

Standalone Atipicial N3 blockchain node daemon with built-in RPC server.

## Overview

`atipicial-node` is a standalone daemon that runs the Atipicial N3 blockchain node. It:
- Synchronizes with the Atipicial network over the Atipicial P2P protocol
- Provides a JSON-RPC API for external clients
- Manages the blockchain database through MDBX
- Supports built-in services (RpcServer, AtipicialIndexer, ApplicationLogs, TokensTracker, StateService, OracleService when enabled)
- Consensus (dBFT 2.0) can be enabled via DBFTPlugin settings and a validator wallet
- Optional TEE support (SGX/Nitro) and HSM-backed consensus signing

## Installation

```bash
cargo build -p atipicial-node --release
```

## Usage

```bash
# Start with the default configuration (TestNet)
atipicial-node

# Start with MainNet configuration
atipicial-node --config atipicial_mainnet_node.toml

# Override storage path
atipicial-node --config atipicial_mainnet_node.toml --storage-path ./data/chain

# Validate configuration and storage without starting P2P/RPC
atipicial-node --config atipicial_mainnet_node.toml --check-all

# Assert the selected built-in chain's network magic
atipicial-node --config atipicial_mainnet_node.toml --network-magic 860833102
```

Notes:
- Storage backend, P2P, RPC, and consensus settings live in TOML.
- `--storage-path` uses the configured persistent backend, defaulting to MDBX in production builds, and overrides `[storage].data_dir` / `[storage].path`.
- Built-in MainNet and TestNet identity fields are assertions. Private/custom chains require an explicit validated `AtipicialChainSpec` from an embedding application.
- When dBFT is enabled, the validator key comes from the `[consensus]` configuration.

## Command-line Options

| Option | Description | Default |
|--------|-------------|--------|
| `-c, --config <PATH>` | Path to TOML configuration file | `atipicial_testnet_node.toml` |
| `--storage-path <PATH>` | Override storage path for the configured/default persistent backend | (from config) |
| `--network-magic <N>` | Assert the selected chain's network magic | (not set) |
| `--check-config` | Validate configuration and exit | false |
| `--check-storage` | Validate storage can be opened and exit | false |
| `--check-all` | Run all preflight checks and exit | false |

## Configuration

See `atipicial_mainnet_node.toml` for a full configuration example. Key sections:

```toml
[network]
network_type = "mainnet"  # or "testnet"

[p2p]
port = 10333
max_connections = 40
seed_nodes = ["seed1.atipicial.com:10333", "seed2.atipicial.com:10333"]

[storage]
data_dir = "./data/chain"
backend = "mdbx"
static_files_dir = "./data/chain-static"
static_files_max_segment_mb = 4096

[rpc]
enabled = true
bind_address = "127.0.0.1"
port = 10332

[indexer]
enabled = true
store_path = "./data/mainnet/indexer"

[application_logs]
enabled = true
path = "ApplicationLogs_{0}"

[tokens_tracker]
enabled = true
db_path = "TokensTracker_{0}"
enabled_trackers = ["AEP-17", "AEP-11"]

[telemetry.metrics]
enabled = true
bind_address = "127.0.0.1"
port = 9090
path = "/metrics"

[observability]
enabled = false
service_name = "atipicial-node-mainnet"
environment = "production"

[logging]
active = true
level = "info"
console_output = true
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      atipicial-node (L7)                       │
│  ┌─────────────────────────────────────────────────────┐│
│  │              atipicial-system (L5 Composition)             ││
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    ││
│  │  │ Blockchain │  │ LocalNode  │  │ Supervisor │    ││
│  │  │  Service   │  │  (P2P)     │  │ (Tasks)    │    ││
│  │  └────────────┘  └────────────┘  └────────────┘    ││
│  └─────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────┐│
│  │           atipicial-rpc + atipicial-oracle-service (L6)         ││
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    ││
│  │  │ RpcServer  │  │ AtipicialIndexer │  │ AppLogs    │    ││
│  │  └────────────┘  └────────────┘  └────────────┘    ││
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    ││
│  │  │TokenTrack  │  │StateRoot   │  │ Oracle     │    ││
│  │  └────────────┘  └────────────┘  └────────────┘    ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
                           │
                           │ JSON-RPC
                           ▼
                    ┌──────────────────┐
                    │ JSON-RPC clients │
                    └──────────────────┘
```

The node follows 8 ordered dependency layers and concrete static composition,
using selected **reth** provider/storage patterns and **Polkadot/Substrate**
bounded-context ideas as architecture references. The earlier type-state
`NodeComponents` and `EngineApi` scaffolds were removed. See `../design.md` for
the 45 ADRs and architecture evolution record.

## License

MIT License - see LICENSE file in the repository root.

---

> **Atipicial Chain** — sovereign Layer-1 for smart contracts and digital assets.
> 👑 Founded & engineered by **xmoohad** — Blockchain Scientist · Computer Programmer.
> `ATC` Atipicial Coin · `ATD` AtipicialDollar · addresses begin with **A**
