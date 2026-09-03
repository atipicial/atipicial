<!-- Atipicial Chain · sovereign Layer-1 for smart contracts and digital assets -->
<!-- 👑 Founded & engineered by xmoohad — Blockchain Scientist · Computer Programmer -->

# Atipicial-rs Deployment Guide

> **Version**: 0.7.0  
> **Last Updated**: 2026-01-28  
> **Target Compatibility**: Atipicial N3 v3.10.1

Comprehensive deployment documentation for the Atipicial N3 Rust node implementation.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build Instructions](#build-instructions)
- [Configuration](#configuration)
- [Docker Deployment](#docker-deployment)
- [Running a Node](#running-a-node)
- [Hardware Requirements](#hardware-requirements)
- [Upgrading](#upgrading)

---

## Prerequisites

### System Requirements

#### Supported Operating Systems

| OS | Version | Status |
|----|---------|--------|
| Ubuntu | 20.04 LTS, 22.04 LTS, 24.04 LTS | ✅ Fully supported |
| Debian | 11 (Bullseye), 12 (Bookworm) | ✅ Fully supported |
| CentOS/RHEL | 8, 9 | ✅ Supported |
| Alpine Linux | 3.18+ | ⚠️ Requires static linking |
| macOS | 13+ (Ventura) | ✅ Development only |
| Windows | 10/11, Server 2019+ | ⚠️ Community support |

### Dependencies

#### Required System Packages

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    gcc \
    g++ \
    cmake \
    make \
    pkg-config \
    librocksdb-dev \
    libssl-dev \
    clang \
    git \
    curl
```

**CentOS/RHEL:**
```bash
sudo yum install -y \
    gcc \
    gcc-c++ \
    cmake \
    make \
    pkgconfig \
    openssl-devel \
    clang \
    git \
    curl

# Install RocksDB from source or EPEL
sudo yum install -y epel-release
sudo yum install -y rocksdb-devel
```

#### Rust Toolchain

Minimum supported Rust version (MSRV): **1.89.0**

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be >= 1.89.0
```

#### Optional Dependencies

| Package | Purpose | Installation |
|---------|---------|--------------|
| `docker` | Container deployment | [Docker Docs](https://docs.docker.com/engine/install/) |
| `docker-compose` | Multi-container orchestration | Included with Docker Desktop |
| `systemd` | Service management | Pre-installed on most Linux distros |
| `prometheus` | Metrics collection | See [MONITORING.md](./MONITORING.md) |
| `libudev-dev` / `systemd-devel` | Ledger hardware wallet HSM support | Required when building with `hsm-ledger` on Linux |

---

## Build Instructions

### Release Build

For production deployments, always use the release profile:

```bash
# Clone the repository
git clone https://github.com/r3e-network/atipicial-rs.git
cd atipicial-rs

# Build the default node crate set in release mode
cargo build --release

# Binaries will be available at:
# - target/release/atipicial-node  (node daemon)
```

Use `cargo build --release --workspace` for explicit full-workspace validation,
including optional TEE/HSM, telemetry, integration test, and benchmark crates.

### Production Profile

For maximum performance, use the custom production profile:

```bash
# Build with production optimizations
cargo build --profile production -p atipicial-node

# Binaries will be at:
# - target/production/atipicial-node
```

The production profile enables:
- LTO (Link Time Optimization) with `fat` mode
- Single codegen unit for maximum optimization
- Panic abort strategy
- Binary stripping for smaller size

### Feature Flags

Optional features for specialized deployments:

| Feature | Description | Build Command |
|---------|-------------|---------------|
| `tee` | Trusted Execution Environment support | `--features tee` |
| `tee-sgx` | TEE with Intel SGX hardware | `--features tee-sgx` |
| `hsm` | Hardware Security Module support | `--features hsm` |
| `hsm-ledger` | HSM with Ledger hardware wallet | `--features hsm-ledger` |
| `hsm-pkcs11` | HSM with PKCS#11 interface | `--features hsm-pkcs11` |

TEE/HSM crates are present behind Cargo features for future integration work, but
the current `atipicial-node` binary does not expose runtime `--tee`, `--tee-auto`, or
HSM CLI flags. Build and operate the standard daemon with `cargo build --release
-p atipicial-node`, then configure node behavior through TOML and the supported CLI
flags shown by `atipicial-node --help`.

### Build Verification

```bash
# Verify binary versions
./target/release/atipicial-node --version

# Run preflight checks
make preflight
```

---

## Configuration

### Config File Format

Atipicial-rs uses TOML configuration files. Three bundled configs are provided:

| Config File | Network | Purpose |
|-------------|---------|---------|
| `atipicial_mainnet_node.toml` | MainNet | Standard mainnet configuration |
| `atipicial_testnet_node.toml` | TestNet | Development and testing |
| `atipicial_production_node.toml` | MainNet | Hardened production settings |

#### Configuration Sections

```toml
# Network identity
[network]
network_magic = 0x334F454E  # MainNet: 0x334F454E, TestNet: 0x3554334E
address_version = 0x35

# Storage backend
[storage]
backend = "rocksdb"         # Options: rocksdb, memory
data_dir = "./data/mainnet"
read_only = false

# P2P networking
[p2p]
port = 10333
max_connections = 100
min_desired_connections = 10
seed_nodes = [
    "seed1.atipicial.com:10333",
    "seed2.atipicial.com:10333",
    "seed3.atipicial.com:10333",
    "seed4.atipicial.com:10333",
    "seed5.atipicial.com:10333"
]
enable_compression = true
broadcast_history_limit = 100000

# JSON-RPC server
[rpc]
enabled = true
port = 10332
bind_address = "127.0.0.1"
cors_enabled = false
auth_enabled = true
max_gas_invoke = 50000000
max_iterator_results = 100
disabled_methods = ["openwallet"]

# Consensus (dBFT)
[consensus]
enabled = false
auto_start = false

# Telemetry and metrics
[telemetry]
[telemetry.metrics]
enabled = false
port = 9090
bind_address = "127.0.0.1"

# Logging configuration
[logging]
level = "info"              # Options: trace, debug, info, warn, error
format = "json"             # Options: json, pretty, compact
file_path = "./logs/atipicial-node-mainnet.log"
max_file_size = "100MB"
max_files = 10

# Blockchain parameters
[blockchain]
block_time = 15000          # 15 seconds in milliseconds
max_transactions_per_block = 512
max_free_transactions_per_block = 20

# Memory pool
[mempool]
max_transactions = 50000
max_transactions_per_sender = 200
```

### Environment Variables

The native `atipicial-node` binary is configured with CLI flags plus TOML. Docker and
compose add a small entrypoint layer that recognizes these environment
variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `ATC_NETWORK` | Docker config selection | `mainnet`, `testnet` |
| `ATC_CONFIG` | Docker custom TOML path | `/config/custom.toml` |
| `ATC_STORAGE` | Docker RocksDB directory passed as `--storage-path` | `/data/mainnet` |
| `ATC_PLUGINS_DIR` | Docker plugin configuration directory | `/data/Plugins` |
| `ATC_RPC_PORT` | Docker health-check port override only | `10332` |
| `RUST_LOG` | Rust logging directive | `info,atipicial_p2p=debug` |

### Network Selection (MainNet/TestNet)

#### Using Configuration Files

```bash
# MainNet node
./target/release/atipicial-node --config atipicial_mainnet_node.toml

# TestNet node
./target/release/atipicial-node --config atipicial_testnet_node.toml
```

Docker uses `ATC_NETWORK=mainnet|testnet` to select a bundled config. For bare
metal, pass the config path explicitly.

#### Using CLI Flags

```bash
# Override specific settings
./target/release/atipicial-node \
    --config atipicial_mainnet_node.toml \
    --network-magic 860833102
```

Set P2P port, seed nodes, storage backend, RPC port, and RPC hardening in TOML.

### Configuration Validation

Validate configuration without starting the node:

```bash
# Check config syntax and paths
./target/release/atipicial-node --config atipicial_mainnet_node.toml --check-config

# Check storage connectivity
./target/release/atipicial-node --config atipicial_mainnet_node.toml --check-storage

# Run all checks
./target/release/atipicial-node --config atipicial_mainnet_node.toml --check-all

# Or use make targets
make check-config CONFIG=atipicial_mainnet_node.toml
make preflight  # Checks both mainnet and testnet configs
```

---

## Docker Deployment

### Docker Build

```bash
# Build the Docker image
docker build -t atipicial-rs:latest .

# Build with specific tag
docker build -t atipicial-rs:v0.7.0 .
```

### Basic Docker Run

```bash
# Run on TestNet with persistent data
docker run -d \
    --name atipicial-node \
    -p 20332:20332 \
    -p 20333:20333 \
    -v $(pwd)/data:/data \
    -e ATC_NETWORK=testnet \
    atipicial-rs:latest

# Run on MainNet
docker run -d \
    --name atipicial-node \
    -p 10332:10332 \
    -p 10333:10333 \
    -v $(pwd)/data:/data \
    -e ATC_NETWORK=mainnet \
    atipicial-rs:latest
```

### Docker Compose Setup

The project includes a `docker-compose.yml` for easy deployment:

```bash
# Copy environment template
cp .env.example .env

# Edit configuration
nano .env

# Start the node
docker compose up -d atipicial-node

# View logs
docker compose logs -f atipicial-node

# Stop the node
docker compose down
```

#### Environment Variables (.env)

```bash
# Network selection: mainnet or testnet
ATC_NETWORK=testnet

# Plugin directory
ATC_PLUGINS_DIR=/data/Plugins

# Custom config (optional)
# ATC_CONFIG=/config/custom.toml

# Custom storage path (optional)
# ATC_STORAGE=/data/blockchain

# RPC health-check port override (optional; node RPC port comes from TOML)
# ATC_RPC_PORT=20332

# Logging
RUST_LOG=info

# Grafana password (for monitoring profile)
GRAFANA_PASSWORD=admin
```

### Volume Mounts

Recommended directory structure for Docker volumes:

```
/data
├── mainnet/          # MainNet blockchain data
├── testnet/          # TestNet blockchain data
├── Plugins/          # Plugin configurations
│   └── RpcServer/
│       └── RpcServer.json
└── Logs/             # Log files
    └── atipicial-node.log
```

#### Docker Volume Configuration

```yaml
# docker-compose.yml snippet
volumes:
  # Named volume for data persistence
  atipicial-data:
    driver: local

  # Bind mount for custom configuration
  - ./config:/config:ro

  # Bind mount for logs on host
  - ./logs:/data/Logs
```

### Container Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ATC_NETWORK` | `testnet` | Network selection |
| `ATC_CONFIG` | - | Custom config path |
| `ATC_STORAGE` | `/data/{network}` | Data directory |
| `ATC_PLUGINS_DIR` | `/data/Plugins` | Plugin directory |
| `ATC_RPC_PORT` | auto | Health-check port override |
| `RUST_LOG` | `info` | Log level |

### Monitoring Profile (Grafana)

```bash
# Start with monitoring
docker compose --profile monitoring up -d

# Or use make target
make compose-monitor

# Access Grafana at http://localhost:3000
# Default credentials: admin/admin (or GRAFANA_PASSWORD from .env)
```

### Docker Health Checks

The container includes built-in health checks:

```bash
# Check container health
docker inspect --format='{{.State.Health.Status}}' atipicial-node

# Manual health check
curl -sf -X POST \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","id":1,"method":"getversion","params":[]}' \
    http://localhost:20332
```

---

## Running a Node

### Starting the Node

#### Systemd Service (Recommended for Production)

Create `/etc/systemd/system/atipicial-node.service`:

```ini
[Unit]
Description=Atipicial N3 Rust Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=atipicial
Group=atipicial
WorkingDirectory=/opt/atipicial

# Binary and config
ExecStart=/opt/atipicial/atipicial-node --config /opt/atipicial/atipicial_production_node.toml

# Restart policy
Restart=always
RestartSec=5
StartLimitInterval=60s
StartLimitBurst=3

# Resource limits
LimitNOFILE=65535
LimitNPROC=8192

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/atipicial/data /var/log/atipicial

# Environment
Environment=RUST_LOG=info
Environment=ATC_PLUGINS_DIR=/var/atipicial/Plugins

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable atipicial-node
sudo systemctl start atipicial-node
```

#### Direct Execution

```bash
# Basic start
./target/release/atipicial-node --config atipicial_mainnet_node.toml

# With custom data directory
./target/release/atipicial-node \
    --config atipicial_mainnet_node.toml \
    --storage-path /var/atipicial/data

# With logging options
RUST_LOG=info,atipicial_p2p=debug ./target/release/atipicial-node \
    --config atipicial_mainnet_node.toml

# Hardened RPC settings are configured in the TOML [rpc] section.
```

### Monitoring

#### Health Checks

```bash
curl -sf -X POST \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","id":1,"method":"getversion","params":[]}' \
    http://localhost:10332
```

Docker health checks use the same `getversion` RPC probe.

#### RPC Status Commands

```bash
# Blockchain height
curl -s -X POST \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' \
    http://localhost:10332

# Peer count
curl -s -X POST \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","id":1,"method":"getconnectioncount","params":[]}' \
    http://localhost:10332
```

#### Log Management

Log configuration in TOML:
```toml
[logging]
level = "info"              # trace, debug, info, warn, error
format = "json"             # json, pretty, compact
file_path = "/var/log/atipicial/node.log"
max_file_size = "100MB"
max_files = 10
```

Log rotation with logrotate (`/etc/logrotate.d/atipicial-node`):
```
/var/log/atipicial/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0644 atipicial atipicial
    sharedscripts
    postrotate
        systemctl reload atipicial-node || true
    endscript
}
```

Viewing logs:
```bash
# Via systemd
sudo journalctl -u atipicial-node -f

# Via log file
tail -f /var/log/atipicial/node.log

# Filter by level
jq 'select(.level == "ERROR")' /var/log/atipicial/node.log
```

---

## Hardware Requirements

### Minimum Specifications

For running a basic node (syncing and validating):

| Resource | Minimum |
|----------|---------|
| CPU | 2 cores (x86_64) |
| RAM | 4 GB |
| Storage | 100 GB SSD |
| Network | 10 Mbps symmetric |

### Recommended Specifications

For production nodes with RPC enabled:

| Resource | Recommended |
|----------|-------------|
| CPU | 4+ cores (x86_64 or ARM64) |
| RAM | 8 GB |
| Storage | 500 GB NVMe SSD |
| Network | 100 Mbps symmetric |

### Consensus Node Requirements

For nodes participating in dBFT consensus:

| Resource | Requirement |
|----------|-------------|
| CPU | 8+ cores |
| RAM | 16 GB |
| Storage | 1 TB NVMe SSD |
| Network | 1 Gbps dedicated |
| Latency | < 50ms to other CNs |

### Storage Requirements

| Network | Current Size | Growth Rate |
|---------|-------------|-------------|
| MainNet | ~50 GB | ~2 GB/month |
| TestNet | ~30 GB | ~1 GB/month |

Storage breakdown:
- RocksDB data: ~90% of storage
- Logs: ~5-10% (with rotation)
- State caches: ~5%

**Important:** RocksDB requires fast, durable storage. Avoid:
- Network-attached storage (NAS) for primary data
- HDDs (insufficient IOPS)
- tmpfs/ephemeral disks

---

## Upgrading

### Migration Procedures

#### Standard Upgrade

1. **Prepare backup:**
```bash
# Stop the node
sudo systemctl stop atipicial-node

# Create backup
make backup-rocksdb ROCKSDB_PATH=/var/atipicial/mainnet BACKUP_DIR=/backups/$(date +%Y%m%d)
```

2. **Deploy new version:**
```bash
# Pull latest code
git fetch origin
git checkout v0.7.1  # or latest tag

# Build new version
cargo build --release -p atipicial-node

# Run preflight checks
make preflight
```

3. **Validate and start:**
```bash
# Check configuration compatibility
./target/release/atipicial-node --config /opt/atipicial/config.toml --check-all

# Start the node
sudo systemctl start atipicial-node

# Monitor logs
sudo journalctl -u atipicial-node -f
```

#### Major Version Upgrade

For breaking changes (check CHANGELOG.md):

1. Export chain data if migration needed
2. Clear data directory if resync required
3. Update configuration schema
4. Deploy and resync from genesis or bootstrap

### Backup/Restore

#### Automated Backup

Using the included script:
```bash
# Daily backup via cron (add to crontab)
0 2 * * * /opt/atipicial/scripts/backup-rocksdb.sh /var/atipicial/mainnet /backups

# Or use make target
make backup-rocksdb ROCKSDB_PATH=/var/atipicial/mainnet BACKUP_DIR=/backups
```

#### Manual Backup

```bash
# Stop node (recommended for consistency)
sudo systemctl stop atipicial-node

# Create tarball
sudo tar czf /backups/atipicial-$(date +%Y%m%d).tar.gz /var/atipicial/mainnet

# Start node
sudo systemctl start atipicial-node
```

#### Restore from Backup

```bash
# Stop the node
sudo systemctl stop atipicial-node

# Remove current data (or move aside)
sudo mv /var/atipicial/mainnet /var/atipicial/mainnet.old

# Extract backup
sudo tar xzf /backups/atipicial-20260128.tar.gz -C /

# Fix permissions
sudo chown -R atipicial:atipicial /var/atipicial/mainnet

# Start node
sudo systemctl start atipicial-node
```

#### Rolling Back

If upgrade fails:
```bash
# Stop current version
sudo systemctl stop atipicial-node

# Restore previous binaries from backup
sudo cp /backups/atipicial-node-v0.7.0 /usr/local/bin/atipicial-node

# Restore data if needed
sudo rm -rf /var/atipicial/mainnet
sudo tar xzf /backups/atipicial-pre-upgrade.tar.gz -C /

# Start previous version
sudo systemctl start atipicial-node
```

---

## Troubleshooting

### Common Issues

#### Node won't start

```bash
# Check configuration
./target/release/atipicial-node --config /opt/atipicial/config.toml --check-all

# Verify permissions
ls -la /var/atipicial/data
ls -la /var/log/atipicial

# Check logs
sudo journalctl -u atipicial-node --no-pager -n 50
```

#### Sync is slow

- Check network connectivity to seed nodes
- Verify disk I/O performance
- Increase peer connections in config
- Check for firewall blocking P2P port

#### RPC not responding

```bash
# Test RPC locally
curl -s -X POST \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","id":1,"method":"getversion","params":[]}' \
    http://127.0.0.1:10332

# Check if RPC is enabled in config
grep -A5 '\[rpc\]' /opt/atipicial/config.toml
```

### Support Resources

- [GitHub Issues](https://github.com/r3e-network/atipicial-rs/issues)
- [Operations Guide](./OPERATIONS.md)
- [Monitoring Guide](./MONITORING.md)
- [RPC Hardening Guide](./RPC_HARDENING.md)

---

## Security Checklist

- [ ] Use production profile for builds
- [ ] Enable RPC authentication (`auth_enabled = true`)
- [ ] Disable CORS in production (`cors_enabled = false`)
- [ ] Run as non-root user
- [ ] Configure firewall (P2P port, RPC port)
- [ ] Enable TLS for RPC (via reverse proxy)
- [ ] Restrict RPC bind address to localhost
- [ ] Disable risky RPC methods (`disabled_methods`)
- [ ] Set up log rotation
- [ ] Configure automated backups
- [ ] Enable monitoring and alerting
- [ ] Keep system packages updated

---

## Appendix

### Port Reference

| Network | P2P Port | RPC Port | Usage |
|---------|----------|----------|-------|
| MainNet | 10333 | 10332 | Production network |
| TestNet | 20333 | 20332 | Testing network |
| Private | 30333 | 30332 | Local development |

### File Locations

| Component | Default Path | Configurable |
|-----------|--------------|--------------|
| Binary | `/usr/local/bin/atipicial-node` | Yes |
| Config | `/etc/atipicial/` | Yes |
| Data | `/var/atipicial/data/` | Yes |
| Logs | `/var/log/atipicial/` | Yes |
| Plugins | `/var/atipicial/Plugins/` | Yes |
| PID | `/run/atipicial-node/atipicial-node.pid` | Via systemd |

### Makefile Reference

| Command | Description |
|---------|-------------|
| `make build-release` | Build release binaries |
| `make compose-up` | Start docker-compose stack |
| `make compose-down` | Stop docker-compose stack |
| `make preflight` | Run config checks |
| `make backup-rocksdb` | Backup RocksDB data |
| `make ci` | Run full CI checks |

---

> **Atipicial Chain** — sovereign Layer-1 for smart contracts and digital assets.
> 👑 Founded & engineered by **xmoohad** — Blockchain Scientist · Computer Programmer.
> `ATC` Atipicial Coin · `ATD` AtipicialDollar · addresses begin with **A**
