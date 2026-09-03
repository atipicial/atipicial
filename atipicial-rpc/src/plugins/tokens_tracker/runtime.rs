//! TokensTracker runtime.
//!
//! Provides balance/transfer indexing for AEP-17/AEP-11 standards. This struct is
//! registered as a committing/committed handler to process block events.

use super::settings::TokensTrackerSettings;
use super::trackers::aep_11::Aep11Tracker;
use super::trackers::aep_17::Aep17Tracker;
use super::trackers::tracker_base::Tracker;
use atipicial_config::ProtocolSettings;
use atipicial_execution::native_contract_provider::NativeContractProvider;
use atipicial_native_contracts::StandardNativeProvider;
use atipicial_payloads::ApplicationExecuted;
use atipicial_payloads::Block;
use atipicial_primitives::panic_message;
use atipicial_runtime::{CommittedHandler, CommittingHandler};
use atipicial_storage::persistence::providers::MemoryStore;
use atipicial_storage::persistence::{CacheRead, DataCache, Store};
use parking_lot::RwLock;
use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::error;

/// Runtime handler for token balance/transfer tracking.
///
/// Implements `CommittingHandler` and `CommittedHandler` to index
/// token transfers during block commits.
pub struct TokensTracker<P = StandardNativeProvider, S: Store = MemoryStore>
where
    P: NativeContractProvider,
{
    settings: TokensTrackerSettings,
    trackers: RwLock<Vec<TrackerRuntime<P, S>>>,
    disabled: AtomicBool,
    _provider: std::marker::PhantomData<P>,
    _store: std::marker::PhantomData<fn(&S)>,
}

enum TrackerRuntime<P, S>
where
    P: NativeContractProvider,
    S: Store,
{
    Aep17(Aep17Tracker<P, S>),
    Aep11(Aep11Tracker<P, S>),
}

impl<P, S> TrackerRuntime<P, S>
where
    P: NativeContractProvider + 'static,
    S: Store + 'static,
{
    fn track_name(&self) -> &str {
        match self {
            Self::Aep17(tracker) => tracker.track_name(),
            Self::Aep11(tracker) => tracker.track_name(),
        }
    }

    fn reset_batch(&mut self) {
        match self {
            Self::Aep17(tracker) => tracker.reset_batch(),
            Self::Aep11(tracker) => tracker.reset_batch(),
        }
    }

    fn on_persist<B: CacheRead>(
        &mut self,
        block: &Block,
        snapshot: &DataCache<B>,
        application_executed_list: &[ApplicationExecuted],
    ) {
        match self {
            Self::Aep17(tracker) => tracker.on_persist(block, snapshot, application_executed_list),
            Self::Aep11(tracker) => tracker.on_persist(block, snapshot, application_executed_list),
        }
    }

    fn commit(&mut self) -> atipicial_error::CoreResult<()> {
        match self {
            Self::Aep17(tracker) => tracker.commit(),
            Self::Aep11(tracker) => tracker.commit(),
        }
    }
}

