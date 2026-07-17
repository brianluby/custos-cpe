use core::str::FromStr;

use crate::error::{ParseError, ParseErrorKind, Span, ValueErrorKind};
use crate::name::{Attribute, Cpe};
use crate::value::{Component, LanguageTag, Part, ValueString};

const PREFIX: &str = "cpe:2.3:";
const FIELD_COUNT: usize = 11;
const ATTRIBUTES: [Attribute; FIELD_COUNT] = [
    Attribute::Part,
    Attribute::Vendor,
    Attribute::Product,
    Attribute::Version,
    Attribute::Update,
    Attribute::Edition,
    Attribute::Language,
    Attribute::SoftwareEdition,
    Attribute::TargetSoftware,
    Attribute::TargetHardware,
    Attribute::Other,
];

#[derive(Clone, Copy, Debug)]
struct Field<'a> {
    value: &'a str,
    offset: usize,
}

impl Cpe {
    /// Parses the CPE 2.3 formatted-string binding.
    ///
    /// Parsing is strict and follows NIST IR 7695 Figure 6-3. It does not
    /// trim input, fill omitted fields, accept URI-bound `cpe:/` names, or
    /// percent-decode values.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] with the affected attribute and UTF-8 byte span
    /// when the input is not a valid formatted-string binding.
    pub fn parse_formatted(input: &str) -> Result<Self, ParseError> {
        if !input.starts_with(PREFIX) {
            let offset = first_prefix_mismatch(input);
            return Err(ParseError::new(
                ParseErrorKind::InvalidPrefix,
                character_span_or_eof(input, offset),
                None,
            ));
        }

        let fields = split_fields(&input[PREFIX.len()..], PREFIX.len())?;
        let [part, vendor, product, version, update, edition, language, software_edition, target_software, target_hardware, other] =
            fields;

        Ok(Self {
            part: parse_part(part)?,
            vendor: parse_value(vendor, Attribute::Vendor)?,
            product: parse_value(product, Attribute::Product)?,
            version: parse_value(version, Attribute::Version)?,
            update: parse_value(update, Attribute::Update)?,
            edition: parse_value(edition, Attribute::Edition)?,
            language: parse_language(language)?,
            software_edition: parse_value(software_edition, Attribute::SoftwareEdition)?,
            target_software: parse_value(target_software, Attribute::TargetSoftware)?,
            target_hardware: parse_value(target_hardware, Attribute::TargetHardware)?,
            other: parse_value(other, Attribute::Other)?,
        })
    }
}

impl FromStr for Cpe {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse_formatted(input)
    }
}

impl TryFrom<&str> for Cpe {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Self::parse_formatted(input)
    }
}

fn split_fields(input: &str, base_offset: usize) -> Result<[Field<'_>; FIELD_COUNT], ParseError> {
    let bytes = input.as_bytes();
    let empty = Field {
        value: "",
        offset: base_offset,
    };
    let mut fields = [empty; FIELD_COUNT];
    let mut field_count = 0_usize;
    let mut first_extra_delimiter = None;
    let mut field_start = 0;
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                if index + 1 == bytes.len() {
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidEscape,
                        Span::new(base_offset + index, base_offset + index + 1),
                        ATTRIBUTES.get(field_count).copied(),
                    ));
                }
                index += 2;
            }
            b':' => {
                if field_count == FIELD_COUNT - 1 && first_extra_delimiter.is_none() {
                    first_extra_delimiter = Some(base_offset + index);
                }
                if let Some(field) = fields.get_mut(field_count) {
                    *field = Field {
                        value: &input[field_start..index],
                        offset: base_offset + field_start,
                    };
                }
                field_count = field_count.saturating_add(1);
                field_start = index + 1;
                index += 1;
            }
            _ => index += 1,
        }
    }

    if let Some(field) = fields.get_mut(field_count) {
        *field = Field {
            value: &input[field_start..],
            offset: base_offset + field_start,
        };
    }
    field_count = field_count.saturating_add(1);

    if field_count != FIELD_COUNT {
        let span = first_extra_delimiter.map_or_else(
            || Span::new(base_offset + input.len(), base_offset + input.len()),
            |offset| Span::new(offset, offset + 1),
        );
        return Err(ParseError::new(
            ParseErrorKind::WrongFieldCount {
                expected: FIELD_COUNT,
                actual: field_count,
            },
            span,
            None,
        ));
    }

    Ok(fields)
}

fn parse_part(field: Field<'_>) -> Result<Component<Part>, ParseError> {
    let part = match field.value {
        "*" => Component::Any,
        "-" => Component::NotApplicable,
        "a" => Component::Value(Part::Application),
        "o" => Component::Value(Part::OperatingSystem),
        "h" => Component::Value(Part::Hardware),
        "" => {
            return Err(field_error(
                ParseErrorKind::EmptyValue,
                field,
                Attribute::Part,
            ))
        }
        _ => {
            return Err(field_error(
                ParseErrorKind::InvalidPart,
                field,
                Attribute::Part,
            ))
        }
    };
    Ok(part)
}

fn parse_value(
    field: Field<'_>,
    attribute: Attribute,
) -> Result<Component<ValueString>, ParseError> {
    match field.value {
        "*" => Ok(Component::Any),
        "-" => Ok(Component::NotApplicable),
        "" => Err(field_error(ParseErrorKind::EmptyValue, field, attribute)),
        value => ValueString::parse_at(value, field.offset)
            .map(Component::Value)
            .map_err(|failure| {
                ParseError::new(
                    match failure.kind {
                        ValueErrorKind::InvalidCharacter { character } => {
                            ParseErrorKind::InvalidCharacter { character }
                        }
                        ValueErrorKind::InvalidEscape => ParseErrorKind::InvalidEscape,
                        ValueErrorKind::InvalidWildcardPlacement => {
                            ParseErrorKind::InvalidWildcardPlacement
                        }
                        ValueErrorKind::Empty => ParseErrorKind::EmptyValue,
                        ValueErrorKind::AmbiguousLogicalValue => {
                            ParseErrorKind::InvalidWildcardPlacement
                        }
                    },
                    failure.span,
                    Some(attribute),
                )
            }),
    }
}

fn parse_language(field: Field<'_>) -> Result<Component<LanguageTag>, ParseError> {
    match field.value {
        "*" => Ok(Component::Any),
        "-" => Ok(Component::NotApplicable),
        "" => Err(field_error(
            ParseErrorKind::EmptyValue,
            field,
            Attribute::Language,
        )),
        value => LanguageTag::new(value)
            .map(Component::Value)
            .map_err(|error| {
                let span = error.span();
                ParseError::new(
                    ParseErrorKind::InvalidLanguage,
                    Span::new(field.offset + span.start(), field.offset + span.end()),
                    Some(Attribute::Language),
                )
            }),
    }
}

fn field_error(kind: ParseErrorKind, field: Field<'_>, attribute: Attribute) -> ParseError {
    ParseError::new(
        kind,
        Span::new(field.offset, field.offset + field.value.len()),
        Some(attribute),
    )
}

fn first_prefix_mismatch(input: &str) -> usize {
    input
        .as_bytes()
        .iter()
        .zip(PREFIX.as_bytes())
        .position(|(actual, expected)| actual != expected)
        .unwrap_or_else(|| input.len().min(PREFIX.len()))
}

fn character_span_or_eof(input: &str, offset: usize) -> Span {
    let end = input
        .get(offset..)
        .and_then(|rest| rest.chars().next())
        .map_or(offset, |character| offset + character.len_utf8());
    Span::new(offset, end)
}
