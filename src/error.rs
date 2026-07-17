use core::error::Error;
use core::fmt;

use crate::Attribute;

/// A half-open byte range within parser input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the inclusive start byte offset.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive end byte offset.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns whether the span contains no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// The category of a formatted-string parse failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// The input does not begin with the canonical `cpe:2.3:` prefix.
    InvalidPrefix,
    /// The binding does not contain exactly eleven attribute fields.
    WrongFieldCount {
        /// The required number of attribute fields.
        expected: usize,
        /// The number of fields found after escape-aware splitting.
        actual: usize,
    },
    /// An attribute field is empty.
    EmptyValue,
    /// The part is not `a`, `o`, `h`, `*`, or `-`.
    InvalidPart,
    /// The language does not satisfy the formatted-binding language grammar.
    InvalidLanguage,
    /// A character is outside the grammar for the current field.
    InvalidCharacter {
        /// The rejected character.
        character: char,
    },
    /// A backslash is dangling or does not quote a character allowed by the
    /// grammar; escape-aware field splitting can report this for any attribute.
    InvalidEscape,
    /// An unquoted wildcard is not in a permitted edge position.
    InvalidWildcardPlacement,
}

/// An error returned while parsing a CPE 2.3 formatted string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: Span,
    attribute: Option<Attribute>,
}

impl ParseError {
    pub(crate) const fn new(
        kind: ParseErrorKind,
        span: Span,
        attribute: Option<Attribute>,
    ) -> Self {
        Self {
            kind,
            span,
            attribute,
        }
    }

    /// Returns the failure category.
    #[must_use]
    pub const fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    /// Returns the byte range responsible for the failure.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the affected CPE attribute, when one can be identified.
    #[must_use]
    pub const fn attribute(&self) -> Option<Attribute> {
        self.attribute
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::InvalidPrefix => {
                write!(
                    formatter,
                    "invalid CPE 2.3 prefix at byte {}",
                    self.span.start
                )
            }
            ParseErrorKind::WrongFieldCount { expected, actual } => write!(
                formatter,
                "expected {expected} CPE attribute fields, found {actual}"
            ),
            ParseErrorKind::EmptyValue => {
                self.write_attribute_error(formatter, format_args!("empty value"))
            }
            ParseErrorKind::InvalidPart => {
                self.write_attribute_error(formatter, format_args!("invalid part"))
            }
            ParseErrorKind::InvalidLanguage => {
                self.write_attribute_error(formatter, format_args!("invalid language"))
            }
            ParseErrorKind::InvalidCharacter { character } => self
                .write_attribute_error(formatter, format_args!("invalid character {character:?}")),
            ParseErrorKind::InvalidEscape => {
                self.write_attribute_error(formatter, format_args!("invalid escape"))
            }
            ParseErrorKind::InvalidWildcardPlacement => {
                self.write_attribute_error(formatter, format_args!("invalid wildcard placement"))
            }
        }
    }
}

impl ParseError {
    fn write_attribute_error(
        &self,
        formatter: &mut fmt::Formatter<'_>,
        description: fmt::Arguments<'_>,
    ) -> fmt::Result {
        if let Some(attribute) = self.attribute {
            write!(
                formatter,
                "{description} for {attribute} at bytes {}..{}",
                self.span.start, self.span.end
            )
        } else {
            write!(
                formatter,
                "{description} at bytes {}..{}",
                self.span.start, self.span.end
            )
        }
    }
}

impl Error for ParseError {}

/// The category of a standalone [`crate::ValueString`] construction failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ValueErrorKind {
    /// A CPE value string cannot be empty.
    Empty,
    /// The value is reserved for the logical `ANY` or `NA` representation.
    AmbiguousLogicalValue,
    /// A character is outside the formatted-string grammar.
    InvalidCharacter {
        /// The rejected character.
        character: char,
    },
    /// A backslash does not quote a permitted character.
    InvalidEscape,
    /// An unquoted wildcard is not in a permitted edge position.
    InvalidWildcardPlacement,
}

/// An error returned while constructing a standalone CPE value string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueError {
    kind: ValueErrorKind,
    span: Span,
}

impl ValueError {
    pub(crate) const fn new(kind: ValueErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Returns the failure category.
    #[must_use]
    pub const fn kind(&self) -> ValueErrorKind {
        self.kind
    }

    /// Returns the offending byte range.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.kind {
            ValueErrorKind::Empty => "empty CPE value",
            ValueErrorKind::AmbiguousLogicalValue => "value conflicts with a logical value",
            ValueErrorKind::InvalidCharacter { character } => {
                return write!(
                    formatter,
                    "invalid character {character:?} in CPE value at bytes {}..{}",
                    self.span.start, self.span.end
                )
            }
            ValueErrorKind::InvalidEscape => "invalid escape in CPE value",
            ValueErrorKind::InvalidWildcardPlacement => "invalid wildcard placement in CPE value",
        };

        write!(
            formatter,
            "{description} at bytes {}..{}",
            self.span.start, self.span.end
        )
    }
}

impl Error for ValueError {}
