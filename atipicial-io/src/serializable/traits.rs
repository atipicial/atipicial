//! Serializable traits and extension helpers for Atipicial binary data.

use crate::{BinaryWriter, IoResult, MemoryReader};

/// Extension helpers for [`Serializable`] values mirroring
/// `Atipicial.Extensions.IO.ISerializableExtensions`.
pub trait SerializableExtensions {
    /// Serializes this value to a byte vector.
    fn to_array(&self) -> IoResult<Vec<u8>>;
}

impl<T: Serializable> SerializableExtensions for T {
    fn to_array(&self) -> IoResult<Vec<u8>> {
        let mut writer = BinaryWriter::with_capacity(self.size());
        self.serialize(&mut writer)?;
        Ok(writer.into_bytes())
    }
}

/// Trait implemented by Atipicial types that can be serialized and deserialized.
///
/// This follows the behaviour of `Atipicial.IO.ISerializable` from the C# codebase.
pub trait Serializable: Sized {
    /// Creates an instance from the provided `MemoryReader`.
    fn deserialize(reader: &mut MemoryReader) -> IoResult<Self>;

    /// Serializes the current value into the provided `BinaryWriter`.
    fn serialize(&self, writer: &mut BinaryWriter) -> IoResult<()>;

    /// Returns the number of bytes the serialized value will consume.
    fn size(&self) -> usize;
}
