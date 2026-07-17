use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt;
use core::str::FromStr;

use crate::error::{Span, ValueError, ValueErrorKind};

/// A CPE attribute component.
///
/// `ANY` and `NA` are logical values, not absent strings. Keeping them as
/// variants prevents a literal `*` or `-` from being mistaken for a logical
/// value.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Component<T> {
    /// No restriction is placed on the attribute.
    #[default]
    Any,
    /// No legal or meaningful value applies to the attribute.
    NotApplicable,
    /// A concrete attribute value.
    Value(T),
}

impl<T> Component<T> {
    /// Returns whether this component is the logical value `ANY`.
    #[must_use]
    pub const fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    /// Returns whether this component is the logical value `NA`.
    #[must_use]
    pub const fn is_not_applicable(&self) -> bool {
        matches!(self, Self::NotApplicable)
    }

    /// Returns a reference to the concrete value, if present.
    #[must_use]
    pub const fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Any | Self::NotApplicable => None,
        }
    }
}

impl<T> From<T> for Component<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: fmt::Display> fmt::Display for Component<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => formatter.write_str("*"),
            Self::NotApplicable => formatter.write_str("-"),
            Self::Value(value) => value.fmt(formatter),
        }
    }
}

/// The product class described by a CPE name.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Part {
    /// A software application.
    Application,
    /// An operating system.
    OperatingSystem,
    /// A hardware device.
    Hardware,
}

impl Part {
    /// Returns the one-character formatted-binding representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "a",
            Self::OperatingSystem => "o",
            Self::Hardware => "h",
        }
    }
}

impl fmt::Display for Part {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One semantic atom in a CPE string value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValueAtom {
    /// A literal printable ASCII character.
    Literal(char),
    /// An unquoted `*`, matching zero or more characters in name matching.
    AnySequence,
    /// An unquoted `?`, matching zero or one character in name matching.
    ZeroOrOne,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Atom {
    Literal(u8),
    AnySequence,
    ZeroOrOne,
}

impl From<Atom> for ValueAtom {
    fn from(atom: Atom) -> Self {
        match atom {
            Atom::Literal(byte) => Self::Literal(char::from(byte)),
            Atom::AnySequence => Self::AnySequence,
            Atom::ZeroOrOne => Self::ZeroOrOne,
        }
    }
}

/// A validated, non-logical CPE attribute string.
///
/// The representation is semantic: escaped `\*` is stored as a literal
/// asterisk, while unescaped `*` is stored as [`ValueAtom::AnySequence`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ValueString {
    atoms: Box<[Atom]>,
}

impl ValueString {
    /// Constructs a value in which every input character is literal.
    ///
    /// Formatting adds the CPE escapes required for punctuation. Input must
    /// be non-empty printable ASCII. A lone `-` is rejected because it is
    /// indistinguishable from logical `NA` in a formatted binding.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError`] for empty, non-printable, non-ASCII, or
    /// unrepresentable input.
    pub fn literal(input: &str) -> Result<Self, ValueError> {
        if input.is_empty() {
            return Err(ValueError::new(ValueErrorKind::Empty, Span::new(0, 0)));
        }
        if input == "-" {
            return Err(ValueError::new(
                ValueErrorKind::AmbiguousLogicalValue,
                Span::new(0, 1),
            ));
        }

        let mut atoms = Vec::with_capacity(input.len());
        for (offset, character) in input.char_indices() {
            if !character.is_ascii_graphic() {
                return Err(ValueError::new(
                    ValueErrorKind::InvalidCharacter { character },
                    Span::new(offset, offset + character.len_utf8()),
                ));
            }
            atoms.push(Atom::Literal(character as u8));
        }

        Ok(Self {
            atoms: atoms.into_boxed_slice(),
        })
    }

    /// Parses one non-logical formatted-binding component.
    ///
    /// Unlike [`Self::literal`], unescaped edge `*` and `?` characters retain
    /// wildcard meaning. A lone `*` or `-` is rejected; callers should use
    /// [`Component::Any`] or [`Component::NotApplicable`] for those values.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError`] when `input` violates the CPE 2.3 formatted
    /// component grammar.
    pub fn parse_formatted(input: &str) -> Result<Self, ValueError> {
        if input.is_empty() {
            return Err(ValueError::new(ValueErrorKind::Empty, Span::new(0, 0)));
        }
        if input == "*" || input == "-" {
            return Err(ValueError::new(
                ValueErrorKind::AmbiguousLogicalValue,
                Span::new(0, input.len()),
            ));
        }

        parse_formatted_value(input).map_err(|failure| ValueError::new(failure.kind, failure.span))
    }

