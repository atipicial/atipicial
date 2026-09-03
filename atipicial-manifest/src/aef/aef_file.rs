//! `AefFile` — AEF (Atipicial Executable Format) wire container
//! (matches C# `Atipicial.SmartContract.AefFile`).
//!
//! ## Layering
//!
//! Pure data type in **Layer 1 (protocol)**. Depends only on
//! `MethodToken` from the same crate and `atipicial-primitives` /
//! `atipicial-crypto` / `atipicial-io`. The `Serializable` impl lives here
//! too because the on-wire encoding is a pure data concern.

use crate::method_token::MethodToken;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use atipicial_crypto::Crypto;
use atipicial_error::{CoreError, CoreResult};
use atipicial_io::serializable::helper::SerializeHelper;
use atipicial_io::{BinaryWriter, IoError, IoResult, MemoryReader, Serializable};
use serde_json::{Value, json};

/// Represents a AEF (Atipicial Executable Format) file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AefFile {
    /// The compiler used to compile the contract.
    pub compiler: String,

    /// The source code information.
    pub source: String,

    /// The tokens used in the contract.
    pub tokens: Vec<MethodToken>,

    /// The script of the contract.
    pub script: Vec<u8>,

    /// The checksum of the AEF file.
    pub checksum: u32,
}

impl AefFile {
    /// The AEF magic number: `'N', 'E', 'F', 3` in little-endian.
    pub const MAGIC: u32 = 0x3346_454E;
    /// The fixed compiler string length (zero-padded to 64 bytes on the wire).
    pub const COMPILER_LENGTH: usize = 64;
    /// The maximum source string length in bytes.
    pub const MAX_SOURCE_LENGTH: usize = 256;
    /// The maximum number of tokens in a AEF file.
    pub const MAX_TOKENS: usize = 128;

    /// Creates a new AEF file with the computed checksum.
    pub fn new(compiler: String, script: Vec<u8>) -> Self {
        let mut aef = Self {
            compiler,
            source: String::new(),
            tokens: Vec::new(),
            script,
            checksum: 0,
        };
        aef.checksum = Self::compute_checksum(&aef);
        aef
    }

    /// Gets the size of the AEF file in bytes (matches the
    /// on-wire serialised length).
    pub fn size(&self) -> usize {
        4 + // Magic (u32)
        Self::COMPILER_LENGTH + // Compiler fixed string (64 bytes)
        SerializeHelper::get_var_size_str(&self.source) + // Source var string
        1 + // Reserved byte
        SerializeHelper::get_var_size_serializable_slice(&self.tokens) + // Tokens array
        2 + // Reserved bytes (u16)
        SerializeHelper::get_var_size_bytes(&self.script) + // Script var bytes
        4 // Checksum (u32)
    }

    /// Computes the AEF checksum using the C# algorithm:
    /// `Hash256(aef_bytes_without_checksum)[..4]` interpreted as
    /// little-endian u32.
    pub fn compute_checksum(aef: &Self) -> u32 {
        match Self::try_compute_checksum(aef) {
            Ok(checksum) => checksum,
            Err(err) => {
                tracing::warn!(
                    target: "atipicial.manifest",
                    error = %err,
                    "failed to compute AEF checksum"
                );
                0
            }
        }
    }

    /// Computes the AEF checksum using a fallible wire writer.
    pub fn try_compute_checksum(aef: &Self) -> IoResult<u32> {
        let mut writer = BinaryWriter::new();
        Self::write_without_checksum(aef, &mut writer)?;

        let bytes = writer.into_bytes();
        let hash = Crypto::hash256(&bytes);
        Ok(u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]))
    }

    fn write_without_checksum(aef: &Self, writer: &mut BinaryWriter) -> IoResult<()> {
        writer.write_u32(Self::MAGIC)?;
        let compiler_bytes = aef.compiler.as_bytes();
        let mut fixed = [0u8; Self::COMPILER_LENGTH];
        let len = compiler_bytes.len().min(Self::COMPILER_LENGTH);
        fixed[..len].copy_from_slice(&compiler_bytes[..len]);
        writer.write_bytes(&fixed)?;

        writer.write_var_string(&aef.source)?;
        writer.write_u8(0)?; // reserved
        writer.write_serializable_vec(&aef.tokens)?;
        writer.write_u16(0)?; // reserved
        writer.write_var_bytes(&aef.script)?;
        Ok(())
    }

    /// Recomputes and updates the checksum in-place.
    pub fn update_checksum(&mut self) {
        self.checksum = Self::compute_checksum(self);
    }

    /// Recomputes and updates the checksum in-place using the fallible writer.
    pub fn try_update_checksum(&mut self) -> IoResult<()> {
        self.checksum = Self::try_compute_checksum(self)?;
        Ok(())
    }

    /// Converts the AEF file to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.try_to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::warn!(
                    target: "atipicial.manifest",
                    error = %err,
                    "failed to serialize AEF file"
                );
                Vec::new()
            }
        }
    }

    /// Converts the AEF file to bytes using the fallible wire writer.
    pub fn try_to_bytes(&self) -> IoResult<Vec<u8>> {
        let mut writer = BinaryWriter::new();
        Self::write_without_checksum(self, &mut writer)?;
        writer.write_u32(self.checksum)?;

        Ok(writer.into_bytes())
    }

    /// Parses a AEF file from bytes.
    pub fn from_bytes(bytes: &[u8]) -> CoreResult<Self> {
        let mut reader = MemoryReader::new(bytes);
        Self::deserialize(&mut reader).map_err(|e| CoreError::other(e.to_string()))
    }

    /// Parses a AEF file from bytes (alias for [`Self::from_bytes`]).
    pub fn parse(bytes: &[u8]) -> CoreResult<Self> {
        Self::from_bytes(bytes)
    }

    /// Converts the AEF file to base64-encoded bytes.
    pub fn to_base64(&self) -> String {
        BASE64_STANDARD.encode(self.to_bytes())
    }

    /// Parses a AEF file from base64-encoded bytes.
    pub fn from_base64(base64: &str) -> CoreResult<Self> {
        let bytes = BASE64_STANDARD
            .decode(base64)
            .map_err(|e| CoreError::other(e.to_string()))?;
        Self::from_bytes(&bytes)
    }

    /// Converts to JSON representation.
    pub fn to_json(&self) -> Value {
        json!({
            "magic": Self::MAGIC,
            "compiler": self.compiler,
            "source": self.source,
            "tokens": self.tokens,
            "script": BASE64_STANDARD.encode(&self.script),
            "checksum": self.checksum,
        })
    }
}

