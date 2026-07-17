//! Black-box conformance tests for the CPE 2.3 formatted-string API.
//!
//! Expectations in this file were independently derived from NISTIR 7695,
//! sections 5 and 6.2. No third-party CPE implementation was used as an
//! oracle.

use std::panic::{catch_unwind, AssertUnwindSafe};

use custos_cpe::{
    Attribute, Component, Cpe, LanguageTag, ParseErrorKind, Part, ValueAtom, ValueErrorKind,
    ValueString,
};

fn parse_cpe(input: &str) -> Cpe {
    input
        .parse()
        .unwrap_or_else(|error| panic!("expected {input:?} to parse, got {error}"))
}

macro_rules! canonical_formatted_string_test {
    ($name:ident, $input:literal) => {
        #[test]
        fn $name() {
            let parsed = parse_cpe($input);

            assert_eq!(parsed.to_string(), $input);
        }
    };
}

mod official_formatted_string_examples {
    use super::*;

    // NISTIR 7695 section 6.2.2.3, examples 1 through 5.
    canonical_formatted_string_test!(
        microsoft_internet_explorer_beta_is_canonical,
        "cpe:2.3:a:microsoft:internet_explorer:8.0.6001:beta:*:*:*:*:*:*"
    );
    canonical_formatted_string_test!(
        microsoft_internet_explorer_wildcards_are_canonical,
        "cpe:2.3:a:microsoft:internet_explorer:8.*:sp?:*:*:*:*:*:*"
    );
    canonical_formatted_string_test!(
        microsoft_internet_explorer_literal_asterisk_is_canonical,
        r"cpe:2.3:a:microsoft:internet_explorer:8.\*:sp?:*:*:*:*:*:*"
    );
    canonical_formatted_string_test!(
        hp_insight_extended_attributes_are_canonical,
        "cpe:2.3:a:hp:insight:7.4.0.1570:-:*:*:online:win2003:x64:*"
    );
    canonical_formatted_string_test!(
        hp_openview_unspecified_attributes_are_canonical,
        "cpe:2.3:a:hp:openview_network_manager:7.51:*:*:*:*:linux:*:*"
    );
    canonical_formatted_string_test!(
        foo_bar_quoted_punctuation_is_canonical,
        r"cpe:2.3:a:foo\\bar:big\$money_2010:*:*:*:*:special:ipod_touch:80gb:*"
    );

    // NISTIR 7695 section 6.2.3.3 includes a product spelling distinct from
    // the binding example above; retain it as its own official input.
    canonical_formatted_string_test!(
        hp_insight_diagnostics_unbinding_example_is_canonical,
        "cpe:2.3:a:hp:insight_diagnostics:7.4.0.1570:-:*:*:online:win2003:x64:*"
    );
    canonical_formatted_string_test!(
        foo_bar_unbinding_example_is_canonical,
        r"cpe:2.3:a:foo\\bar:big\$money:2010:*:*:*:special:ipod_touch:80gb:*"
    );

    #[test]
    fn wildcard_example_preserves_unquoted_special_atoms() {
        let parsed = parse_cpe("cpe:2.3:a:microsoft:internet_explorer:8.*:sp?:*:*:*:*:*:*");
        let version_atoms = parsed
            .version()
            .as_value()
            .expect("version is a concrete string")
            .atoms()
            .collect::<Vec<_>>();
        let update_atoms = parsed
            .update()
            .as_value()
            .expect("update is a concrete string")
            .atoms()
            .collect::<Vec<_>>();

        assert_eq!(
            (version_atoms, update_atoms),
            (
                vec![
                    ValueAtom::Literal('8'),
                    ValueAtom::Literal('.'),
                    ValueAtom::AnySequence,
                ],
                vec![
                    ValueAtom::Literal('s'),
                    ValueAtom::Literal('p'),
                    ValueAtom::ZeroOrOne,
                ],
            )
        );
    }

