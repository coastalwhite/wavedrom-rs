# Releasing

## Tools

```bash
cargo install cargo-dist
cargo install cargo-release
cargo install cargo-hack
cargo install cargo-deny
```

## Formatting

## Running Tests

```bash
cd wavedrom-core
cargo hack --feature-powerset check
cargo hack --feature-powerset test
cargo hack --feature-powerset doc
```

## Linting

```bash
cd wavedrom-core
cargo deny check
cargo clippy
```

## Pushing a new release

Releasing is done with [cargo-dist](https://github.com/axodotdev/cargo-dist) and
[cargo-release](https://github.com/crate-ci/cargo-release).

```rust
cargo release X.Y.Z
```
