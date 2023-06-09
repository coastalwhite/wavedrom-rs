use std::fmt::Display;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use serde_json::error::Error as JsonError;
use wavedrom::{svg::ToSvg, wavejson::WaveJson, Figure};

#[derive(Debug)]
pub enum InsertionError {
    Json(JsonError),
    InvalidFigure,
    Assemble,
    WriteSvg,
    InvalidUtf8,
}

impl From<JsonError> for InsertionError {
    #[inline]
    fn from(error: JsonError) -> Self {
        InsertionError::Json(error)
    }
}

impl Display for InsertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(err) => write!(f, "{err}"),
            Self::InvalidFigure => write!(f, "Failed to form a figure from the given WaveJson"),
            Self::Assemble => write!(f, "Failed to assemble WaveDrom figure"),
            Self::WriteSvg => write!(f, "Failed to write svg of WaveDrom figure"),
            Self::InvalidUtf8 => write!(f, "Wavedrom returned invalid UTF-8"),
        }
    }
}

impl std::error::Error for InsertionError {}

pub fn insert_wavedrom(content: &str) -> Result<String, InsertionError> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mut diagrams = Vec::new();
    let mut current_spans = Vec::new();
    let mut wavedrom_block_start = None;

    for (e, span) in Parser::new_ext(content, opts).into_offset_iter() {
        if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(code))) = e {
            if code.as_ref() == "wavedrom" {
                wavedrom_block_start = Some(span.start);
            }

            continue;
        }

        let Some(block_start) = wavedrom_block_start else {
            continue;
        };

        // We're in the code block. The text is what we want.
        // Code blocks can come in multiple text events.
        if let Event::Text(_) = e {
            current_spans.push(span);
            continue;
        }

        if let Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(code))) = e {
            assert_eq!(
                "wavedrom",
                code.as_ref(),
                "After an opening a wavedrom code block we expect it to close again"
            );
            let block_end = span.end;

            let mut diagram_content = String::with_capacity(block_end - block_start);
            for span in std::mem::take(&mut current_spans).into_iter() {
                diagram_content.push_str(&content[span]);
            }

            let mut wavedrom_code = Vec::new();

            let wavejson = WaveJson::from_str(&diagram_content)?;
            let wavedrom_figure =
                Figure::try_from(wavejson).map_err(|_| InsertionError::InvalidFigure)?;

            wavedrom_figure
                .assemble()
                .map_err(|_| InsertionError::Assemble)?
                .write_svg(&mut wavedrom_code)
                .map_err(|_| InsertionError::WriteSvg)?;

            let wavedrom_code =
                String::from_utf8(wavedrom_code).map_err(|_| InsertionError::InvalidUtf8)?;

            diagrams.push((block_start..block_end, wavedrom_code));
        }
    }

    let mut out = String::with_capacity(content.len());
    let mut end_prev = 0;

    for (span, block) in diagrams.into_iter() {
        out.push_str(&content[end_prev..span.start]);
        out.push_str(r#"<pre class="wavedrom">"#);
        out.push_str(&block);
        out.push_str("</pre>\n\n");

        end_prev = span.end;
    }

    out.push_str(&content[end_prev..]);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_insertion() {
        let content = r#"
# Header

```wavedrom
{
    "signal": []
}
```
        "#;

        let replaced_content = insert_wavedrom(content).unwrap();
        assert_ne!(content, &replaced_content);
        assert!(replaced_content.contains(r#"<pre class="wavedrom">"#));
        assert!(replaced_content.contains("</svg>"));
    }
}