    #[test]
    fn quoted_asterisk_example_preserves_a_literal_atom() {
        let parsed = parse_cpe(r"cpe:2.3:a:microsoft:internet_explorer:8.\*:sp?:*:*:*:*:*:*");
        let version = parsed
            .version()
            .as_value()
            .expect("version is a concrete string");

        assert_eq!(
            (
                version.atoms().collect::<Vec<_>>(),
                version.has_wildcards(),
                version.to_literal_string(),
            ),
            (
                vec![
                    ValueAtom::Literal('8'),
                    ValueAtom::Literal('.'),
                    ValueAtom::Literal('*'),
                ],
                false,
                Some("8.*".to_owned()),
            )
        );
    }
}

mod logical_values {
    use super::*;

    const LOGICAL_AND_LITERAL_VALUES: &str = r"cpe:2.3:a:*:-:\*:\?:*:*:*:*:*:*";

    #[test]
    fn lone_asterisk_unbinds_to_any() {
        let parsed = parse_cpe(LOGICAL_AND_LITERAL_VALUES);

        assert_eq!(parsed.vendor(), &Component::Any);
    }

    #[test]
    fn lone_hyphen_unbinds_to_not_applicable() {
        let parsed = parse_cpe(LOGICAL_AND_LITERAL_VALUES);

        assert_eq!(parsed.product(), &Component::NotApplicable);
    }

    #[test]
    fn escaped_asterisk_unbinds_to_a_literal_value() {
        let parsed = parse_cpe(LOGICAL_AND_LITERAL_VALUES);
        let version = parsed
            .version()
            .as_value()
            .expect("escaped asterisk is a value");

        assert_eq!(version.to_literal_string().as_deref(), Some("*"));
    }

    #[test]
    fn escaped_question_mark_unbinds_to_a_literal_value() {
        let parsed = parse_cpe(LOGICAL_AND_LITERAL_VALUES);
        let update = parsed
            .update()
            .as_value()
            .expect("escaped question mark is a value");

        assert_eq!(update.to_literal_string().as_deref(), Some("?"));
    }

    #[test]
    fn logical_values_are_allowed_for_part_and_language() {
        let any = parse_cpe("cpe:2.3:*:*:*:*:*:*:*:*:*:*:*");
        let not_applicable = parse_cpe("cpe:2.3:-:*:*:*:*:*:-:*:*:*:*");

        assert_eq!(
            (
                any.part(),
                any.language(),
                not_applicable.part(),
                not_applicable.language(),
            ),
            (
                &Component::Any,
                &Component::Any,
                &Component::NotApplicable,
                &Component::NotApplicable,
            )
        );
    }

    #[test]
    fn literal_constructor_rejects_lone_hyphen_as_ambiguous() {
        let error = ValueString::literal("-").expect_err("lone hyphen binds as logical NA");

        assert_eq!(error.kind(), ValueErrorKind::AmbiguousLogicalValue);
    }

    #[test]
    fn formatted_value_parser_rejects_reserved_logical_spellings() {
        for input in ["*", "-"] {
            let error = ValueString::parse_formatted(input)
                .expect_err("logical spellings require Component variants");
            assert_eq!(
                error.kind(),
                ValueErrorKind::AmbiguousLogicalValue,
                "unexpected error for {input:?}"
            );
        }
    }
}

mod escaping_and_wildcards {
    use super::*;

    #[test]
    fn escaped_colon_is_data_instead_of_a_field_delimiter() {
        let input = r"cpe:2.3:a:foo\:bar:widget:*:*:*:*:*:*:*:*";
        let parsed = parse_cpe(input);

        assert_eq!(
            (
                parsed
                    .vendor()
                    .as_value()
                    .expect("vendor is a value")
                    .to_literal_string(),
                parsed.to_string(),
            ),
            (Some("foo:bar".to_owned()), input.to_owned())
        );
    }

    #[test]
    fn escaped_backslash_decodes_to_one_literal_backslash() {
        let input = r"cpe:2.3:a:foo\\bar:widget:*:*:*:*:*:*:*:*";
        let parsed = parse_cpe(input);

        assert_eq!(
            parsed
                .vendor()
                .as_value()
                .expect("vendor is a value")
                .to_literal_string()
                .as_deref(),
            Some(r"foo\bar")
        );
    }

