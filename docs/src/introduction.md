# WaveDrom-rs

This is the book for [`wavedrom-rs`][github]. It provides a reference on the
features of `wavedrom-rs` and aims to be an extension of the original
[Hitcherhiker's Guide to WaveDrom][tutorial].

This book teaches you how to generate beautiful [Digital Timing
Diagrams][dtd-wiki] such as the one illustrated below. These diagrams are easy
to edit and extend.

```wavedrom[with_source]
{
    signal: [
        { name: "clk",  wave: "p........." },
        { name: "req",  wave: "010......." },
        { name: "done", wave: "0......10." },
        {
            name: "state",
            wave: "==.=.=.==.",
            data: [ 'Idle', 'Fetch', 'Calculate', 'Process', 'Return', 'Idle' ]
        }
    ]
}
```

This book has sections.

1. Installation of `wavedrom-rs` for different environments
2. Capabilities of `wavedrom-rs`


[github]: https://github.com/coastalwhite/wavedrom-rs
[tutorial]: https://wavedrom.com/tutorial.html
[dtd-wiki]: https://en.wikipedia.org/wiki/Digital_timing_diagram
