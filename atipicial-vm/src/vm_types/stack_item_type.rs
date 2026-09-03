//! Canonical AtipicialVM stack-item type tags.

/// AtipicialVM `StackItemType.Any`.
pub const ATCVM_STACK_ITEM_TYPE_ANY: u8 = 0x00;
/// AtipicialVM `StackItemType.Pointer`.
pub const ATCVM_STACK_ITEM_TYPE_POINTER: u8 = 0x10;
/// AtipicialVM `StackItemType.Boolean`.
pub const ATCVM_STACK_ITEM_TYPE_BOOLEAN: u8 = 0x20;
/// AtipicialVM `StackItemType.Integer`.
pub const ATCVM_STACK_ITEM_TYPE_INTEGER: u8 = 0x21;
/// AtipicialVM `StackItemType.ByteString`.
pub const ATCVM_STACK_ITEM_TYPE_BYTESTRING: u8 = 0x28;
/// AtipicialVM `StackItemType.Buffer`.
pub const ATCVM_STACK_ITEM_TYPE_BUFFER: u8 = 0x30;
/// AtipicialVM `StackItemType.Array`.
pub const ATCVM_STACK_ITEM_TYPE_ARRAY: u8 = 0x40;
/// AtipicialVM `StackItemType.Struct`.
pub const ATCVM_STACK_ITEM_TYPE_STRUCT: u8 = 0x41;
/// AtipicialVM `StackItemType.Map`.
pub const ATCVM_STACK_ITEM_TYPE_MAP: u8 = 0x48;
/// AtipicialVM `StackItemType.InteropInterface`.
pub const ATCVM_STACK_ITEM_TYPE_INTEROP_INTERFACE: u8 = 0x60;

/// C# Atipicial.VM stack-item type tags.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackItemType {
    /// Represents any type.
    Any = ATCVM_STACK_ITEM_TYPE_ANY,
    /// Represents a code pointer.
    Pointer = ATCVM_STACK_ITEM_TYPE_POINTER,
    /// Represents a boolean value.
    Boolean = ATCVM_STACK_ITEM_TYPE_BOOLEAN,
    /// Represents an integer value.
    Integer = ATCVM_STACK_ITEM_TYPE_INTEGER,
    /// Represents an immutable byte sequence.
    ByteString = ATCVM_STACK_ITEM_TYPE_BYTESTRING,
    /// Represents a mutable byte sequence.
    Buffer = ATCVM_STACK_ITEM_TYPE_BUFFER,
    /// Represents an array.
    Array = ATCVM_STACK_ITEM_TYPE_ARRAY,
    /// Represents a structure.
    Struct = ATCVM_STACK_ITEM_TYPE_STRUCT,
    /// Represents an ordered key-value map.
    Map = ATCVM_STACK_ITEM_TYPE_MAP,
    /// Represents a host interop interface.
    InteropInterface = ATCVM_STACK_ITEM_TYPE_INTEROP_INTERFACE,
}

impl StackItemType {
    /// Decodes a C# Atipicial.VM stack-item type byte.
    #[must_use]
    pub const fn from_byte(value: u8) -> Option<Self> {
        match value {
            ATCVM_STACK_ITEM_TYPE_ANY => Some(Self::Any),
            ATCVM_STACK_ITEM_TYPE_POINTER => Some(Self::Pointer),
            ATCVM_STACK_ITEM_TYPE_BOOLEAN => Some(Self::Boolean),
            ATCVM_STACK_ITEM_TYPE_INTEGER => Some(Self::Integer),
            ATCVM_STACK_ITEM_TYPE_BYTESTRING => Some(Self::ByteString),
            ATCVM_STACK_ITEM_TYPE_BUFFER => Some(Self::Buffer),
            ATCVM_STACK_ITEM_TYPE_ARRAY => Some(Self::Array),
            ATCVM_STACK_ITEM_TYPE_STRUCT => Some(Self::Struct),
            ATCVM_STACK_ITEM_TYPE_MAP => Some(Self::Map),
            ATCVM_STACK_ITEM_TYPE_INTEROP_INTERFACE => Some(Self::InteropInterface),
            _ => None,
        }
    }

    /// Returns the C# Atipicial.VM stack-item type byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        self as u8
    }

    /// Returns the canonical AtipicialVM stack-item type name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::Pointer => "Pointer",
            Self::Boolean => "Boolean",
            Self::Integer => "Integer",
            Self::ByteString => "ByteString",
            Self::Buffer => "Buffer",
            Self::Array => "Array",
            Self::Struct => "Struct",
            Self::Map => "Map",
            Self::InteropInterface => "InteropInterface",
        }
    }
}
