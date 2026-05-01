// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 Marius Meschter
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::missing_inline_in_public_items)]
#![deny(clippy::unwrap_in_result)]
#![deny(clippy::unwrap_used)]

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
//! - [`MaxLength`](validators::MaxLength)/[`MinLength`](validators::MinLength) for Strings
//! - [`AsciiString`](validators::pattern::AsciiString)/[`HexString`](validators::pattern::HexString) for Strings
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
//! ```
//! use pedantic::Refined;
//! use pedantic::validators::*;
//! use pedantic::validators::pattern::*;
//!
//! type MyAsciiString = Refined<String, (MinLength<5>, MaxLength<15>, AsciiString)>;
//!
//! let my_valid_ascii_string = MyAsciiString::parse("Hello, World!".to_string());
//! assert!(my_valid_ascii_string.is_ok());
//!
//! let my_invalid_ascii_string = MyAsciiString::parse("Hello, World but slightly longer!".to_string());
//! assert!(my_invalid_ascii_string.is_err());
//!
//! let my_non_ascii_string = MyAsciiString::parse("Hello, Wörld!".to_string());
//! assert!(my_invalid_ascii_string.is_err());
//! ```
//!
//! or create your own pattern validators using [pattern]
//!
//! # Limitations
//! The [Validator] trait is not dyn compatible and does not take an &self
//! which might make writing more complicated validators hard.
//! This is however not a goal of this library. Consider wrapping a [Refined]
//! type in your own type if you need more complicated logic.
//!
//! There currently is no way to `OR` validators.
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Debug, Display},
    marker::PhantomData,
};

/// An opaque error type
///
/// Use the [`PedanticError::new`] or [`PedanticError::with_source_error`] function to construct an instance of this type.
/// The error is opaque on purpose and only contains a String to display the validation error to an
/// user.
///
/// It also optionally contains an error source that can be retrived using [`Error::source`]
#[derive(Debug)]
pub struct PedanticError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl PedanticError {
    /// Construct a new error with the given message.
    #[inline]
    pub fn new<T: Display>(message: T) -> Self {
        Self {
            message: message.to_string(),
            source: None,
        }
    }

    /// Construct a new error with the given message and source error.
    #[inline]
    pub fn with_source_error<T: Display>(
        message: T,
        source: Box<dyn Error + Send + Sync + 'static>,
    ) -> Self {
        Self {
            message: message.to_string(),
            source: Option::Some(source),
        }
    }
}

impl Display for PedanticError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "validation failure: {}", self.message)
    }
}

impl Error for PedanticError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }
}

/// The main trait for validation.
///
/// This trait is deliberately not object safe and doesn't take a `&self` as it's argument.
/// This is done to keep the `type` declarations clean and declarative.
pub trait Validator<T> {
    /// Returns a validation result for the given `T`.
    ///
    /// # Errors
    ///
    /// Should return [`PedanticError`] if the validation for `value` failed.
    fn validate(value: &T) -> Result<(), PedanticError>;
}

/// Trait used for the [`MaxLength`](validators::MaxLength) and [`MinLength`](validators::MinLength)
/// validators.
///
/// This trait includes a blanket impl for various traits that have a length from the standard
/// library.
///
/// # Note
/// `HasLen` is implemented for [String] and [str] using the [`String::len`] function which returns
/// the number of bytes and not the actual length of the String in chars or graphemes.
#[allow(clippy::len_without_is_empty)]
pub trait HasLen {
    /// Returns the length of the type
    fn len(&self) -> usize;
}

impl<T> HasLen for [T] {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for Vec<T> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl HasLen for str {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl HasLen for String {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: HasLen + ?Sized> HasLen for &T {
    #[inline]
    fn len(&self) -> usize {
        (**self).len()
    }
}

impl<K, V, S: std::hash::BuildHasher> HasLen for HashMap<K, V, S> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T, S: std::hash::BuildHasher> HasLen for HashSet<T, S> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T, V1, V2> Validator<T> for (V1, V2)
where
    V1: Validator<T>,
    V2: Validator<T>,
{
    #[inline]
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
    #[inline]
    fn validate(value: &T) -> Result<(), PedanticError> {
        V1::validate(value)?;
        V2::validate(value)?;
        V3::validate(value)?;

        Ok(())
    }
}

/// A wrapper around a T, parsing it using the given [Validators](Validator).
///
/// # Limitations
/// It is not possible to get an `&mut T` since bypassing the validators in that way would be
/// trivial. For updating the value use [`Refined::update`].
pub struct Refined<T, V: Validator<T>> {
    data: T,
    validators: PhantomData<V>,
}

impl<T, V> Debug for Refined<T, V>
where
    T: Debug,
    V: Validator<T>,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Refined").field("data", &self.data).finish()
    }
}

impl<T, V> Clone for Refined<T, V>
where
    T: Clone,
    V: Validator<T>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self { data: self.data.clone(), validators: self.validators }
    }
}

