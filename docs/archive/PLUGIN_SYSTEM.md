<!-- Atipicial Chain · sovereign Layer-1 for smart contracts and digital assets -->
<!-- 👑 Founded & engineered by xmoohad — Blockchain Scientist · Computer Programmer -->

# Plugin System Guide

## Overview

The Rust implementation follows a **compile-time integration** approach rather than dynamic plugin loading used in the C# version. This provides better type safety, performance, and deployment simplicity.

## Plugin Architecture Comparison

### C# (Dynamic Loading)

```csharp
// Plugin attribute marks a class for reflection-based loading
[Plugin]
public class RpcServer : Plugin
{
    protected override void Configure() { }
    protected override void OnSystemLoaded(AtipicialSystem system) { }
}

// Separate config file per plugin
// Plugins/RpcServer/RpcServer.json
```

**Workflow:**
1. Scan `Plugins/` directory for assemblies
2. Reflectively find `[Plugin]`-attributed classes
3. Load config from `<PluginName>.json`
4. Instantiate and inject dependencies

### Rust (Compile-time Integration)

```rust
// Feature flag enables plugin at build time
#[cfg(feature = "rpc")]
mod rpc_server;

// Feature flag in Cargo.toml
[features]
default = ["rpc", "consensus", "oracle"]
rpc = []
consensus = []
oracle = []
hsm = []
tee = []
```

**Workflow:**
1. Cargo features determine what gets compiled
2. Services registered in `atipicial-node` main()
3. Configuration via unified `atipicial-node.toml`
4. ServiceRegistry manages lifecycle

## Module Mapping

| Functionality | C# Plugin | Rust Module/Crate | Feature Flag |
|--------------|-------------|-------------------|--------------|
| **RPC Server** | `Atipicial.Plugins.RpcServer` | `atipicial-rpc` (server module) | `rpc` |
| **Consensus** | `Atipicial.Plugins.DBFTPlugin` | `atipicial-consensus` | `consensus` |
| **Oracle Service** | `Atipicial.Plugins.OracleService` | `atipicial-core::oracle_service` | `oracle` |
| **State Service** | `Atipicial.Plugins.StateService` | `atipicial-core::state_service` | `state-root` |
| **Tokens Tracker** | `Atipicial.Plugins.TokensTracker` | `atipicial-core::tokens_tracker` | Built-in |
| **Application Logs** | `Atipicial.Plugins.ApplicationLogs` | `atipicial-core::application_logs` | Built-in |
| **HSM Support** | Atipicial.Plugins.SignClient | `atipicial-hsm` + integration | `hsm` |
| **TEE Support** | (External) | `atipicial-tee` | `tee` |
| **AtipicialFs** | `OracleService` protocol | `oracle_service/atipicialfs` | `oracle` |

## Configuration

### C# Style (Per-Plugin Config)

```json
// Plugins/RpcServer/RpcServer.json
{
  "Network": 5195086,
  "BindAddress": "127.0.0.1",
  "Port": 10332,
  "User": "",
  "Pass": ""
}

// Plugins/OracleService/OracleService.json
{
  "Network": 5195086,
  "AutoStart": true,
  "AllowedContentTypes": ["Url"],
  "MaxPrice": 100000000
}
```

### Rust Style (Unified Config)

```toml
# atipicial-node.toml
[rpc]
enabled = true
bind_address = "127.0.0.1"
port = 10332
user = ""
pass = ""

[oracle]
enabled = true
auto_start = true
max_price = 100000000
```

## Service Lifecycle

### C# Plugin Lifecycle

```csharp
public abstract class Plugin
{
    protected abstract void Configure();
    protected abstract void Dispose();
    protected virtual void OnSystemLoaded(AtipicialSystem system) { }
    protected virtual void OnSystemStarted() { }
}
```

**Sequence:**
1. `Configure()` - Load configuration
2. `OnSystemLoaded()` - Access AtipicialSystem services
3. `OnSystemStarted()` - System ready
4. `Dispose()` - Cleanup

### Rust Service Lifecycle

```rust
// ServiceRegistry pattern
pub struct ServiceRegistry {
    services: HashMap<TypeId, Arc<dyn Any>>,
}

// Service initialization in atipicial-node main()
let _application_logs_service =
    maybe_enable_application_logs(&node_config, &protocol_settings, &system)?;

let _tokens_tracker_service =
    maybe_enable_tokens_tracker(&node_config, &protocol_settings, &system)?;

let oracle_service =
    maybe_enable_oracle_service(&node_config, &protocol_settings, &system)?;
```

**Sequence:**
1. Check config enablement
2. Instantiate service with dependencies
3. Register with ServiceRegistry
4. Start background tasks
5. Drop on shutdown (RAII)

## Adding a New Service

### Step 1: Define Feature

Add to `atipicial-node/Cargo.toml`:

```toml
[features]
default = ["my-service"]
my-service = []
```

### Step 2: Implement Service Module

Create `atipicial-node/src/my_service.rs`:

```rust
use crate::config::MyServiceSettings;
use atipicial_core::atipicial_system::AtipicialSystem;
use std::sync::Arc;

pub struct MyService {
    system: Arc<AtipicialSystem>,
    settings: MyServiceSettings,
}

impl MyService {
    pub fn new(
        system: Arc<AtipicialSystem>,
        settings: MyServiceSettings,
    ) -> Self {
        Self { system, settings }
    }

    pub fn start(&self) -> Result<(), Error> {
        // Service logic here
        Ok(())
    }
}
```

