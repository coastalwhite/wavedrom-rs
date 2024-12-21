#[cfg(feature = "json5")]
pub use self::json5::*;

#[cfg(feature = "serde_json")]
pub use self::serde_json::*;

#[cfg(feature = "json5")]
mod json5 {
    use std::error::Error;
    use std::fmt::Display;
    use std::io;

    use crate::{Figure, Options};

    /// An error with the [`render_json5`][crate::render_json5] or
    /// [`render_json5_with_options`][crate::render_json5_with_options] functions.
    #[derive(Debug)]
    pub enum RenderJson5Error {
        /// An error parsing the JSON
        Json(json5::Error),
        /// An error with the IO
        Io(io::Error),
    }

    /// Render the contents of a json5 file to a `writer`.
    #[inline]
    pub fn render_json5(json: &str, writer: &mut impl io::Write) -> Result<(), RenderJson5Error> {
        render_json5_with_options(json, writer, &Options::default())
    }

    /// Render the contents of a json5 file to a `writer` with a specific set of options.
    pub fn render_json5_with_options(
        json: &str,
        writer: &mut impl io::Write,
        options: &Options,
    ) -> Result<(), RenderJson5Error> {
        let figure = Figure::from_json5(json)?;

        match figure {
            Figure::Signal(figure) => {
                let assembled = figure.assemble_with_options(options);
                assembled.write_svg_with_options(writer, options)?;
            }
            Figure::Register(register) => {
                register.write_svg(writer)?;
            }
        }

        Ok(())
    }

    impl From<json5::Error> for RenderJson5Error {
        #[inline]
        fn from(error: json5::Error) -> Self {
            Self::Json(error)
        }
    }

    impl From<io::Error> for RenderJson5Error {
        #[inline]
        fn from(error: io::Error) -> Self {
            Self::Io(error)
        }
    }

    impl Display for RenderJson5Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Json(err) => err.fmt(f),
                Self::Io(err) => err.fmt(f),
            }
        }
    }

    impl Error for RenderJson5Error {}
}

#[cfg(all(feature = "serde", feature = "serde_json"))]
mod serde_json {
    use std::error::Error;
    use std::fmt::Display;
    use std::io;

    use crate::Figure;
    use crate::Options;

    /// An error with the [`render_json`][crate::render_json] or
    /// [`render_json_with_options`][crate::render_json_with_options] functions.
    #[derive(Debug)]
    pub enum RenderJsonError {
        /// An error parsing the JSON
        Json(serde_json::error::Error),
        /// An error with the IO
        Io(io::Error),
    }

    /// Render the contents of a json file to a `writer`.
    #[inline]
    pub fn render_json(json: &str, writer: &mut impl io::Write) -> Result<(), RenderJsonError> {
        render_json_with_options(json, writer, &Options::default())
    }

    /// Render the contents of a json file to a `writer` with a specific set of options.
    pub fn render_json_with_options(
        json: &str,
        writer: &mut impl io::Write,
        options: &Options,
    ) -> Result<(), RenderJsonError> {
        let figure = Figure::from_json(json)?;

        match figure {
            Figure::Signal(figure) => {
                let assembled = figure.assemble_with_options(options);
                assembled.write_svg_with_options(writer, options)?;
            }
            Figure::Register(register) => {
                register.write_svg(writer)?;
            }
        }

        Ok(())
    }

    impl From<serde_json::error::Error> for RenderJsonError {
        #[inline]
        fn from(error: serde_json::error::Error) -> Self {
            Self::Json(error)
        }
    }

    impl From<io::Error> for RenderJsonError {
        #[inline]
        fn from(error: io::Error) -> Self {
            Self::Io(error)
        }
    }

    impl Display for RenderJsonError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Json(err) => err.fmt(f),
                Self::Io(err) => err.fmt(f),
            }
        }
    }

    impl Error for RenderJsonError {}
}