impl<T, V> Copy for Refined<T, V>
where
    T: Copy,
    V: Validator<T>,
{}

impl<T, V> PartialEq for Refined<T, V>
where
    T: PartialEq,
    V: Validator<T>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T, V> PartialOrd for Refined<T, V>
where
    T: PartialOrd,
    V: Validator<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T, V> Ord for Refined<T, V>
where
    T: Ord,
    V: Validator<T>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T, V> Eq for Refined<T, V>
where
    T: Eq,
    V: Validator<T>,
{}

impl<T, V> ::std::hash::Hash for Refined<T, V>
where
    T: ::std::hash::Hash,
    V: Validator<T>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T, V> Default for Refined<T, V>
where
    T: Default,
    V: Validator<T>,
{
    #[inline]
    fn default() -> Self {
        Self { data: T::default(), validators: PhantomData }
    }
}


impl<T: std::fmt::Display, V: Validator<T>> std::fmt::Display for Refined<T, V> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl<T, V> Refined<T, V>
where
    V: Validator<T>,
{
    /// Returns either an error if validation of `T` fails or the wrapped type.
    ///
    /// # Errors
    ///
    /// Returns [`PedanticError`] if any of the validators fail.
    #[inline]
    pub fn parse(value: T) -> Result<Self, PedanticError> {
        V::validate(&value)?;

        Ok(Self {
            data: value,
            validators: PhantomData,
        })
    }

    /// Updates the value inside the Refined struct.
    ///
    /// This method will run the validators on T and if it passes the checks, update the wrapped
    /// value with T.
    ///
    /// # Errors
    ///
    /// Return [`PedanticError`] is any of the validators fail for the new value.
    #[inline]
    pub fn update(&mut self, value: T) -> Result<(), PedanticError> {
        V::validate(&value)?;
        self.data = value;

        Ok(())
    }

    /// Consumes self and returns the wrapped value.
    #[inline]
    pub fn take(self) -> T {
        self.data
    }
}

impl<T, V> AsRef<T> for Refined<T, V>
where
    V: Validator<T>,
{
    #[inline]
    fn as_ref(&self) -> &T {
        &self.data
    }
}

#[cfg(feature = "serde")]
impl<T: ::serde::Serialize, V: Validator<T>> ::serde::Serialize for Refined<T, V> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T, V> ::serde::Deserialize<'de> for Refined<T, V>
where
    T: ::serde::Deserialize<'de>,
    V: Validator<T>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;

        Self::parse(value).map_err(::serde::de::Error::custom)
    }
}

#[cfg(feature = "regex")]
#[doc(hidden)]
pub mod pattern {
    use crate::{PedanticError, Validator};
    use regex::Regex;

    /// Trait for a regex validator. For creating a new pattern validator use the [pattern] macro.
    #[doc(hidden)]
    pub trait PatternValidator {
        const PATTERN: &'static str;
        fn regex() -> &'static Regex;
    }

    impl<T: AsRef<str>, U: PatternValidator> Validator<T> for U {
        #[inline]
        fn validate(value: &T) -> Result<(), PedanticError> {
            let regex = U::regex();
            if !regex.is_match(value.as_ref()) {
                let err = "the given value does not conform to the pattern.".to_string();
                return Err(PedanticError::new(err));
            }

            Ok(())
        }
    }

