// Copyright (c) 2024 R3E Network
// This file is part of the atipicial-rs project
// Licensed under the MIT License
// See LICENSE file for details

//! # atipicial-vm
//!
//! AtipicialVM execution engine, opcode dispatch, stack items, and runtime types.
//!
//! ## Boundary
//!
//! This VM crate owns deterministic script execution and must not own ledger
//! persistence, network transport, or node composition.
//!
//! ## Contents
//!
//! - `types`: Storage-domain types shared by store implementations.
//! - `script_builder`: Helpers for constructing AtipicialVM scripts
//!   deterministically.
//! - `runtime`: Runtime flags, execution context state, and VM-facing support
//!   types.
//! - `execution_context`: AtipicialVM execution context frames and instruction-
//!   pointer state.
//! - `execution_engine`: AtipicialVM execution engine loop and runtime state.
//! - `jump_table`: Opcode dispatch tables and instruction implementations.
//! - `stack_item`: AtipicialVM stack item representations and operations.

extern crate alloc;

// ============================================================================
// Core VM Modules
// ============================================================================

/// VM error types and result handling.
mod types;
pub use types::error;

mod vm_types;

mod execution_profile;
pub use execution_profile::{
    OpcodeClass, ScriptEntryProfile, ScriptExecutionProfile, StackOperationProfile,
    VmExecutionProfile,
};

mod execution_plan;
pub use execution_plan::{
    ArgumentContract, BasicBlock, ByteExpression, ByteExpressionSegment, CallContract,
    CandidateAuthority, CandidateContract, CandidateContractError, CandidateContractLimits,
    CandidateContractParts, CandidateId, CandidateIdentity, CandidateVersion, ContextDependency,
    ContextScriptHash, ContractResolutionIdentity, ContractTarget, EffectContract, ExecutionPlan,
    ExecutionPlanBuildError, ExecutionPlanCache, ExecutionPlanCacheError, ExecutionPlanCacheLimits,
    ExecutionPlanCacheSnapshot, ExecutionPlanKey, ExecutionPlanKeyVersion, ExecutionPlanLimits,
    ExecutionPlanRoute, FaultClass, FaultContract, FaultEffectDisposition, GasAmount,
    GasStepContract, HardforkPlanState, HardforkTableIdentity, HostEffectContract,
    InstructionCount, InvocationEligibility, NativeCacheDependency, NativeCacheDomain,
    NativeCacheScope, PlannedControlFlow, PlannedInstruction, PointStateDependency,
    ProtocolIdentity, ProtocolVersion, RangeDirection, RangeDomain, RangeStateDependency,
    ReadRequirement, RegistryBuildError, RegistrySnapshot, SlotContract, SlotSource,
    SpecializationContractVersion, SpecializationMode, SpecializationRegistry,
    SpecializationRegistryLimits, SpecializationSelection, StackEffectContract,
    StackItemConstraint, StackItemEligibility, StackItemShape, StateDependencyContract,
    StorageTarget, StorageWriteKind,
};

/// Script builder for programmatic VM script construction.
pub mod script_builder;

/// Type-safe evaluation stack implementation.
///
/// The [`EvaluationStack`] is the primary operand stack for VM operations.
/// It provides type-safe operations and automatic reference counting.
mod runtime;
pub use runtime::evaluation_stack;

/// Script execution context with local variables.
///
/// Each [`ExecutionContext`] represents a call frame with:
/// - Instruction pointer
/// - Evaluation stack
/// - Local variables
/// - Static fields
pub mod execution_context;

/// Core VM execution engine.
///
/// The [`ExecutionEngine`] is the main VM that:
/// - Executes scripts
/// - Manages the context stack
/// - Handles the instruction cycle
/// - Tracks gas consumption
pub mod execution_engine;

/// Interoperable trait for smart contract state round-tripping.
pub use runtime::interoperable;

/// Interop service registry.
///
/// [`InteropService`] manages native contract methods accessible via SYSCALL.
pub use runtime::interop_service;

/// Stateful opcode dispatch and Atipicial N3 instruction implementations.
pub mod jump_table;

/// Reference counting for garbage collection.
pub use runtime::reference_counter;

/// VM script representation and validation.
pub use types::script;

/// JSON-RPC envelope rendering for VM stack items.
pub use types::rpc_json;

/// Slot storage for locals, arguments, and static fields.
pub use runtime::slot;

/// Stateful, reference-counted stack items used by the local execution engine.
pub mod stack_item;

// ============================================================================
// Canonical VM primitives
//
// `atipicial-vm` is the workspace's only VM boundary. Consumers must not couple
// themselves to another interpreter or runtime value model.
// ============================================================================

pub use vm_types::{
    DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, ExceptionHandlingContext,
    ExceptionHandlingState, ExecutionEngineLimits, FromOperand, Instruction, InstructionError,
    InstructionErrorKind, InstructionResult, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE,
    ATCVM_STACK_ITEM_TYPE_ANY, ATCVM_STACK_ITEM_TYPE_ARRAY, ATCVM_STACK_ITEM_TYPE_BOOLEAN,
    ATCVM_STACK_ITEM_TYPE_BUFFER, ATCVM_STACK_ITEM_TYPE_BYTESTRING, ATCVM_STACK_ITEM_TYPE_INTEGER,
    ATCVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, ATCVM_STACK_ITEM_TYPE_MAP,
    ATCVM_STACK_ITEM_TYPE_POINTER, ATCVM_STACK_ITEM_TYPE_STRUCT, OpCode, ScriptInstruction,
    StackItemType, ValidatedScript, ValidationResult, VmOrderedDictionary, VmState, encode_integer,
    instruction_jump_target, instruction_try_targets, interop_hash, next_stack_item_id,
    parse_script_instructions, syscall_arg_count, validate_script, validate_strict_script,
};

// ============================================================================
// Public Re-exports from the local VM host (stateful types)
// ============================================================================

pub use error::{VmError, VmResult};
pub use execution_context::ExecutionContext;
pub use execution_engine::ExecutionEngine;
pub use jump_table::JumpTable;
pub use runtime::{
    CompoundId, EvaluationStack, InteropService, Interoperable, InteroperableError,
    ReferenceCounter, Slot,
};
pub use stack_item::{InteropInterface, StackItem};
pub use types::rpc_json::StackItemRpcJson;
pub use types::script::Script;

/// Verification contract (script + parameter list + cached hash).
pub use types::contract::Contract;

#[cfg(test)]
#[path = "tests/differential/csharp.rs"]
mod csharp_differential_tests;

// ============================================================================
// I/O Abstraction
// ============================================================================

/// Production I/O implementation.
pub use atipicial_io as io;