    #[test]
    fn literal_constructor_quotes_formatted_binding_punctuation() {
        let value = ValueString::literal(r##"a!"#$%&'()+,/:;<=>@[\]^`{|}~z"##)
            .expect("printable punctuation is representable when quoted");

        assert_eq!(
            value.to_string(),
            r#"a\!\"\#\$\%\&\'\(\)\+\,\/\:\;\<\=\>\@\[\\\]\^\`\{\|\}\~z"#
        );
    }

    #[test]
    fn wfn_string_escapes_hyphen_and_period_that_formatted_display_leaves_unquoted() {
        let value = ValueString::literal("release-1.0").expect("literal is representable");

        assert_eq!(
            (value.to_string(), value.to_wfn_string()),
            ("release-1.0".to_owned(), r"release\-1\.0".to_owned())
        );
    }

    #[test]
    fn escaped_and_unescaped_specials_have_distinct_atoms() {
        let literal = ValueString::parse_formatted(r"\*word\?")
            .expect("quoted special characters are literals");
        let wildcard = ValueString::parse_formatted("*word?")
            .expect("unquoted edge special characters are wildcards");

        assert_eq!(
            (
                literal.atoms().collect::<Vec<_>>(),
                wildcard.atoms().collect::<Vec<_>>(),
            ),
            (
                vec![
                    ValueAtom::Literal('*'),
                    ValueAtom::Literal('w'),
                    ValueAtom::Literal('o'),
                    ValueAtom::Literal('r'),
                    ValueAtom::Literal('d'),
                    ValueAtom::Literal('?'),
                ],
                vec![
                    ValueAtom::AnySequence,
                    ValueAtom::Literal('w'),
                    ValueAtom::Literal('o'),
                    ValueAtom::Literal('r'),
                    ValueAtom::Literal('d'),
                    ValueAtom::ZeroOrOne,
                ],
            )
        );
    }

    #[test]
    fn permitted_edge_wildcard_forms_parse() {
        for input in ["??word??", "*word*", "word*", "*word", "*word?"] {
            assert!(
                ValueString::parse_formatted(input).is_ok(),
                "expected wildcard form {input:?} to parse"
            );
        }
    }

    #[test]
    fn lone_question_mark_is_not_a_formatted_string_value() {
        // Section 5.3.2 permits `?` as an abstract WFN value, but the
        // formatted-string grammar in Figure 6-3 requires a body token.
        let error = ValueString::parse_formatted("?")
            .expect_err("formatted value has no required body token");

        assert_eq!(error.kind(), ValueErrorKind::InvalidWildcardPlacement);
    }
}

mod part_and_language_grammar {
    use super::*;

    #[test]
    fn concrete_part_spellings_map_to_typed_variants() {
        let application = parse_cpe("cpe:2.3:a:*:*:*:*:*:*:*:*:*:*");
        let operating_system = parse_cpe("cpe:2.3:o:*:*:*:*:*:*:*:*:*:*");
        let hardware = parse_cpe("cpe:2.3:h:*:*:*:*:*:*:*:*:*:*");

        assert_eq!(
            (application.part(), operating_system.part(), hardware.part(),),
            (
                &Component::Value(Part::Application),
                &Component::Value(Part::OperatingSystem),
                &Component::Value(Part::Hardware),
            )
        );
    }

    #[test]
    fn invalid_part_spellings_are_rejected() {
        for part in ["A", "x", "aa", r"\a"] {
            let input = format!("cpe:2.3:{part}:*:*:*:*:*:*:*:*:*:*");
            let error = input.parse::<Cpe>().expect_err("part must follow the ABNF");
            assert_eq!(
                error.kind(),
                ParseErrorKind::InvalidPart,
                "unexpected error for part {part:?}"
            );
        }
    }

    #[test]
    fn formatted_language_tag_forms_parse() {
        for tag in ["en", "eng", "en-US", "EN-us", "eng-419"] {
            let input = format!("cpe:2.3:a:*:*:*:*:*:{tag}:*:*:*:*");
            assert!(
                input.parse::<Cpe>().is_ok(),
                "expected language tag {tag:?} to parse"
            );
        }
    }

    #[test]
    fn malformed_language_tags_are_rejected() {
        for tag in [
            "e",
            "engl",
            "en-U",
            "en-USA",
            "en-12",
            "en-US-extra",
            "e1-US",
            "en_US",
        ] {
            let input = format!("cpe:2.3:a:*:*:*:*:*:{tag}:*:*:*:*");
            let error = input
                .parse::<Cpe>()
                .expect_err("language must follow the formatted-binding grammar");
            assert_eq!(
                (error.kind(), error.attribute()),
                (ParseErrorKind::InvalidLanguage, Some(Attribute::Language)),
                "unexpected error for language {tag:?}"
            );
        }
    }

