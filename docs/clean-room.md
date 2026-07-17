# Clean-room implementation policy

`custos-cpe` is independently derived from public specifications, primarily
NIST IR 7695. Its implementation must remain usable under the repository's
MIT OR Apache-2.0 license.

Contributors must not copy, translate, or closely paraphrase source code,
comments, tests, fixtures, API documentation, or implementation-specific
behavior from GPL-licensed CPE projects, including `scap-rs`. Such projects
must not be added as dependencies, development dependencies, vendored code,
or differential-test oracles.

Nontrivial parsing behavior should be traceable to a section of a public
specification or to independently authored design rationale. Normative test
vectors may come from the NIST specifications and should identify their
source section.

Public package metadata may be consulted to understand the ecosystem, but
implementation source is outside this project's design inputs.