    /// Returns the semantic atoms in this value.
    pub fn atoms(&self) -> impl ExactSizeIterator<Item = ValueAtom> + '_ {
        self.atoms.iter().copied().map(ValueAtom::from)
    }

    /// Returns whether this value contains an unquoted wildcard.
    #[must_use]
    pub fn has_wildcards(&self) -> bool {
        self.atoms
            .iter()
            .any(|atom| !matches!(atom, Atom::Literal(_)))
    }

    /// Returns the decoded literal text when the value has no wildcards.
    ///
    /// Returns `None` when any unquoted `*` or `?` is present.
    #[must_use]
    pub fn to_literal_string(&self) -> Option<String> {
        if self.has_wildcards() {
            return None;
        }

        Some(
            self.atoms
                .iter()
                .filter_map(|atom| match atom {
                    Atom::Literal(byte) => Some(char::from(*byte)),
                    Atom::AnySequence | Atom::ZeroOrOne => None,
                })
                .collect(),
        )
    }

    /// Returns this value in the illustrative WFN attribute-string syntax.
    #[must_use]
    pub fn to_wfn_string(&self) -> String {
        let mut output = String::new();
        for atom in &self.atoms {
            match atom {
                Atom::Literal(byte) => {
                    let character = char::from(*byte);
                    if character.is_ascii_alphanumeric() || character == '_' {
                        output.push(character);
                    } else {
                        output.push('\\');
                        output.push(character);
                    }
                }
                Atom::AnySequence => output.push('*'),
                Atom::ZeroOrOne => output.push('?'),
            }
        }
        output
    }

    pub(crate) fn parse_at(input: &str, base_offset: usize) -> Result<Self, ValueFailure> {
        parse_formatted_value(input).map_err(|failure| failure.shift(base_offset))
    }
}

impl FromStr for ValueString {
    type Err = ValueError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse_formatted(input)
    }
}

impl fmt::Display for ValueString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for atom in &self.atoms {
            match atom {
                Atom::Literal(byte) => {
                    let character = char::from(*byte);
                    if character.is_ascii_alphanumeric() || matches!(character, '-' | '.' | '_') {
                        write!(formatter, "{character}")?;
                    } else {
                        write!(formatter, "\\{character}")?;
                    }
                }
                Atom::AnySequence => formatter.write_str("*")?,
                Atom::ZeroOrOne => formatter.write_str("?")?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValueFailure {
    pub(crate) kind: ValueErrorKind,
    pub(crate) span: Span,
}

impl ValueFailure {
    const fn shift(self, amount: usize) -> Self {
        Self {
            kind: self.kind,
            span: Span::new(self.span.start() + amount, self.span.end() + amount),
        }
    }
}

fn parse_formatted_value(input: &str) -> Result<ValueString, ValueFailure> {
    let bytes = input.as_bytes();
    let mut atoms = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut body_seen = false;
    let mut trailing_wildcard = None;
    let mut leading_kind = None;
    let mut trailing_kind = None;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\\' {
            let Some(&quoted) = bytes.get(index + 1) else {
                return Err(ValueFailure {
                    kind: ValueErrorKind::InvalidEscape,
                    span: Span::new(index, index + 1),
                });
            };
            if !is_quotable(quoted) {
                let end = next_character_end(input, index + 1);
                return Err(ValueFailure {
                    kind: ValueErrorKind::InvalidEscape,
                    span: Span::new(index, end),
                });
            }
            if let Some(wildcard_offset) = trailing_wildcard {
                return Err(ValueFailure {
                    kind: ValueErrorKind::InvalidWildcardPlacement,
                    span: Span::new(wildcard_offset, wildcard_offset + 1),
                });
            }
            atoms.push(Atom::Literal(quoted));
            body_seen = true;
            index += 2;
            continue;
        }

        if is_unreserved(byte) {
            if let Some(wildcard_offset) = trailing_wildcard {
                return Err(ValueFailure {
                    kind: ValueErrorKind::InvalidWildcardPlacement,
                    span: Span::new(wildcard_offset, wildcard_offset + 1),
                });
            }
            atoms.push(Atom::Literal(byte));
            body_seen = true;
            index += 1;
            continue;
        }

        if byte == b'*' || byte == b'?' {
            let atom = if byte == b'*' {
                Atom::AnySequence
            } else {
                Atom::ZeroOrOne
            };
            let slot = if body_seen {
                trailing_wildcard.get_or_insert(index);
                &mut trailing_kind
            } else {
                &mut leading_kind
            };
            if !wildcard_sequence_accepts(*slot, byte) {
                return Err(ValueFailure {
                    kind: ValueErrorKind::InvalidWildcardPlacement,
                    span: Span::new(index, index + 1),
                });
            }
            *slot = Some(byte);
            atoms.push(atom);
            index += 1;
            continue;
        }

        let character = input[index..].chars().next().unwrap_or('\0');
        return Err(ValueFailure {
            kind: ValueErrorKind::InvalidCharacter { character },
            span: Span::new(index, index + character.len_utf8()),
        });
    }

