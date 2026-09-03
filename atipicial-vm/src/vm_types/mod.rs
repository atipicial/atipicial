//! # AtipicialVM Runtime Types
//!
//! Canonical Atipicial N3 bytecode, execution-state, and VM metadata types.
//!
//! ## Boundary
//!
//! This module owns VM-level primitives shared by script decoding and
//! execution. Mutable runtime values remain owned by `crate::stack_item`.
//!
//! ## Contents
//!
//! Opcode and instruction metadata, execution limits and state, strict script
//! validation, exception contexts, syscall metadata, and small VM collections.

mod collections;
mod exception_handling;
mod identity;
mod instruction;
mod limits;
mod opcode;
mod script_validation;
mod stack_item_type;
mod state;
mod syscall;

pub use collections::{VmOrderedDictionary, encode_integer};
pub use exception_handling::{ExceptionHandlingContext, ExceptionHandlingState};
pub use identity::next_stack_item_id;
pub use instruction::{
    FromOperand, Instruction, InstructionError, InstructionErrorKind, InstructionResult,
};
pub use limits::{
    DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, ExecutionEngineLimits, MAX_ITEM_SIZE,
    MAX_SCRIPT_SIZE,
};
pub use opcode::OpCode;
pub use script_validation::{
    ScriptInstruction, ValidatedScript, ValidationResult, instruction_jump_target,
    instruction_try_targets, parse_script_instructions, validate_script, validate_strict_script,
};
pub use stack_item_type::{
    ATCVM_STACK_ITEM_TYPE_ANY, ATCVM_STACK_ITEM_TYPE_ARRAY, ATCVM_STACK_ITEM_TYPE_BOOLEAN,
    ATCVM_STACK_ITEM_TYPE_BUFFER, ATCVM_STACK_ITEM_TYPE_BYTESTRING, ATCVM_STACK_ITEM_TYPE_INTEGER,
    ATCVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, ATCVM_STACK_ITEM_TYPE_MAP,
    ATCVM_STACK_ITEM_TYPE_POINTER, ATCVM_STACK_ITEM_TYPE_STRUCT, StackItemType,
};
pub use state::VmState;
pub use syscall::{interop_hash, syscall_arg_count};
