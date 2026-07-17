//! Strict, clean-room parsing and formatting for CPE 2.3 names.
//!
//! This release implements the CPE 2.3 formatted-string binding defined by
//! NIST IR 7695. It distinguishes the logical values `ANY` and `NA` from
//! ordinary strings and preserves whether `*` and `?` are wildcards or
//! escaped literal characters.
//!
//! # Example
//!
//! ```
//! use custos_cpe::{Component, Cpe};
//!
//! let cpe: Cpe =
//!     "cpe:2.3:a:microsoft:internet_explorer:8.0.6001:beta:*:*:*:*:*:*"
//!         .parse()?;
//!
//! assert_eq!(cpe.vendor().as_value().unwrap().to_literal_string().as_deref(), Some("microsoft"));
//! assert!(matches!(cpe.edition(), Component::Any));
//! assert_eq!(cpe.to_string(), "cpe:2.3:a:microsoft:internet_explorer:8.0.6001:beta:*:*:*:*:*:*");
//! # Ok::<(), custos_cpe::ParseError>(())
//! ```
//!
//! URI-bound names beginning with `cpe:/` and CPE name matching are not yet
//! implemented.

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
