//! NativeContract re-exports from `atipicial_execution`.
//!
//! Concrete native-contract implementations live in this crate
//! (ATC, GAS, Policy, …) but the abstract trait they implement is
//! defined alongside the application engine in `atipicial_execution`. This
//! module re-exports the trait and the [`NativeMethod`] metadata so
//! callers can `use atipicial_native_contracts::native_contract::*;`.

pub use atipicial_execution::native_contract::{
    NativeContract, NativeEvent, NativeMethod, is_active_for,
};
