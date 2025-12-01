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

impl<T, V> AsRef<T> for Refined<T, V>
where
    V: Validator<T>,
{
    fn as_ref(&self) -> &T {
        &self.data
    }
}

#[cfg(test)]
mod test {
    use crate::{Refined, validators::*};
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
}

pub mod validators {
    use crate::{PedanticError, Validator};

    use paste::paste;
    macro_rules! less_than {
        ($( $type:ty ) +) => {
            paste! {
                $(
                    pub struct [<LessThan $type:upper>]<const MAX: $type>;

                    impl<const MAX: $type> Validator<$type> for [<LessThan $type:upper>]<MAX> {
                        fn validate(value: &$type) -> Result<(), crate::PedanticError> {
                            if *value >= MAX {
                                return Err(PedanticError::ParsingFailedError);
                            }

                            Ok(())
                        }
                    }
                )+
            }
        };
    }

    less_than!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

    macro_rules! greater_than {
        ($( $type:ty ) +) => {
            paste! {
                $(
                    pub struct [<GreaterThan $type:upper>]<const MIN: $type>;

                    impl<const MIN: $type> Validator<$type> for [<GreaterThan $type:upper>]<MIN> {
                        fn validate(value: &$type) -> Result<(), crate::PedanticError> {
                            if *value <= MIN {
                                return Err(PedanticError::ParsingFailedError);
                            }

                            Ok(())
                        }
                    }
                )+
            }
        };
    }

    greater_than!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);
}
