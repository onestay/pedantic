#![warn(missing_docs)]

//! # Pedantic
//! Pedantic is a simple, lightweight framework for creating checked types.
//!
//! At the core of Pedantic is the [Refined] struct which wraps any type
//! and one or more [Validators](Validator).
//!
//! One [Refined] type can have multiple validators by declaring them using a tuple.
//! The validators in the tuple are then evaluated from left to right.
//! If any one of the validators fail, parsing of the type fails.
//!
//! This crate provides the a number of common validators in the [validators] module. This
//! includes:
//! - LessThan/LessEqual for all integers
//! - GreaterThan/GreatherEqual for all integer
//! - [MaxLength](validators::MaxLength)/[MinLength](validators::MinLength) for Strings
//! - AsciiString/HexString for Strings
//!
//! as well as a simple way to create your own regex validators using the [pattern] macro.
//!
//! # Example
//!
//! ```
//! use pedantic::Refined;
//! use pedantic::validators::*;
//!
//! type MyU32 = Refined<u32, (GreaterThanU32<10>, LessThanU32<15>)>;
//!
//! let my_valid_u32 = MyU32::parse(12);
//! assert!(my_valid_u32.is_ok());
//!
//! let my_big_u32 = MyU32::parse(17);
//! assert!(my_big_u32.is_err());
//! ```
//!
//! # Limitations
//! The [Validator] trait is not dyn compatible and does not take an &self
//! which might make writing more complicated validators hard.
//! This is however not a goal of this library. Consider wrapping a [Refined]
//! type in your own type if you need more complicated logic.
//!
//! There currently is no way to `OR` validators.
use std::marker::PhantomData;
use regex::Regex;

pub enum PedanticError {
    ParsingFailedError,
}

/// The main trait for validation.
///
/// This trait is deliberately not object safe and doesn't take a `&self` as it's argument.
/// This is done to keep the `type` declarations clean and declarative.
pub trait Validator<T> {
    fn validate(value: &T) -> Result<(), PedanticError>;
}

pub trait PatternValidator {
    const PATTERN: &'static str;
    fn regex() -> &'static Regex;
}

impl<T: AsRef<str>, U: PatternValidator> Validator<T> for U {
    fn validate(value: &T) -> Result<(), PedanticError> {
        let regex = U::regex();
        if !regex.is_match(value.as_ref()) {
            return Err(PedanticError::ParsingFailedError);
        }

        Ok(())
    }
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

impl<T, V1, V2, V3> Validator<T> for (V1, V2, V3)
where
    V1: Validator<T>,
    V2: Validator<T>,
    V3: Validator<T>,
{
    fn validate(value: &T) -> Result<(), PedanticError> {
        V1::validate(value)?;
        V2::validate(value)?;
        V3::validate(value)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
pub struct Refined<T, V: Validator<T>> {
    data: T,
    validators: PhantomData<V>,
}

impl<T: std::fmt::Display, V: Validator<T>> std::fmt::Display for Refined<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
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

impl<T, V> AsRef<T> for Refined<T, V>
where
    V: Validator<T>,
{
    fn as_ref(&self) -> &T {
        &self.data
    }
}

pub mod validators {
    use crate::{PedanticError, Validator, PatternValidator};

    use std::sync::LazyLock;
    use paste::paste;
    use regex::Regex;

    /// Create custom regex validators using this macro.
    ///
    /// The name of the validator is the first option and the regex the second.
    /// # Example
    /// ```
    /// use pedantic::pattern;
    /// use pedantic::PatternValidator;
    /// use regex::Regex;
    /// use std::sync::LazyLock;
    ///
    /// pattern!(MyCoolRegexValidator, r"^[b-chm-pP]at|ot$");
    /// ```
    #[macro_export]
    macro_rules! pattern {
        ($name:ident, $regex:expr) => {
            pub struct $name;

            impl PatternValidator for $name {
                const PATTERN: &'static str = $regex;

                fn regex() -> &'static Regex {
                    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
                        Regex::new(<$name as PatternValidator>::PATTERN).expect("invalid regex")
                    });
                    &REGEX
                }
            }
        };
    }

    pattern!(AsciiString, r"^[[:ascii:]]*$");
    pattern!(HexString, r"^[\da-fA-f]*$");

