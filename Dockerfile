# Multi-stage Dockerfile for Atipicial Rust Node
# R3E Network <jimmy@r3e.network>

FROM rust:1.89-bullseye AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    g++ \
    cmake \
    make \
    pkg-config \
    llvm \
    libclang-dev \
    clang \
    libsnappy-dev \
    liblz4-dev \
    libzstd-dev \
    zlib1g-dev \
    libbz2-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# libclang for bindgen (MDBX bindings). Bullseye ships LLVM 11; the
# libclang-dev package puts libclang.so under /usr/lib/llvm-11/lib. The ENV
# must be set directly (not via bashrc) so it's visible to the RUN cargo build.
ENV LIBCLANG_PATH=/usr/lib/llvm-11/lib

WORKDIR /workspace/atipicial-rs

# Copy manifests and workspace crates (kept explicit for better Docker layer caching).
COPY Cargo.toml Cargo.lock ./
COPY atipicial-primitives/ atipicial-primitives/
COPY atipicial-config/ atipicial-config/
COPY atipicial-crypto/ atipicial-crypto/
COPY atipicial-trie/ atipicial-trie/
COPY atipicial-storage/ atipicial-storage/
COPY atipicial-static-files/ atipicial-static-files/
COPY atipicial-state-packs/ atipicial-state-packs/
COPY atipicial-checkpoint/ atipicial-checkpoint/
COPY atipicial-io/ atipicial-io/
COPY atipicial-vm/ atipicial-vm/
COPY atipicial-error/ atipicial-error/
COPY atipicial-serialization/ atipicial-serialization/
COPY atipicial-manifest/ atipicial-manifest/
COPY atipicial-payloads/ atipicial-payloads/
COPY atipicial-consensus/ atipicial-consensus/
COPY atipicial-hsm/ atipicial-hsm/
COPY atipicial-runtime/ atipicial-runtime/
COPY atipicial-execution/ atipicial-execution/
COPY atipicial-native-contracts/ atipicial-native-contracts/
COPY atipicial-state-service/ atipicial-state-service/
COPY atipicial-mempool/ atipicial-mempool/
COPY atipicial-blockchain/ atipicial-blockchain/
COPY atipicial-network/ atipicial-network/
COPY atipicial-wallets/ atipicial-wallets/
COPY atipicial-indexer/ atipicial-indexer/
COPY atipicial-system/ atipicial-system/
COPY atipicial-rpc/ atipicial-rpc/
COPY atipicial-oracle-service/ atipicial-oracle-service/
COPY atipicial-node/ atipicial-node/
COPY atipicial-test-fixtures/ atipicial-test-fixtures/
COPY tests/ tests/
COPY benches-package/ benches-package/
COPY scripts/ scripts/
COPY atipicial_mainnet_node.toml atipicial_testnet_node.toml atipicial_production_node.toml ./

# Build the node daemon.
RUN cargo build --release --locked -p atipicial-node

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    bash \
    libsnappy1v5 \
    liblz4-1 \
    libzstd1 \
    zlib1g \
    libbz2-1.0 \
    libssl1.1 \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Create atipicial user and home
RUN groupadd -r atipicial && useradd -r -g atipicial -d /home/atipicial atipicial \
    && mkdir -p /home/atipicial && chown -R atipicial:atipicial /home/atipicial

# Create data directories (Logs for default config; keep /data/logs for backward compatibility)
RUN mkdir -p /data /data/blocks /data/Logs /data/logs && chown -R atipicial:atipicial /data

# Copy binaries from builder stage
COPY --from=builder /workspace/atipicial-rs/target/release/atipicial-node /usr/local/bin/atipicial-node

# Copy default configs and entrypoint
COPY atipicial_mainnet_node.toml /etc/atipicial/atipicial_mainnet_node.toml
COPY atipicial_testnet_node.toml /etc/atipicial/atipicial_testnet_node.toml
COPY atipicial_production_node.toml /etc/atipicial/atipicial_production_node.toml
COPY config/*.toml /etc/atipicial/config/
COPY scripts/docker-entrypoint.sh /usr/local/bin/atipicial-entrypoint.sh
RUN chmod +x /usr/local/bin/atipicial-entrypoint.sh && chown -R atipicial:atipicial /etc/atipicial

# Set up volumes
VOLUME ["/data"]

# Switch to atipicial user and working directory
USER atipicial
WORKDIR /data
ENV HOME=/home/atipicial

# Expose ports
# TestNet ports
EXPOSE 20332 20333
# MainNet ports
EXPOSE 10332 10333
# Telemetry / health endpoints used by service-provider presets
EXPOSE 9090 9091

# Health check - JSON-RPC getversion on the configured RPC port
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD ["sh", "-c", "port_file=/tmp/atipicial_rpc_port; if [ -f \"$port_file\" ]; then port=$(cat \"$port_file\"); else port=${ATC_RPC_PORT:-}; fi; if [ -z \"$port\" ]; then port=20332; case \"${ATC_NETWORK:-testnet}\" in [Mm]ain*) port=10332 ;; esac; fi; curl -sf -X POST -H 'Content-Type: application/json' --data '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getversion\",\"params\":[]}' http://127.0.0.1:${port} >/dev/null"]

# Environment variables
ENV ATC_NETWORK=testnet \
    ATC_BACKEND=mdbx \
    ATC_PLUGINS_DIR=/data/Plugins \
    RUST_LOG=info

# Default command for atipicial-cli (configurable via env)
ENTRYPOINT ["/usr/local/bin/atipicial-entrypoint.sh"]
CMD []

# Metadata
LABEL org.opencontainers.image.title="Atipicial-Rust-Node"
LABEL org.opencontainers.image.description="Rust implementation of the Atipicial N3 blockchain protocol"
LABEL org.opencontainers.image.url="https://github.com/r3e-network/atipicial-rs"
LABEL org.opencontainers.image.documentation="https://github.com/r3e-network/atipicial-rs/blob/master/README.md"
LABEL org.opencontainers.image.source="https://github.com/r3e-network/atipicial-rs"
LABEL org.opencontainers.image.vendor="R3E Network"
LABEL org.opencontainers.image.licenses="MIT"
