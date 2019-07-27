use super::Node;
use crate::{error::Error, lexer::Token};
use serde::{Serialize, Serializer};

/// Signed and unsigned integer wrapper
#[derive(Clone, Copy, Debug, From, Hash, PartialEq, Eq)]
pub enum Int {
    /// `u8`
    U8(u8),

    /// `u16`
    U16(u16),

    /// `u32`
    U32(u32),

    /// `u64`
    U64(u64),

    /// `u128`
    U128(u128),

    /// `i8`
    I8(i8),

    /// `i16`
    I16(i16),

    /// `i32`
    I32(i32),

    /// `i64`
    I64(i64),

    /// `i128`
    I128(i128),
}

impl<'a> Node<'a> for Int {
    #[inline]
    fn kind(&self) -> Token {
        Token::Integer(*self)
    }

    #[inline]
    fn as_int(&self) -> Option<Int> {
        Some(*self)
    }
}

impl Serialize for Int {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Int::U8(num) => serializer.serialize_u8(num),
            Int::U16(num) => serializer.serialize_u16(num),
            Int::U32(num) => serializer.serialize_u32(num),
            Int::U64(num) => serializer.serialize_u64(num),
            Int::U128(num) => serializer.serialize_u128(num),
            Int::I8(num) => serializer.serialize_i8(num),
            Int::I16(num) => serializer.serialize_i16(num),
            Int::I32(num) => serializer.serialize_i32(num),
            Int::I64(num) => serializer.serialize_i64(num),
            Int::I128(num) => serializer.serialize_i128(num),
        }
    }
}

impl std::str::FromStr for Int {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!();
    }
}
