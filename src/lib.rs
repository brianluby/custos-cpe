#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod error;
mod formatted;
mod name;
mod value;

pub use error::{ParseError, ParseErrorKind, Span, ValueError, ValueErrorKind};
pub use name::{Attribute, Cpe, CpeBuilder};
pub use value::{Component, LanguageTag, LanguageTagError, Part, ValueAtom, ValueString};
