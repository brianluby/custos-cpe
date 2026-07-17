# Contributing

Run the full local verification before submitting a change:

```console
cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo test --all-targets --locked
cargo test --doc --locked
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --manifest-path fuzz/Cargo.toml --bins --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
cargo check --manifest-path fuzz/Cargo.toml --bins --locked
```

Run a parser or round-trip fuzz campaign with `cargo-fuzz` installed:

```console
mkdir -p fuzz/corpus/parse_formatted_utf8 fuzz/corpus/formatted_round_trip
cargo fuzz run parse_formatted_utf8 fuzz/corpus/parse_formatted_utf8 fuzz/seeds
cargo fuzz run formatted_round_trip fuzz/corpus/formatted_round_trip fuzz/seeds
```

All code must follow the [clean-room implementation policy](docs/clean-room.md).
In particular, do not copy from or use GPL CPE implementations as test
oracles. Cite the applicable NIST specification section in tests for subtle
grammar or binding behavior.
