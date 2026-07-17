# custos-cpe

`custos-cpe` is a strict, independently implemented Rust parser and formatter
for the CPE 2.3 formatted-string binding. It is dual-licensed under MIT or
Apache-2.0 and works in `no_std` environments with an allocator.

The implementation is derived from the public NIST specifications. It does
not contain or translate source code from `scap-rs` or another CPE
implementation.

## Current scope

| Capability | Status |
| --- | --- |
| CPE 2.3 formatted-string parsing | Implemented |
| Canonical formatted-string output | Implemented |
| Typed `ANY`, `NA`, and string components | Implemented |
| Literal-versus-wildcard preservation | Implemented |
| Structured errors with byte spans | Implemented |
| Programmatic name construction | Implemented |
| Legacy `cpe:/` URI binding | Planned |
| CPE name matching (NIST IR 7696) | Planned |

The crate does not yet claim complete CPE Naming 2.3 conformance. NIST IR 7695
also recommends consuming legacy URI-bound names, which is intentionally a
separate milestone.

## Parsing

```rust
use custos_cpe::{Component, Cpe, Part};

let input =
    "cpe:2.3:a:microsoft:internet_explorer:8.0.6001:beta:*:*:*:*:*:*";
let cpe: Cpe = input.parse()?;

assert_eq!(cpe.part(), &Component::Value(Part::Application));
assert_eq!(
    cpe.vendor().as_value().unwrap().to_literal_string().as_deref(),
    Some("microsoft")
);
assert_eq!(cpe.to_string(), input);
# Ok::<(), custos_cpe::ParseError>(())
```

Parsing is deliberately strict:

- the prefix is exactly `cpe:2.3:`;
- all eleven fields are required;
- escaped colons are data, not delimiters;
- only printable ASCII accepted by the NIST grammar is allowed;
- `*` is logical `ANY`, while `\*` is a literal asterisk;
- `-` is logical `NA`, while `foo-bar` is a string;
- wildcard placement and the formatted-binding language grammar are checked.

“Permissive” describes the license, not parser behavior.

## Construction

```rust
use custos_cpe::{Cpe, Part, ValueString};

let name = Cpe::builder(Part::Application)
    .vendor(ValueString::literal("example_corp")?)
    .product(ValueString::literal("widget")?)
    .version(ValueString::literal("1.0")?)
    .build();

assert_eq!(
    name.to_string(),
    "cpe:2.3:a:example_corp:widget:1.0:*:*:*:*:*:*:*"
);
assert!(ValueString::literal("example corp").is_err());
# Ok::<(), custos_cpe::ValueError>(())
```

CPE values cannot contain whitespace. Use the normalized value from the
product identifier, such as `example_corp`.

For a parsed wildcard component, [`ValueString::atoms`] exposes the semantic
difference between an unquoted wildcard and an escaped literal character.

## Resource behavior

Parsing is linear in the input length and does not use recursion or
backtracking. A successfully parsed owned name allocates in proportion to its
attribute data; the crate deliberately imposes no arbitrary maximum CPE
length. Applications accepting untrusted data should enforce a request-size
limit appropriate to their environment before parsing.

## Specification sources

- [NIST IR 7695: CPE Naming Specification Version 2.3](https://doi.org/10.6028/NIST.IR.7695)
- [NIST IR 7696: CPE Name Matching Specification Version 2.3](https://doi.org/10.6028/NIST.IR.7696)
- [NIST CPE Naming resources](https://csrc.nist.gov/projects/security-content-automation-protocol/specifications/cpe/naming)

CPE is associated with the U.S. National Institute of Standards and
Technology. This project is independent and is not endorsed by NIST.

## MSRV

The minimum supported Rust version is 1.81.0.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
