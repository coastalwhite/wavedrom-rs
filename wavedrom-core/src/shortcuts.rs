#[cfg(all(feature = "serde", feature = "json5"))]
pub use self::json5::*;

#[cfg(all(feature = "serde", feature = "serde_json"))]
pub use self::serde_json::*;

#[cfg(all(feature = "serde", feature = "json5"))]
mod json5 {
    use std::error::Error;
    use std::fmt::Display;
    use std::io;

    use crate::svg::options::RenderOptions;

    #[derive(Debug)]
    pub enum RenderJson5Error {
        Json(json5::Error),
        Io(io::Error),
    }

    pub fn render_json5(json: &str, writer: &mut impl io::Write) -> Result<(), RenderJson5Error> {
        render_json5_with_options(json, writer, &RenderOptions::default())
    }

    pub fn render_json5_with_options(
        json: &str,
        writer: &mut impl io::Write,
        options: &RenderOptions,
    ) -> Result<(), RenderJson5Error> {
        use crate::svg::ToSvg;

        let figure = crate::Figure::from_json5(json)?;
        let assembled = figure.assemble_with_options(&options.wave_dimensions);
        assembled.write_svg_with_options(writer, options)?;

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

    use crate::svg::options::RenderOptions;

    pub fn render_json(json: &str, writer: &mut impl io::Write) -> Result<(), RenderJsonError> {
        render_json_with_options(json, writer, &RenderOptions::default())
    }

    pub fn render_json_with_options(
        json: &str,
        writer: &mut impl io::Write,
        options: &RenderOptions,
    ) -> Result<(), RenderJsonError> {
        use crate::svg::ToSvg;

        let figure = crate::Figure::from_json(json)?;
        let assembled = figure.assemble_with_options(&options.wave_dimensions);
        assembled.write_svg_with_options(writer, options)?;

        Ok(())
    }

    #[derive(Debug)]
    pub enum RenderJsonError {
        Json(serde_json::error::Error),
        Io(io::Error),
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