    #[test]
    fn language_tag_constructor_preserves_valid_input() {
        let tag = LanguageTag::new("en-US").expect("valid language-region tag");

        assert_eq!(
            (tag.as_str(), tag.to_string()),
            ("en-US", "en-US".to_owned())
        );
    }

    #[test]
    fn language_tag_constructor_reports_invalid_input_directly() {
        let error = LanguageTag::new("e").expect_err("one-letter language is invalid");

        assert_eq!(
            (error.span().start(), error.span().end(), error.to_string(),),
            (0, 1, "invalid CPE language tag at bytes 0..1".to_owned())
        );
    }
}

mod malformed_formatted_strings {
    use super::*;

    #[test]
    fn ten_attribute_fields_are_rejected() {
        let input = "cpe:2.3:*:*:*:*:*:*:*:*:*:*";
        let error = input.parse::<Cpe>().expect_err("one field is missing");

        assert_eq!(
            error.kind(),
            ParseErrorKind::WrongFieldCount {
                expected: 11,
                actual: 10,
            }
        );
    }

    #[test]
    fn twelve_attribute_fields_are_rejected() {
        let input = "cpe:2.3:*:*:*:*:*:*:*:*:*:*:*:*";
        let first_extra_delimiter = input
            .match_indices(':')
            .nth(12)
            .map(|(offset, _)| offset)
            .expect("input contains an extra delimiter");
        let error = input.parse::<Cpe>().expect_err("one field is extra");

        assert_eq!(
            (error.kind(), error.span().start(), error.span().end(),),
            (
                ParseErrorKind::WrongFieldCount {
                    expected: 11,
                    actual: 12,
                },
                first_extra_delimiter,
                first_extra_delimiter + 1,
            )
        );
    }

    #[test]
    fn empty_attribute_field_is_rejected() {
        let input = "cpe:2.3:a:*::*:*:*:*:*:*:*:*";
        let error = input
            .parse::<Cpe>()
            .expect_err("formatted fields cannot be empty");

        assert_eq!(
            (error.kind(), error.attribute()),
            (ParseErrorKind::EmptyValue, Some(Attribute::Product))
        );
    }

    #[test]
    fn noncanonical_prefixes_are_rejected() {
        // Figure 6-3's ABNF literals are case-insensitive under RFC 5234
        // Section 2.3. This crate deliberately accepts only the lowercase
        // binding emitted by NISTIR 7695 Section 6.2.2.2; see the README.
        for input in [
            "cpe:/a:microsoft:internet_explorer:8.0.6001:beta",
            "CPE:2.3:a:*:*:*:*:*:*:*:*:*:*",
            "cpe:2.2:a:*:*:*:*:*:*:*:*:*:*",
        ] {
            let error = input
                .parse::<Cpe>()
                .expect_err("formatted binding prefix is exact");
            assert_eq!(
                error.kind(),
                ParseErrorKind::InvalidPrefix,
                "unexpected error for {input:?}"
            );
        }
    }

    #[test]
    fn invalid_escape_sequences_are_rejected() {
        for input in [r"\a", r"\.", r"\-", r"\_", "word\\"] {
            let error = ValueString::parse_formatted(input)
                .expect_err("escape target is not allowed by the formatted grammar");
            assert_eq!(
                error.kind(),
                ValueErrorKind::InvalidEscape,
                "unexpected error for {input:?}"
            );
        }
    }

    #[test]
    fn invalid_value_characters_are_rejected() {
        for input in ["two words", "$money", "café", "nul\0byte"] {
            let error = ValueString::parse_formatted(input)
                .expect_err("character is outside the formatted grammar");
            assert!(
                matches!(error.kind(), ValueErrorKind::InvalidCharacter { .. }),
                "unexpected error for {input:?}: {error:?}"
            );
        }
    }

