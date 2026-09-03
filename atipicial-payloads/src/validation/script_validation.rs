//! Strict VM script validation helpers re-exported from `atipicial-vm`.

pub use atipicial_vm::{
    ScriptInstruction, ValidatedScript, ValidationResult, parse_script_instructions,
    validate_script, validate_strict_script,
};