    pub struct MinLength<const MIN: usize>;
    impl<const MIN: usize, T: AsRef<str>> Validator<T> for MinLength<MIN> {
        fn validate(value: &T) -> Result<(), PedanticError> {
            if value.as_ref().len() < MIN {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }

    pub struct MaxLength<const MAX: usize>;
    impl<const MAX: usize, T: AsRef<str>> Validator<T> for MaxLength<MAX> {
        fn validate(value: &T) -> Result<(), PedanticError> {
            if value.as_ref().len() > MAX {
                return Err(PedanticError::ParsingFailedError);
            }
            Ok(())
        }
    }

    macro_rules! int_comp {
        ($base_name:ident $generic_name:ident $op:tt) => {
            int_comp!(@impl $base_name $generic_name $op u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);
        };

        (@impl $base_name:ident $generic_name:ident $op:tt $( $type:ty ) +) => {
            paste! {
                $(
                    #[doc = concat!("Compare the given [", stringify!($type) ,"] with the constant `", stringify!($generic_name), "` using the `", stringify!($op), "` operation.")]
                    #[doc = ""]
                    #[doc = "Check the crate level documentation for more information on usage."]
                    pub struct [<$base_name $type:upper>]<const $generic_name: $type>;
                    impl<const $generic_name: $type> Validator<$type> for [<$base_name $type:upper>]<$generic_name> {
                        fn validate(value: &$type) -> Result<(), $crate::PedanticError> {
                            if *value $op $generic_name {
                                return Err(PedanticError::ParsingFailedError);
                            }
                            Ok(())
                        }
                    }
                )+
            }
        };
    }

    int_comp!(GreaterThan MIN <=);
    int_comp!(GreaterEqual MIN >);
    int_comp!(LessThan MAX >=);
    int_comp!(LessEqual MAX >);
}

#[cfg(test)]
mod test {
    use crate::*;
    use crate::validators::*;
    use rstest::rstest;

    const U32_TEST_VALUE: u32 = 20;

    #[rstest]
    #[case(5, true)]
    #[case(20, false)]
    #[case(21, false)]
    fn less_than_u32(#[case] value: u32, #[case] is_ok: bool) {
        type TestType = Refined<u32, LessThanU32<U32_TEST_VALUE>>;

        let a = TestType::parse(value);
        assert_eq!(a.is_ok(), is_ok);
    }

    #[rstest]
    #[case(U32_TEST_VALUE, false)]
    #[case(U32_TEST_VALUE + 1, true)]
    #[case(U32_TEST_VALUE - 1, false)]
    fn greater_than_u32(#[case] value: u32, #[case] is_ok: bool) {
        type TestType = Refined<u32, GreaterThanU32<U32_TEST_VALUE>>;

        let a = TestType::parse(value);
        assert_eq!(a.is_ok(), is_ok);
    }

    const I128_TEST_MAX: i128 = 328_123_123_122;

    #[rstest]
    #[case(I128_TEST_MAX, false)]
    #[case(I128_TEST_MAX - 1, true)]
    #[case(I128_TEST_MAX + 1, false)]
    fn less_than_i128(#[case] value: i128, #[case] is_ok: bool) {
        type TestType = Refined<i128, LessThanI128<I128_TEST_MAX>>;
        let a = TestType::parse(value);
        assert_eq!(a.is_ok(), is_ok);
    }

    #[test]
    fn combined_gt_lt() {
        type TestType = Refined<u32, (GreaterThanU32<10>, LessThanU32<15>)>;

        let res = TestType::parse(12);
        assert!(res.is_ok());

        let res = TestType::parse(10);
        assert!(res.is_err());

        let res = TestType::parse(15);
        assert!(res.is_err());
    }

    #[rstest]
    #[case("123", false)]
    #[case("123456789a", true)]
    #[case("123456789abc", true)]
    fn string_length_min(#[case] value: &'static str, #[case] is_ok: bool) {
        type TestType = Refined<String, MinLength<10>>;
        let res = TestType::parse(value.to_string());
        assert_eq!(res.is_ok(), is_ok);
    }

    #[rstest]
    #[case("123", true)]
    #[case("123456789a", true)]
    #[case("123456789abc", false)]
    fn string_length_max(#[case] value: &'static str, #[case] is_ok: bool) {
        type TestType = Refined<String, MaxLength<10>>;
        let res = TestType::parse(value.to_string());
        assert_eq!(res.is_ok(), is_ok);
    }

    #[rstest]
    #[case("hello world!", true)]
    #[case("this is not än ascii string", false)]
    fn test_pattern_ascii_string(#[case] value: &'static str, #[case] is_ok: bool) {
        type TestType = Refined<&'static str, AsciiString>;

        let res = TestType::parse(value);

        assert_eq!(res.is_ok(), is_ok);
    }

    #[rstest]
    #[case("0123456789abcdef", true)]
    #[case("xyz", false)]
    fn test_pattern_hex_string(#[case] value: &'static str, #[case] is_ok: bool) {
        type TestType = Refined<&'static str, HexString>;

        let res = TestType::parse(value);

        assert_eq!(res.is_ok(), is_ok);
    }
}
