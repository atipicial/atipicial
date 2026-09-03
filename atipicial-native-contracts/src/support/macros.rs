//! Local native-contract implementation macros.
//!
//! These macros keep the root of each native contract uniform: handle identity,
//! `NativeContract` identity methods, and binding-table dispatch all follow the
//! same shape across the crate.

macro_rules! native_contract_handle {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            id: $id:expr,
            contract_name: $contract_name:expr,
            hash: $hash:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Default, Clone, Copy)]
        $vis struct $name;

        impl $name {
            #[doc = concat!("Stable native contract id (matches C# `", $contract_name, "`).")]
            pub const ID: i32 = $id;
            #[doc = concat!("Stable native contract name (matches C# `", $contract_name, ".Name`).")]
            pub const NAME: &'static str = $contract_name;

            #[doc = concat!("Construct a new `", stringify!($name), "` handle.")]
            #[must_use]
            pub const fn new() -> Self {
                Self
            }

            #[doc = concat!("Returns the stable native contract id for `", stringify!($name), "`.")]
            #[must_use]
            pub const fn id(&self) -> i32 {
                Self::ID
            }

            #[doc = concat!("Returns the ", $contract_name, " script hash.")]
            #[must_use]
            pub fn hash(&self) -> atipicial_primitives::UInt160 {
                Self::script_hash()
            }

            #[doc = concat!("Returns the stable native contract name for `", stringify!($name), "`.")]
            #[must_use]
            pub fn name(&self) -> &str {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::name(self)
            }

            #[doc = concat!("Returns the activation hardfork for `", stringify!($name), "`, if any.")]
            #[must_use]
            pub fn active_in(&self) -> Option<atipicial_config::Hardfork> {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::active_in(self)
            }

            #[doc = concat!("Returns the manifest-refresh hardforks for `", stringify!($name), "`.")]
            #[must_use]
            pub fn activations(&self) -> &'static [atipicial_config::Hardfork] {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::activations(self)
            }

            #[doc = concat!("Returns the used hardforks for `", stringify!($name), "` metadata.")]
            #[must_use]
            pub fn used_hardforks(&self) -> Vec<atipicial_config::Hardfork> {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::used_hardforks(self)
            }

            #[doc = concat!("Returns whether `", stringify!($name), "` is active at the given block.")]
            #[must_use]
            pub fn is_active(
                &self,
                settings: &atipicial_config::ProtocolSettings,
                block_height: u32,
            ) -> bool {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::is_active(self, settings, block_height)
            }

            #[doc = concat!("Returns whether `", stringify!($name), "` initializes at the given block.")]
            #[must_use]
            pub fn is_initialize_block(
                &self,
                settings: &atipicial_config::ProtocolSettings,
                index: u32,
            ) -> (bool, Vec<atipicial_config::Hardfork>) {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::is_initialize_block(self, settings, index)
            }

            #[doc = concat!("Builds the provider-free contract state for `", stringify!($name), "`.")]
            #[must_use]
            pub fn contract_state(
                &self,
                settings: &atipicial_config::ProtocolSettings,
                block_height: u32,
            ) -> Option<atipicial_execution::ContractState> {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::contract_state(self, settings, block_height)
            }

            #[doc = concat!("Returns the native method metadata for `", stringify!($name), "`.")]
            #[must_use]
            pub fn methods(&self) -> &[atipicial_execution::NativeMethod] {
                <Self as atipicial_execution::NativeContract<
                    atipicial_execution::native_contract_provider::NoNativeContractProvider,
                >>::methods(self)
            }

            #[doc = concat!("Returns the ", $contract_name, " script hash.")]
            #[must_use]
            pub fn script_hash() -> atipicial_primitives::UInt160 {
                *($hash)
            }
        }
    };
}

macro_rules! native_contract_identity {
    ($contract:ident) => {
        fn id(&self) -> i32 {
            $contract::ID
        }

        fn hash(&self) -> atipicial_primitives::UInt160 {
            $contract::script_hash()
        }

        fn name(&self) -> &str {
            $contract::NAME
        }
    };
}