### Step 3: Add Config

Add to `atipicial-node/src/config.rs`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MyServiceSettings {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub config_path: Option<String>,
}
```

Add to `NodeConfig`:

```rust
#[derive(Debug, Deserialize)]
pub struct NodeConfig {
    // ... existing fields ...
    #[serde(default)]
    pub my_service: MyServiceSettings,
}
```

### Step 4: Add CLI Flags

Add to `atipicial-node/src/cli.rs`:

```rust
#[derive(Parser, Debug)]
pub struct NodeCli {
    // ... existing flags ...

    #[clap(group = "my-service")]
    pub my_service_args: Option<MyServiceArgs>,
}

#[derive(Parser, Debug)]
pub struct MyServiceArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
}
```

### Step 5: Initialize in main()

Add to `atipicial-node/src/main.rs`:

```rust
#[cfg(feature = "my-service")]
mod my_service;

// ... in main() function ...
#[cfg(feature = "my-service")]
let _my_service = maybe_enable_my_service(&node_config, &system)?;

fn maybe_enable_my_service(
    config: &NodeConfig,
    system: &Arc<AtipicialSystem>,
) -> Result<Option<MyService>> {
    if !config.my_service.enabled {
        return Ok(None);
    }

    let service = MyService::new(system.clone(), config.my_service.clone());
    service.start()?;
    Ok(Some(service))
}
```

### Step 6: Update TOML Schema

Add to `atipicial-node/config.rs` or create schema validation:

```rust
pub fn validate_node_config(
    config: &NodeConfig,
    // ... other params ...
) -> Result<(), Error> {
    // ... existing validation ...

    if config.my_service.enabled {
        // Validate my-service specific settings
    }

    Ok(())
}
```

## Beaefits of Rust Approach

### 1. Type Safety
- Compile-time checking prevents configuration errors
- No reflection overhead
- Better IDE support and documentation

### 2. Performance
- No dynamic assembly loading
- Zero-cost abstractions
- Inlining opportunities

### 3. Deployment
- Single binary (`atipicial-node`) vs multiple DLLs
- No plugin version conflicts
- Easier containerization

### 4. Security
- No untrusted code loading
- All code audited at build time
- Smaller attack surface

## Migration from C# Plugins

If you have custom C# plugins, follow this guide:

### 1. Identify Dependencies

What services does your plugin need?
- `AtipicialSystem` (core services)
- `IWalletProvider` (wallet access)
- `LocalNode` (P2P)
- `Blockchain` (ledger)

### 2. Port Logic

- Translate C# to Rust
- Replace Akka actors with Tokio tasks
- Use `ServiceRegistry` for dependency injection

### 3. Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_initialization() {
        // Test your service
    }
}
```

### 4. Update Documentation

- Add to `ARCHITECTURE_COMPARISON.md`
- Document config options in `README.md`
- Add migration notes

## Common Patterns

### Background Tasks

```rust
pub struct MyService {
    handle: JoinHandle<()>,
}

impl MyService {
    pub fn start(&self) -> Result<()> {
        self.handle = tokio::spawn(async move {
            loop {
                // Background work
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
        Ok(())
    }
}
```

### Event Handlers

```rust
use atipicial_core::i_event_handlers::{ICommittingHandler, IPersistHandler};

impl ICommittingHandler for MyService {
    fn i_blockchain_committing_handler(&self, _block: &Block) {
        // Handle block committing
    }
}

impl IPersistHandler for MyService {
    fn i_blockchain_persist_handler(&self, block: &Block) {
        // Handle block persisted
    }
}
```

### RPC Integration

```rust
impl RpcServerMyService {
    pub fn register_handlers() -> Vec<RpcHandler> {
        vec![
            Self::handler("myServiceMethod", Self::my_method),
        ]
    }

    fn my_method(server: &RpcServer, params: &[Value])
        -> Result<Value, RpcException>
    {
        // Implement RPC method
        Ok(json!("result"))
    }
}
```

## Troubleshooting

### Service Not Starting

Check:
1. Feature flag enabled in `Cargo.toml`?
2. Configuration has `enabled = true`?
3. Dependencies available in `ServiceRegistry`?
4. Check logs for startup errors

### Config Validation Errors

Check:
1. TOML syntax correct?
2. Config schema updated?
3. All required fields present?
4. Types match expected values?

### Build Errors

Check:
1. All modules compiled with proper features?
2. Dependencies in `Cargo.toml`?
3. Imports and visibility correct?

## Further Reading

- [Architecture Comparison](./ARCHITECTURE_COMPARISON.md)
- [Deployment Guide](./DEPLOYMENT.md)
- [Operations Guide](./OPERATIONS.md)
- [Atipicial Node Core Architecture](./ARCHITECTURE.md)

---

> **Atipicial Chain** — sovereign Layer-1 for smart contracts and digital assets.
> 👑 Founded & engineered by **xmoohad** — Blockchain Scientist · Computer Programmer.
> `ATC` Atipicial Coin · `ATD` AtipicialDollar · addresses begin with **A**
