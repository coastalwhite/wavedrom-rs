# MdBook-WaveDrom

Integration for [wavedrom-rs] with [mdbook].

## Getting started

First, install the preprocessor utilizing the [Rust Toolchain].

```bash
cargo install mdbook-wavedrom
```

Then, add the preprocessor to the list of preprocessors in your [mdbook] configuration file `book.toml`.

```toml
# book.toml

[preprocessor.wavedrom]
```

Afterwards, you should be able to add a `wavedrom` codeblock, which should automatically get replaced by a [wavedrom-rs] diagram when building the [mdbook].

`````markdown
# Chapter 1

```wavedrom
{
    signal: [
        { name: "clk", wave: "p......." },
        { name: "pulses", wave: "0..10.10" },
    ]
}
```
`````

[Rust Toolchain]: https://www.rust-lang.org/tools/install
[wavedrom-rs]: https://github.com/coastalwhite/wavedrom-rs
[mdbook]: https://rust-lang.github.io/mdBook/