macro_rules! native_contract_dispatch {
    ($module:ident :: $bindings:ident) => {
        fn invoke<D, B>(
            &self,
            engine: &mut atipicial_execution::ApplicationEngine<P, D, B>,
            method: &str,
            args: &[Vec<u8>],
        ) -> atipicial_error::CoreResult<Vec<u8>>
        where
            D: atipicial_execution::Diagnostic + 'static,
            B: atipicial_storage::CacheRead,
        {
            let bindings = $module::$bindings::<P, D, B>();
            crate::support::invoke::dispatch_by_name(self, &bindings, engine, method, args)
                .unwrap_or_else(|| {
                    Err(atipicial_error::CoreError::invalid_operation(format!(
                        "{} method '{}({})' is not implemented",
                        <Self as atipicial_execution::NativeContract<P>>::name(self),
                        method,
                        args.len()
                    )))
                })
        }

        fn invoke_resolved<D, B>(
            &self,
            engine: &mut atipicial_execution::ApplicationEngine<P, D, B>,
            method_index: usize,
            method: &atipicial_execution::NativeMethod,
            args: &[Vec<u8>],
        ) -> atipicial_error::CoreResult<Vec<u8>>
        where
            D: atipicial_execution::Diagnostic + 'static,
            B: atipicial_storage::CacheRead,
        {
            let bindings = $module::$bindings::<P, D, B>();
            crate::support::invoke::dispatch_by_index(self, &bindings, engine, method_index, args)
                .unwrap_or_else(|| {
                    Err(atipicial_error::CoreError::invalid_operation(format!(
                        "{} method '{}({})' is not implemented",
                        <Self as atipicial_execution::NativeContract<P>>::name(self),
                        method.name,
                        args.len()
                    )))
                })
        }
    };

    ($module:ident :: $bindings:ident, by_name_and_arity) => {
        fn invoke<D, B>(
            &self,
            engine: &mut atipicial_execution::ApplicationEngine<P, D, B>,
            method: &str,
            args: &[Vec<u8>],
        ) -> atipicial_error::CoreResult<Vec<u8>>
        where
            D: atipicial_execution::Diagnostic + 'static,
            B: atipicial_storage::CacheRead,
        {
            let bindings = $module::$bindings::<P, D, B>();
            crate::support::invoke::dispatch_by_name_and_arity(
                self, &bindings, engine, method, args,
            )
            .unwrap_or_else(|| {
                Err(atipicial_error::CoreError::invalid_operation(format!(
                    "{} method '{}({})' is not implemented",
                    <Self as atipicial_execution::NativeContract<P>>::name(self),
                    method,
                    args.len()
                )))
            })
        }

        fn invoke_resolved<D, B>(
            &self,
            engine: &mut atipicial_execution::ApplicationEngine<P, D, B>,
            method_index: usize,
            method: &atipicial_execution::NativeMethod,
            args: &[Vec<u8>],
        ) -> atipicial_error::CoreResult<Vec<u8>>
        where
            D: atipicial_execution::Diagnostic + 'static,
            B: atipicial_storage::CacheRead,
        {
            let bindings = $module::$bindings::<P, D, B>();
            crate::support::invoke::dispatch_by_index(self, &bindings, engine, method_index, args)
                .unwrap_or_else(|| {
                    Err(atipicial_error::CoreError::invalid_operation(format!(
                        "{} method '{}({})' is not implemented",
                        <Self as atipicial_execution::NativeContract<P>>::name(self),
                        method.name,
                        args.len()
                    )))
                })
        }
    };

    (
        $module:ident :: $bindings:ident,
        by_name_and_arity,
        resolved_by_index = $resolver_module:ident :: $resolver:ident
    ) => {
        fn invoke<D, B>(
            &self,
            engine: &mut atipicial_execution::ApplicationEngine<P, D, B>,
            method: &str,
            args: &[Vec<u8>],
        ) -> atipicial_error::CoreResult<Vec<u8>>
        where
            D: atipicial_execution::Diagnostic + 'static,
            B: atipicial_storage::CacheRead,
        {
            let bindings = $module::$bindings::<P, D, B>();
            crate::support::invoke::dispatch_by_name_and_arity(
                self, &bindings, engine, method, args,
            )
            .unwrap_or_else(|| {
                Err(atipicial_error::CoreError::invalid_operation(format!(
                    "{} method '{}({})' is not implemented",
                    <Self as atipicial_execution::NativeContract<P>>::name(self),
                    method,
                    args.len()
                )))
            })
        }

        fn invoke_resolved<D, B>(
            &self,
            engine: &mut atipicial_execution::ApplicationEngine<P, D, B>,
            method_index: usize,
            method: &atipicial_execution::NativeMethod,
            args: &[Vec<u8>],
        ) -> atipicial_error::CoreResult<Vec<u8>>
        where
            D: atipicial_execution::Diagnostic + 'static,
            B: atipicial_storage::CacheRead,
        {
            $resolver_module::$resolver::<P, D, B>(self, engine, method_index, args).unwrap_or_else(
                || {
                    Err(atipicial_error::CoreError::invalid_operation(format!(
                        "{} method '{}({})' is not implemented",
                        <Self as atipicial_execution::NativeContract<P>>::name(self),
                        method.name,
                        args.len()
                    )))
                },
            )
        }
    };
}