    #[test]
    fn invalid_wildcard_placements_are_rejected() {
        for input in ["foo*bar", "foo?bar", "**foo", "foo**", "*foo**bar"] {
            let error = ValueString::parse_formatted(input)
                .expect_err("wildcard is embedded or repeated illegally");
            assert_eq!(
                error.kind(),
                ValueErrorKind::InvalidWildcardPlacement,
                "unexpected error for {input:?}"
            );
        }
    }

    #[test]
    fn embedded_asterisk_from_nist_unbinding_example_is_rejected() {
        let input = "cpe:2.3:a:hp:insight_diagnostics:7.4.*.1570:*:*:*:*:*:*:*";
        let error = input
            .parse::<Cpe>()
            .expect_err("NIST example identifies embedded asterisk as an error");

        assert_eq!(
            (error.kind(), error.attribute()),
            (
                ParseErrorKind::InvalidWildcardPlacement,
                Some(Attribute::Version),
            )
        );
    }
}

mod byte_spans {
    use super::*;

    #[test]
    fn standalone_value_error_span_uses_utf8_byte_offsets() {
        let error = ValueString::literal("abé")
            .expect_err("non-ASCII code point is outside the CPE grammar");

        assert_eq!((error.span().start(), error.span().end()), (2, 4));
    }

    #[test]
    fn standalone_formatted_error_span_covers_invalid_character() {
        let error =
            ValueString::parse_formatted("ab$cd").expect_err("unquoted dollar sign is not allowed");

        assert_eq!(
            (error.kind(), error.span().start(), error.span().end()),
            (ValueErrorKind::InvalidCharacter { character: '$' }, 2, 3,)
        );
    }

    #[test]
    fn cpe_error_span_is_shifted_to_the_full_input() {
        let input = "cpe:2.3:a:vendor:proéduct:*:*:*:*:*:*:*:*";
        let error = input
            .parse::<Cpe>()
            .expect_err("non-ASCII product character is invalid");

        assert_eq!(
            (
                error.kind(),
                error.span().start(),
                error.span().end(),
                error.attribute(),
            ),
            (
                ParseErrorKind::InvalidCharacter { character: 'é' },
                20,
                22,
                Some(Attribute::Product),
            )
        );
    }

    #[test]
    fn wildcard_error_span_covers_the_embedded_asterisk() {
        let input = "cpe:2.3:a:vendor:product:foo*bar:*:*:*:*:*:*:*";
        let expected_start = input.find('*').expect("input contains an asterisk");
        let error = input
            .parse::<Cpe>()
            .expect_err("embedded wildcard is invalid");

        assert_eq!(
            (error.span().start(), error.span().end(), error.attribute(),),
            (expected_start, expected_start + 1, Some(Attribute::Version),)
        );
    }
}

mod error_messages {
    use super::*;

    #[test]
    fn parse_error_display_covers_every_current_kind() {
        let cases = [
            (
                "CPE:2.3:a:*:*:*:*:*:*:*:*:*:*",
                "invalid CPE 2.3 prefix at byte 0",
            ),
            ("cpe:2.3:*", "expected 11 CPE attribute fields, found 1"),
            (
                "cpe:2.3:a:*::*:*:*:*:*:*:*:*",
                "empty value for product at bytes 12..12",
            ),
            (
                "cpe:2.3:x:*:*:*:*:*:*:*:*:*:*",
                "invalid part for part at bytes 8..9",
            ),
            (
                "cpe:2.3:a:*:*:*:*:*:e:*:*:*:*",
                "invalid language for language at bytes 20..21",
            ),
            (
                "cpe:2.3:a:$:*:*:*:*:*:*:*:*:*",
                "invalid character '$' for vendor at bytes 10..11",
            ),
            (
                "cpe:2.3:a:foo*bar:*:*:*:*:*:*:*:*:*",
                "invalid wildcard placement for vendor at bytes 13..14",
            ),
            (
                "cpe:2.3:*:*:*:*:*:*:*:*:*:*:*:\\",
                "invalid escape at bytes 30..31",
            ),
        ];

        for (input, expected) in cases {
            let error = input.parse::<Cpe>().expect_err("input must be rejected");
            assert_eq!(
                error.to_string(),
                expected,
                "unexpected message for {input:?}"
            );
        }
    }

