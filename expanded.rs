#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::{marker::PhantomData, ops::Deref};
pub enum PedanticError {
    ParsingFailedError,
}
pub trait Validator<T> {
    fn validate(value: &T) -> Result<(), PedanticError>;
}
impl<T, V1, V2> Validator<T> for (V1, V2)
where
    V1: Validator<T>,
    V2: Validator<T>,
{
    fn validate(value: &T) -> Result<(), PedanticError> {
        V1::validate(value)?;
        V2::validate(value)?;
        Ok(())
    }
}
pub struct Refined<T, V>
where
    V: Validator<T>,
{
    data: T,
    validators: PhantomData<V>,
}
impl<T, V> Refined<T, V>
where
    V: Validator<T>,
{
    pub fn parse(value: T) -> Result<Self, PedanticError> {
        V::validate(&value)?;
        Ok(Self {
            data: value,
            validators: PhantomData,
        })
    }
}
pub mod validators {
    use crate::{PedanticError, Validator};
    use paste::paste;
    pub struct LessThanU8<const MAX: u8>;
    impl<const MAX: u8> Validator<u8> for LessThanU8<MAX> {
        fn validate(value: &u8) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanU16<const MAX: u16>;
    impl<const MAX: u16> Validator<u16> for LessThanU16<MAX> {
        fn validate(value: &u16) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanU32<const MAX: u32>;
    impl<const MAX: u32> Validator<u32> for LessThanU32<MAX> {
        fn validate(value: &u32) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanU64<const MAX: u64>;
    impl<const MAX: u64> Validator<u64> for LessThanU64<MAX> {
        fn validate(value: &u64) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanU128<const MAX: u128>;
    impl<const MAX: u128> Validator<u128> for LessThanU128<MAX> {
        fn validate(value: &u128) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanI8<const MAX: i8>;
    impl<const MAX: i8> Validator<i8> for LessThanI8<MAX> {
        fn validate(value: &i8) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanI16<const MAX: i16>;
    impl<const MAX: i16> Validator<i16> for LessThanI16<MAX> {
        fn validate(value: &i16) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanI32<const MAX: i32>;
    impl<const MAX: i32> Validator<i32> for LessThanI32<MAX> {
        fn validate(value: &i32) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanI64<const MAX: i64>;
    impl<const MAX: i64> Validator<i64> for LessThanI64<MAX> {
        fn validate(value: &i64) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
    pub struct LessThanI128<const MAX: i128>;
    impl<const MAX: i128> Validator<i128> for LessThanI128<MAX> {
        fn validate(value: &i128) -> Result<(), crate::PedanticError> {
            if *value > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }
}
