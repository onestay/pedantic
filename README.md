# Pedantic

Pedantic is a simple, lightweight framework for creating checked types.

At the core of Pedantic is the [Refined] struct which wraps any type
and one or more [Validators](Validator).

One [Refined] type can have multiple validators by declaring them using a tuple.
The validators in the tuple are then evaluated from left to right.
If any one of the validators fail, parsing of the type fails.

This crate provides the a number of common validators in the [validators]
module. This includes:

- LessThan/LessEqual for all integers
- GreaterThan/GreatherEqual for all integer
- [`MaxLength`](validators::MaxLength)/[`MinLength`](validators::MinLength) for Strings
- AsciiString/HexString for Strings

As well as a simple way to create your own regex validators using the [pattern] macro.

## Example

```
use pedantic::Refined;
use pedantic::validators::*;

type MyU32 = Refined<u32, (GreaterThanU32<10>, LessThanU32<15>)>;

let my_valid_u32 = MyU32::parse(12);
assert!(my_valid_u32.is_ok());

let my_big_u32 = MyU32::parse(17);
assert!(my_big_u32.is_err());
```

```
use pedantic::Refined;
use pedantic::validators::*;

type MyAsciiString = Refined<String, (MinLength<5>, MaxLength<15>, AsciiString)>;

let my_valid_ascii_string = MyAsciiString::parse("Hello, World!".to_string());
assert!(my_valid_ascii_string.is_ok());

let my_invalid_ascii_string = MyAsciiString::parse("Hello, World but slightly longer!".to_string());
assert!(my_invalid_ascii_string.is_err());

let my_non_ascii_string = MyAsciiString::parse("Hello, Wörld!".to_string());
assert!(my_invalid_ascii_string.is_err());
```

Or create your own pattern validators using [pattern]

## Limitations

The [Validator] trait is not dyn compatible and does not take an &self
which might make writing more complicated validators hard.
This is however not a goal of this library. Consider wrapping a [Refined]
type in your own type if you need more complicated logic.

There currently is no way to `OR` validators.
