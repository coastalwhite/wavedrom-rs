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

# License

Licensed under a [MIT License](./LICENSE). The demo website utilizes icons from
[Lucide][lucide] which are licensed under an ISC license.

[demo]: https://gburghoorn.com/wavedrom
[svg]: https://en.wikipedia.org/wiki/SVG
[wavedrom-js]: https://wavedrom.com/
[tutorial]: https://wavedrom.com/tutorial.html
[lucide]: https://lucide.dev/