    #[test]
    fn value_error_display_covers_every_current_kind() {
        let errors = [
            ValueString::literal("").expect_err("empty value is invalid"),
            ValueString::literal("-").expect_err("lone hyphen conflicts with logical NA"),
            ValueString::parse_formatted("$")
                .expect_err("unescaped dollar is invalid formatted syntax"),
            ValueString::parse_formatted(r"\a").expect_err("escape target is invalid"),
            ValueString::parse_formatted("foo*bar").expect_err("embedded wildcard is invalid"),
        ];
        let expected = [
            "empty CPE value at bytes 0..0",
            "value conflicts with a logical value at bytes 0..1",
            "invalid character '$' in CPE value at bytes 0..1",
            "invalid escape in CPE value at bytes 0..2",
            "invalid wildcard placement in CPE value at bytes 3..4",
        ];

        for (error, expected) in errors.iter().zip(expected) {
            assert_eq!(error.to_string(), expected);
        }
    }
}

mod builder_round_trips {
    use super::*;

    #[test]
    fn unspecified_builder_attributes_default_to_any() {
        let built = Cpe::builder(Part::Application).build();

        assert_eq!(built.to_string(), "cpe:2.3:a:*:*:*:*:*:*:*:*:*:*");
    }

    #[test]
    fn default_cpe_sets_every_attribute_to_any() {
        let default = Cpe::default();

        assert_eq!(default.to_string(), "cpe:2.3:*:*:*:*:*:*:*:*:*:*:*");
    }

    #[test]
    fn fully_populated_builder_round_trips_through_formatted_string() {
        let built = Cpe::builder(Part::Application)
            .vendor(ValueString::literal(r"foo\bar").expect("literal vendor"))
            .product(ValueString::literal("big$money").expect("literal product"))
            .version(ValueString::parse_formatted("8.*").expect("wildcard version"))
            .update(Component::<ValueString>::NotApplicable)
            .edition(Component::<ValueString>::Any)
            .language(LanguageTag::new("en-US").expect("valid language"))
            .software_edition(ValueString::literal("special").expect("software edition"))
            .target_software(ValueString::literal("win2003").expect("target software"))
            .target_hardware(ValueString::literal("x64").expect("target hardware"))
            .other(ValueString::literal("*").expect("literal asterisk"))
            .build();
        let expected = r"cpe:2.3:a:foo\\bar:big\$money:8.*:-:*:en-US:special:win2003:x64:\*";
        let reparsed = parse_cpe(expected);

        assert_eq!((built.to_string(), reparsed), (expected.to_owned(), built));
    }

    #[test]
    fn logical_part_builder_round_trips() {
        let any = Cpe::builder(Component::<Part>::Any).build();
        let not_applicable = Cpe::builder(Component::<Part>::NotApplicable).build();

        assert_eq!(
            (
                parse_cpe(&any.to_string()),
                parse_cpe(&not_applicable.to_string()),
            ),
            (any, not_applicable)
        );
    }

    #[test]
    fn literal_value_round_trips_through_formatted_parser() {
        let literal = ValueString::literal(r"colon:slash\star*question?")
            .expect("printable literal is representable");
        let formatted = literal.to_string();
        let reparsed = ValueString::parse_formatted(&formatted)
            .expect("formatter output must be accepted by parser");

        assert_eq!(reparsed, literal);
    }
}

mod robustness {
    use super::*;

    #[test]
    fn representative_malformed_inputs_do_not_panic() {
        let malformed = [
            "",
            "cpe",
            "cpe:2.3:",
            "cpe:2.3::::::::::: ",
            "cpe:2.3:a:*:*:*:*:*:*:*:*:*",
            "cpe:2.3:a:*:*:*:*:*:*:*:*:*:*:extra",
            "cpe:2.3:a:foo\\",
            "cpe:2.3:a:foo\0bar:*:*:*:*:*:*:*:*:*",
            "cpe:2.3:a:fooé:bar:*:*:*:*:*:*:*:*",
            "cpe:2.3:a:foo*bar:baz:*:*:*:*:*:*:*:*",
            r"cpe:2.3:a:foo\:bar:*:*:*:*:*:*:*:*",
        ];

        for input in malformed {
            let outcome = catch_unwind(AssertUnwindSafe(|| input.parse::<Cpe>()));
            assert!(outcome.is_ok(), "parser panicked for {input:?}");
        }
    }
}