    if !body_seen {
        return Err(ValueFailure {
            kind: ValueErrorKind::InvalidWildcardPlacement,
            span: Span::new(0, input.len()),
        });
    }

    Ok(ValueString {
        atoms: atoms.into_boxed_slice(),
    })
}

const fn wildcard_sequence_accepts(current: Option<u8>, next: u8) -> bool {
    match current {
        None => true,
        Some(b'?') => next == b'?',
        Some(b'*') => false,
        Some(_) => false,
    }
}

const fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_')
}

const fn is_quotable(byte: u8) -> bool {
    matches!(
        byte,
        b'\\'
            | b'*'
            | b'?'
            | b'!'
            | b'"'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'+'
            | b','
            | b'/'
            | b':'
            | b';'
            | b'<'
            | b'='
            | b'>'
            | b'@'
            | b'['
            | b']'
            | b'^'
            | b'`'
            | b'{'
            | b'|'
            | b'}'
            | b'~'
    )
}

fn next_character_end(input: &str, offset: usize) -> usize {
    input
        .get(offset..)
        .and_then(|suffix| suffix.chars().next())
        .map_or(offset, |character| offset + character.len_utf8())
}

/// A language tag accepted by the CPE 2.3 formatted-string binding.
///
/// The grammar is intentionally narrower than general RFC 5646: two or three
/// ASCII letters, optionally followed by `-` and either two ASCII letters or
/// three digits.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LanguageTag(Box<str>);

impl LanguageTag {
    /// Parses a formatted-binding language tag.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageTagError`] if the input is outside the CPE 2.3
    /// formatted-binding language grammar.
    pub fn new(input: &str) -> Result<Self, LanguageTagError> {
        validate_language(input).map_err(LanguageTagError::new)?;
        Ok(Self(Box::from(input)))
    }

    /// Returns the original language tag.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for LanguageTag {
    type Err = LanguageTagError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// An invalid CPE formatted-binding language tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageTagError {
    span: Span,
}

impl LanguageTagError {
    const fn new(span: Span) -> Self {
        Self { span }
    }

    /// Returns the offending byte range.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for LanguageTagError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid CPE language tag at bytes {}..{}",
            self.span.start(),
            self.span.end()
        )
    }
}

impl Error for LanguageTagError {}

pub(crate) fn validate_language(input: &str) -> Result<(), Span> {
    let (language, region) = match input.split_once('-') {
        Some((language, region)) if !region.contains('-') => (language, Some(region)),
        Some(_) => return Err(Span::new(0, input.len())),
        None => (input, None),
    };

    if !(2..=3).contains(&language.len())
        || !language.bytes().all(|byte| byte.is_ascii_alphabetic())
    {
        return Err(first_invalid_language_span(input));
    }

    if let Some(region) = region {
        let valid_region = (region.len() == 2
            && region.bytes().all(|byte| byte.is_ascii_alphabetic()))
            || (region.len() == 3 && region.bytes().all(|byte| byte.is_ascii_digit()));
        if !valid_region {
            return Err(first_invalid_language_span(input));
        }
    }

    Ok(())
}

fn first_invalid_language_span(input: &str) -> Span {
    input
        .char_indices()
        .find_map(|(offset, character)| {
            (!character.is_ascii_alphanumeric() && character != '-')
                .then_some(Span::new(offset, offset + character.len_utf8()))
        })
        .unwrap_or_else(|| Span::new(0, input.len()))
}