impl Serializable for AefFile {
    fn size(&self) -> usize {
        self.size()
    }

    fn serialize(&self, writer: &mut BinaryWriter) -> IoResult<()> {
        if self.compiler.len() > Self::COMPILER_LENGTH {
            return Err(IoError::invalid_data(format!(
                "Compiler name too long: {} > {}",
                self.compiler.len(),
                Self::COMPILER_LENGTH
            )));
        }
        if self.source.len() > Self::MAX_SOURCE_LENGTH {
            return Err(IoError::invalid_data(format!(
                "Source too long: {} > {}",
                self.source.len(),
                Self::MAX_SOURCE_LENGTH
            )));
        }
        if self.tokens.len() > Self::MAX_TOKENS {
            return Err(IoError::invalid_data(format!(
                "Too many tokens: {} > {}",
                self.tokens.len(),
                Self::MAX_TOKENS
            )));
        }
        // Check script length is within the var-int encoding range
        // (matches the on-wire AEF3 format).
        if self.script.len() > u32::MAX as usize {
            return Err(IoError::invalid_data("Script too long for AEF format"));
        }
        // Also enforce the Atipicial VM item-size cap (matches
        // atipicial-vm::ExecutionEngineLimits::max_item_size).
        let max_item_size = atipicial_vm::ExecutionEngineLimits::DEFAULT.max_item_size as usize;
        if self.script.len() > max_item_size {
            return Err(IoError::invalid_data(format!(
                "Script exceeds max item size: {} > {}",
                self.script.len(),
                max_item_size
            )));
        }
        Self::write_without_checksum(self, writer)?;
        writer.write_u32(self.checksum)?;
        Ok(())
    }

    fn deserialize(reader: &mut MemoryReader) -> IoResult<Self> {
        let start_position = reader.position();
        let magic = reader.read_u32()?;
        if magic != Self::MAGIC {
            return Err(IoError::invalid_data(format!(
                "Bad magic: {magic:#x}, expected {:#x}",
                Self::MAGIC
            )));
        }

        let compiler_bytes = reader.read_bytes(Self::COMPILER_LENGTH)?;
        let compiler_end = compiler_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(compiler_bytes.len());
        let compiler = String::from_utf8(compiler_bytes[..compiler_end].to_vec())
            .map_err(|e| IoError::invalid_data(format!("Invalid compiler UTF-8: {e}")))?;

        let source = reader.read_var_string(Self::MAX_SOURCE_LENGTH)?;

        let _reserved = reader.read_u8()?;
        let tokens = SerializeHelper::deserialize_array::<MethodToken>(reader, Self::MAX_TOKENS)
            .map_err(|e| IoError::invalid_data(e.to_string()))?;
        let _reserved2 = reader.read_u16()?;
        // C# AefFile.Deserialize reads the script capped at MaxItemSize and
        // rejects an empty script.
        let max_item_size = atipicial_vm::ExecutionEngineLimits::DEFAULT.max_item_size as usize;
        let script = reader.read_var_bytes(max_item_size)?;
        if script.is_empty() {
            return Err(IoError::invalid_data("Script cannot be empty."));
        }
        let checksum = reader.read_u32()?;

        let aef = AefFile {
            compiler,
            source,
            tokens,
            script,
            checksum,
        };

        // Validate the on-wire checksum (matches C# AEF verifier).
        let expected = Self::try_compute_checksum(&aef)?;
        if expected != aef.checksum {
            return Err(IoError::invalid_data(format!(
                "Bad checksum: {:#x}, expected {:#x}",
                aef.checksum, expected
            )));
        }

        // C# AefFile.Deserialize verify: total deserialized size must not
        // exceed MaxItemSize.
        let consumed = reader.position().saturating_sub(start_position);
        if consumed > max_item_size {
            return Err(IoError::invalid_data("Max vm item size exceed"));
        }

        Ok(aef)
    }
}

#[cfg(test)]
#[path = "../tests/aef/aef_file.rs"]
mod tests;
