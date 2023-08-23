//! WaveDrom allows for the programmatic creation of beautiful [Diagram Timing Diagrams][dtd] in
//! Rust. This is the crate that powers all the wavedrom tools including the [editor], the
//! [command-line interface][cli], and the [mdbook preprocessor][mdbook-wavedrom].
//!
//! This crate is be used in two ways. It can be given [WaveJson][wavejson] which is a JSON format
//! to describe [Diagram Timing Diagrams][dtd]. Alternatively, you can programmatically define a
//! figure by building it using the [`Figure`] struct.
//!
//! # Getting Started
//!
//! Getting started with this crate is quite easy. Here, we have two examples. First, how to use
//! [WaveJson][wavejson] as an input to your figures and second how to programmically define
//! figures.
//!
//! ## WaveJson
//!
#![cfg_attr(
    feature = "json5",
    doc = r####"
```
use std::fs::File;

let path = "path/to/file.svg";
# let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/doc-root-wavejson.svg");
let mut file = File::create(path)?;

wavedrom::render_json5(r##"
    { signal: [
        { name: "clk",  wave: "P......" },
        { name: "bus",  wave: "x.==.=x", data: ["head", "body", "tail", "data"] },
        { name: "wire", wave: "0.1..0." }
    ]}
"##, &mut file)?;
# <Result<(), wavedrom::RenderJson5Error>>::Ok(())
```"####
)]
//!
//! **Result:**
//!
#![doc=include_str!("../assets/doc-root-wavejson.svg")]
//!
//! ## Programmically defining a Figure
//!
//! ```
//! use std::fs::File;
//! use wavedrom::{Figure, Signal};
//!
//! let figure = Figure::new()
//!                  .header_text("Timing Schema")
//!                  .add_signals([
//!                      Signal::with_cycle_str("p........").name("clk"),
//!                      Signal::with_cycle_str("010......").name("req"),
//!                      Signal::with_cycle_str("0......10").name("done"),
//!                      Signal::with_cycle_str("0......10").name("done"),
//!                      Signal::with_cycle_str("==.=.=.=.").name("state")
//!                         .add_data_fields([
//!                             "Idle", "Fetch", "Calculate", "Return", "Idle",
//!                         ]),
//!                  ]);
//! let assembled_figure = figure.assemble();
//!
//! let path = "path/to/file.svg";
//! # let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/doc-root-programatically.svg");
//! let mut file = File::create(path)?;
//!
//! assembled_figure.write_svg(&mut file)?;
//! # <Result<(), std::io::Error>>::Ok(())
//! ```
//!
//! **Result:**
//!
#![doc=include_str!("../assets/doc-root-programatically.svg")]
//!
//! # Cargo Features
//!
//! There are a set of cargo features, most of which are enabled by default.
//!
//! * `serde`. Enabled by default. Adds the [`wavejson`] module, which defines the serialize and
//! deserialize formats for a wave format for a wave.
//! * `embed_font`. Enabled by default. Adds an embedded [Helvetica][helvetica] into the library
//! which is used to find the dimensions of certain texts. When this is disabled, it is replaced by
//! a width look-up table that is only accurate for ASCII and over-estimates the width for other
//! UTF-8 characters.
//! * `json5`. Enabled by default. The human friendly variant of JSON that can be used with the
//! `serde` feature to deserialize a WaveJson file.
//! * `serde_json`. Disabled by default. Formal version of JSON that can be used with the `serde`
//! feature to deserialize a WaveJson file.
//! * `skins`. Enabled by default. Adds the [`skin`] module, which defines the serialize and
//! deserialize formats for WaveDrom skins. Also adds logic to merge a skin into an existing set of
//! options.
//!
//! # Rendering Process
//!
//! The rendering process of this crate is done in 3 steps.
//!
//! **1. Create [`Figure`]**
//!
//! A [`Figure`] can be created in two ways. First, a [`Figure`] can be built programmatically with
//! the [`Figure::new`] method and the builder pattern methods. Second, a [`Figure`] can be built
//! by loading a [WaveJson][wavejson] file. This can be done with the [`Figure::from_json5`] or
//! [`Figure::from_json`] methods.
//!
//! **2. Assemble [`Figure`] to [`AssembledFigure`]**
//!
//! A [`Figure`] needs to be assembled. This shapes the signal waves removes any invalid groups and
//! edges. Assembling is done with the [`Figure::assemble`] and [`Figure::assemble_with_options`]
//! methods.
//!
//! **3. Render [`AssembledFigure`] to SVG**
//!
//! An [`AssembledFigure`] can be rendered by calling the [`AssembledFigure::write_svg`] or
//! [`AssembledFigure::write_svg_with_options`] methods. This will write an SVG into an
//! [`io::Write`][std::io::Write] buffer. If a write to the [`io::Write`][std::io::Write] is
//! expensive, it is recommended to wrap the [`io::Write`][std::io::Write] in a
//! [`std::io::BufWriter`].
//!
//! [helvetica]: https://en.wikipedia.org/wiki/Helvetica
//! [dtd]: https://en.wikipedia.org/wiki/Digital_timing_diagram
//! [editor]: https://gburghoorn.com/wavedrom
//! [cli]: https://github.com/coastalwhite/wavedrom-rs/tree/main/wavedrom
//! [mdbook-wavedrom]: https://github.com/coastalwhite/wavedrom-rs/tree/main/mdbook-wavedrom

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(
    all(
        feature = "serde_json",
        feature = "json5",
        feature = "serde",
        feature = "skins"
    ),
    deny(rustdoc::broken_intra_doc_links)
)]
#![deny(missing_docs)]

#[cfg(feature = "json5")]
pub use json5;

#[cfg(feature = "serde_json")]
pub use serde_json;

#[cfg(feature = "skins")]
pub mod skin;

mod color;
mod font;
mod shortcuts;
pub mod signal;
mod escape;
 
pub use font::Font;
pub use color::Color;
pub use shortcuts::*;

use self::signal::SignalFigure;

#[cfg(feature = "serde")]
pub mod wavejson;

/// A general wavedrom figure
pub enum Figure {
    /// A figure containing a set of signals
    Signal(SignalFigure),
}
