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
//! use wavedrom::Figure;
//! use wavedrom::signal::{Signal, SignalFigure};
//!
//! let figure = SignalFigure::new()
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
//!
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
// #![deny(missing_docs)]

#[cfg(feature = "json5")]
pub use json5;

#[cfg(feature = "serde_json")]
pub use serde_json;

#[cfg(feature = "skins")]
pub mod skin;

mod color;
mod font;
mod shortcuts;
mod escape;

#[macro_use]
mod options;

pub mod signal;
pub mod reg;
 
pub use font::Font;
pub use color::Color;
pub use shortcuts::*;

use self::reg::RegisterFigure;
use self::signal::SignalFigure;

define_options! {
    Options,

    PartialOptions {
        /// The figure background
        background: Option<Color> => Some(Color::WHITE),
        /// The figure's paddings
        padding: FigurePadding[PartialFigurePadding],
        /// The figure's spacings
        spacing: FigureSpacing[PartialFigureSpacing],
        /// The figure's header options
        header: HeaderOptions[PartialHeaderOptions],
        /// The figure's footer options
        footer: FooterOptions[PartialFooterOptions],

        /// The options specific to signal figures.
        signal: signal::options::SignalOptions[signal::options::PartialSignalOptions],
        /// The options specific to register figures.
        reg: reg::options::RegisterOptions[reg::options::PartialRegisterOptions],

        /// The background colors for the Box2 to Box9 states
        backgrounds: [Color; 8] => [
                Color { red: 0xFF, green: 0xFF, blue: 0xFF },
                Color { red: 0xF7, green: 0xF7, blue: 0xA1 },
                Color { red: 0xF9, green: 0xD4, blue: 0x9F },
                Color { red: 0xAD, green: 0xDE, blue: 0xFF },
                Color { red: 0xAC, green: 0xD5, blue: 0xB6 },
                Color { red: 0xA4, green: 0xAB, blue: 0xE1 },
                Color { red: 0xE8, green: 0xA8, blue: 0xF0 },
                Color { red: 0xFB, green: 0xDA, blue: 0xDA },
        ],

        /// The background color of the undefined background pattern
        undefined_background: Option<Color> => None,
    }
}

define_options! {
    /// The paddings of the figure
    FigurePadding,

    /// A subset of [`FigurePadding`]
    PartialFigurePadding {
        /// The padding at the top of the figure
        figure_top: u32 => 8,
        /// The padding at the bottom of the figure
        figure_bottom: u32 => 8,
        /// The padding at the left of the figure
        figure_left: u32 => 8,
        /// The padding at the right of the figure
        figure_right: u32 => 8,

        /// The padding at the top of the signal schema
        schema_top: u32 => 8,
        /// The padding at the bottom of the signal schema
        schema_bottom: u32 => 8,
    }
}

define_options! {
    /// The spacings for the figure
    FigureSpacing,

    /// A subset of [`FigureSpacing`]
    PartialFigureSpacing {
        /// The spacing between the signal names and the signal schema
        textbox_to_schema: u32 => 8,
        /// The spacing group indicators and the signal names
        groupbox_to_textbox: u32 => 8,
        /// The between signal lines
        line_to_line: u32 => 8,
    }
}

define_options! {
    /// The header options for the figure
    HeaderOptions,

    /// A subset of [`HeaderOptions`]
    PartialHeaderOptions {
        /// The header font size
        font_size: u32 => 24,
        /// The header height
        height: u32 => 32,
        /// The header text color
        color: Color => Color::BLACK,

        /// The cycle enumeration marker height
        cycle_marker_height: u32 => 12,
        /// The cycle enumeration marker font size
        cycle_marker_fontsize: u32 => 12,
        /// The cycle enumeration marker text color
        cycle_marker_color: Color => Color::BLACK,
    }
}

define_options! {
    /// The footer options for the figure
    FooterOptions,

    /// A subset of [`FooterOptions`]
    PartialFooterOptions {
        /// The footer font size
        font_size: u32 => 24,
        /// The footer height
        height: u32 => 32,
        /// The footer text color
        color: Color => Color::BLACK,

        /// The cycle enumeration marker height
        cycle_marker_height: u32 => 12,
        /// The cycle enumeration marker font size
        cycle_marker_fontsize: u32 => 12,
        /// The cycle enumeration marker text color
        cycle_marker_color: Color => Color::BLACK,
    }
}


#[cfg(feature = "serde")]
pub mod wavejson;

/// A general wavedrom figure
pub enum Figure {
    /// A figure containing a set of signals
    Signal(SignalFigure),

    Register(RegisterFigure),
}
