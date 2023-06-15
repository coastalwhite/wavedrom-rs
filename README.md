<center>
    <h1>wavedrom-rs 🌊</h1>
    <h3>A Rust 🦀 <b>Signal Diagram</b> generator based on <a href="https://wavedrom.com/">WaveDrom</a></h3>
</center>

---

[Demo][demo] | [Tutorial][tutorial]

The `wavedrom-rs` crate provides an interface to shape beautiful *Signal
Diagrams* into an [SVG][svg]. It aims to be mostly compatible with the
[WaveDrom][wavedrom-js] project. Namely, it allows all the signal types that
WaveDrom allows and has support for [WaveJson][tutorial] shaping.

There are currently four ways to use `wavedrom-rs`:

- [On the editor website][demo]
- [As a Rust crate][cratesio]
- [As a Command-Line Application][cli]
- [As a MdBook Preprocessor][mdbook-wavedrom]

## Features

- [x] All wavedrom-rs Signal Types
- [x] Signal Groups
- [x] Marker Edges
- [x] Header and Footer Text
- [x] Cycle Enumeration Markers
- [x] Editor Website through WASM
- [x] MdBook Preprocessor
- [ ] Full Customization via Skin

## Documentation

This project uses the same syntax for wavejson as the original
[WaveDrom][wavedrom-js] project. Therefore, the original [Hitchhiker's Guide to
WaveDrom][hitchhiker] is still a good introduction into several concepts of
of WaveJson. Additionally, this repository maintains a [book][book].

## Testing

Tests are written in the [`./tests`](./tests) directory and can be generated
using the [`./tests/run.py`](./tests/run.py) scripts. This will generate a
`result.html` in the `tests` directory that contains all the rendered SVGs for
each `json5` file in the `./tests` directory.

## License

Licensed under a [MIT License](./LICENSE). The demo website utilizes icons from
[Lucide][lucide] which are licensed under an ISC license.

[demo]: https://gburghoorn.com/wavedrom
[svg]: https://en.wikipedia.org/wiki/SVG
[wavedrom-js]: https://wavedrom.com/
[tutorial]: https://wavedrom.com/tutorial.html
[lucide]: https://lucide.dev/
[hitchhiker]: https://wavedrom.com/tutorial.html
[book]: https://coastalwhite.github.io/wavedrom-rs