    /// Create custom regex validators using this macro.
    ///
    /// The name of the validator is the first option and the regex the second.
    /// # Example
    /// ```
    /// use pedantic::pattern_validator;
    /// use pedantic::pattern::PatternValidator;
    ///
    /// pattern_validator!(MyCoolRegexValidator, r"^[b-chm-pP]at|ot$");
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "regex")))]
    #[macro_export]
    macro_rules! pattern_validator {
        ($name:ident, $regex:expr) => {
            #[doc = concat!("Match the given str against the regex `", stringify!($regex), "`.")]
            pub struct $name;

            impl $crate::pattern::PatternValidator for $name {
                const PATTERN: &'static str = $regex;

                #[inline]
                fn regex() -> &'static ::regex::Regex {
                    static REGEX: ::std::sync::LazyLock<::regex::Regex> =
                        ::std::sync::LazyLock::new(|| {
                            ::regex::Regex::new(
                                <$name as $crate::pattern::PatternValidator>::PATTERN,
                            )
                            .expect("invalid regex")
                        });
                    &REGEX
                }
            }
        };
    }
}

/// Basic validators provided by this crate.
///
/// This module contains basic validators for all string types
pub mod validators {
    use crate::{HasLen, PedanticError, Validator};

    use paste::paste;

    /// Validates that the length of `T` is longer than `MIN`.
    pub struct MinLength<const MIN: usize>;
    impl<const MIN: usize, T: HasLen> Validator<T> for MinLength<MIN> {
        #[inline]
        fn validate(value: &T) -> Result<(), PedanticError> {
            if value.len() < MIN {
                let err = format!("the given string needs to be at least {MIN} bytes long");
                return Err(PedanticError::new(err));
            }
            Ok(())
        }
    }

    /// Validates that the length of `T` is shorter than `MAX`.
    pub struct MaxLength<const MAX: usize>;
    impl<const MAX: usize, T: HasLen> Validator<T> for MaxLength<MAX> {
        #[inline]
        fn validate(value: &T) -> Result<(), PedanticError> {
            if value.len() > MAX {
                let err = format!("the given string can not be longer than {MAX} byes");
                return Err(PedanticError::new(err));
            }
            Ok(())
        }
    }

    /// Validates that the length of `T` is exactly `LEN`.
    pub struct ExactLength<const LEN: usize>;
    impl<const LEN: usize, T: HasLen> Validator<T> for ExactLength<LEN> {
        #[inline]
        fn validate(value: &T) -> Result<(), PedanticError> {
            if value.len() > LEN {
                let err = format!("the given value must be exactly {LEN} long");
                return Err(PedanticError::new(err));
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
                        #[inline]
                        fn validate(value: &$type) -> Result<(), $crate::PedanticError> {
                            if *value $op $generic_name {
                                let err = format!("condition '{value} {} {}' not fulfilled.", stringify!($op), $generic_name);
                                return Err(PedanticError::new(err));
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

    #[cfg(feature = "regex")]
    #[cfg_attr(docsrs, doc(cfg(feature = "regex")))]
    /// Provided pattern validators.
    pub mod pattern {
        use crate::pattern_validator;
        pattern_validator!(AsciiString, r"^[[:ascii:]]*$");
        pattern_validator!(HexString, r"^[\da-fA-f]*$");
    }
}

#[cfg(test)]
mod test {
    use crate::Refined;
    use crate::pattern_validator;
    use crate::validators::pattern::*;
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
        type TestType<'a> = Refined<&'a String, MaxLength<10>>;
        let value = value.to_string();
        let res = TestType::parse(&value);
        assert_eq!(res.is_ok(), is_ok);
    }

    #[test]
    fn vec_length() {
        type TestType = Refined<Vec<u32>, MaxLength<2>>;

        let res = TestType::parse(vec![1, 2, 3, 4]);

        assert!(res.is_err());
    }

    #[test]
    fn slice_length() {
        type TestType<'a> = Refined<&'a [u32], MaxLength<2>>;
        let test_arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let res = TestType::parse(&test_arr[0..5]);

        assert!(res.is_err());
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

    #[test]
    fn test_custom_pattern() {
        pattern_validator!(MyCoolRegexValidator, r"^[cb]at$");
        type MyCoolString = Refined<String, MyCoolRegexValidator>;

        let my_valid_string = MyCoolString::parse("cat".to_string());
        assert!(my_valid_string.is_ok());
    }

    #[test]
    fn test_usage_in_struct() {
        // just to check it compiles
        #[allow(unused)]
        type MyType = Refined<u32, GreaterEqualU32<10>>;

        #[allow(unused)]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
        struct MyStruct {
            my_type: MyType,
        }
    }
}
