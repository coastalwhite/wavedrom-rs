# Command-Line Interface

There is an official Command-Line Interface for wavedrom that can easily be
incorporated into scripts or workflows. This binary is available from the
[GitHub releases](https://github.com/coastalwhite/wavedrom-rs/releases).

A binary can be downloaded from here and be put into a location corresponding to
a operating system.

For Mac and Linux, `/usr/bin/local` is a good place to store the binary.

## Compilation from source

It is possible to compile from the GitHub directory. This generates the most
up-to-date binary, but may be less stable. This requires `git` and the Rust
toolchain `cargo` to be installed.

```bash
git clone https://github.com/coastalwhite/wavedrom-rs
cargo install --path=./wavedrom
```