impl<P, S> TokensTracker<P, S>
where
    P: NativeContractProvider + 'static,
    S: Store + 'static,
{
    /// Creates a new TokensTracker with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `settings` - Tracker configuration
    /// * `db` - Database store for balance/transfer data
    /// * `protocol_settings` - Protocol settings (for VM execution)
    pub fn new(
        settings: TokensTrackerSettings,
        db: Arc<S>,
        protocol_settings: Arc<ProtocolSettings>,
        native_contract_provider: Arc<P>,
    ) -> Self {
        let mut trackers = Vec::new();

        if settings.enabled_aep17() {
            trackers.push(TrackerRuntime::Aep17(Aep17Tracker::new(
                Arc::clone(&db),
                settings.max_results,
                settings.track_history,
                Arc::clone(&protocol_settings),
                Arc::clone(&native_contract_provider),
            )));
        }

        if settings.enabled_aep11() {
            trackers.push(TrackerRuntime::Aep11(Aep11Tracker::new(
                Arc::clone(&db),
                settings.max_results,
                settings.track_history,
                Arc::clone(&protocol_settings),
                Arc::clone(&native_contract_provider),
            )));
        }

        Self {
            settings,
            trackers: RwLock::new(trackers),
            disabled: AtomicBool::new(false),
            _provider: std::marker::PhantomData,
            _store: std::marker::PhantomData,
        }
    }

    /// Returns a reference to the settings.
    pub fn settings(&self) -> &TokensTrackerSettings {
        &self.settings
    }

    fn handle_panic(&self, tracker: &str, action: &str, payload: Box<dyn Any + Send>) -> bool {
        let message = panic_message(payload.as_ref(), "panic");
        self.handle_failure(tracker, action, "panicked", message)
    }

    fn handle_error(&self, tracker: &str, action: &str, error_message: String) -> bool {
        self.handle_failure(tracker, action, "failed", error_message)
    }

    fn handle_failure(
        &self,
        tracker: &str,
        action: &str,
        outcome: &'static str,
        error_message: String,
    ) -> bool {
        match self.settings.exception_policy {
            atipicial_primitives::unhandled_exception_policy::UnhandledExceptionPolicy::Ignore => {
                return true;
            }
            _ => {
                error!(
                    target: "atipicial::tokens_tracker",
                    track = tracker,
                    action,
                    error = error_message,
                    "tokens tracker {outcome}"
                );
            }
        }

        self.apply_exception_policy()
    }

    fn apply_exception_policy(&self) -> bool {
        self.settings
            .exception_policy
            .apply(|| self.disabled.store(true, Ordering::Relaxed))
    }

    fn run_tracker_action<F>(&self, tracker: &str, action: &str, f: F) -> bool
    where
        F: FnOnce(),
    {
        match panic::catch_unwind(AssertUnwindSafe(f)) {
            Ok(()) => true,
            Err(payload) => self.handle_panic(tracker, action, payload),
        }
    }

    fn run_tracker_result_action<F>(&self, tracker: &str, action: &str, f: F) -> bool
    where
        F: FnOnce() -> atipicial_error::CoreResult<()>,
    {
        match panic::catch_unwind(AssertUnwindSafe(f)) {
            Ok(Ok(())) => true,
            Ok(Err(err)) => self.handle_error(tracker, action, err.to_string()),
            Err(payload) => self.handle_panic(tracker, action, payload),
        }
    }
}

impl<P, S> CommittingHandler for TokensTracker<P, S>
where
    P: NativeContractProvider + 'static,
    S: Store + 'static,
{
    fn blockchain_committing_handler<B: atipicial_storage::CacheRead>(
        &self,
        network: u32,
        block: &Block,
        snapshot: &DataCache<B>,
        application_executed_list: &[ApplicationExecuted],
    ) {
        if network != self.settings.network {
            return;
        }

        if self.disabled.load(Ordering::Relaxed) {
            return;
        }

        let mut trackers = self.trackers.write();
        for tracker in trackers.iter_mut() {
            if self.disabled.load(Ordering::Relaxed) {
                break;
            }
            let track_name = tracker.track_name().to_string();
            if !self.run_tracker_action(&track_name, "reset_batch", || tracker.reset_batch()) {
                break;
            }
            if !self.run_tracker_action(&track_name, "on_persist", || {
                tracker.on_persist(block, snapshot, application_executed_list)
            }) {
                break;
            }
        }
    }
}

impl<P, S> CommittedHandler for TokensTracker<P, S>
where
    P: NativeContractProvider + 'static,
    S: Store + 'static,
{
    fn blockchain_committed_handler(&self, network: u32, _block: &Block) {
        if network != self.settings.network {
            return;
        }

        if self.disabled.load(Ordering::Relaxed) {
            return;
        }

        let mut trackers = self.trackers.write();
        for tracker in trackers.iter_mut() {
            if self.disabled.load(Ordering::Relaxed) {
                break;
            }
            let track_name = tracker.track_name().to_string();
            if !self.run_tracker_result_action(&track_name, "commit", || tracker.commit()) {
                break;
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/plugins/tokens_tracker/runtime.rs"]
mod tests;
