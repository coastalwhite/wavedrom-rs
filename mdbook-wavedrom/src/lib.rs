use std::fmt::Display;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use wavedrom::json5::Error as JsonError;
use wavedrom::options::RenderOptions;
use wavedrom::Figure;
use wavedrom::PathAssembleOptions;

#[derive(Debug)]
pub enum InsertionError {
    Json(JsonError),
    InvalidFigure,
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
            Self::WriteSvg => write!(f, "Failed to write svg of WaveDrom figure"),
            Self::InvalidUtf8 => write!(f, "Wavedrom returned invalid UTF-8"),
        }
    }
}

impl std::error::Error for InsertionError {}

pub fn insert_wavedrom(
    content: &str,
    assemble_options: PathAssembleOptions,
    render_options: &RenderOptions,
) -> Result<String, InsertionError> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mut diagrams = Vec::new();
    let mut current_spans = Vec::new();
    let mut wavedrom_block_start = None;
    let mut keep_source_code_tag = None;

    for (e, span) in Parser::new_ext(content, opts).into_offset_iter() {
        if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(code))) = e {
            if code.as_ref().trim_start().starts_with("wavedrom") {
                if code.as_ref().trim() == "wavedrom" {
                    wavedrom_block_start = Some(span.start);
                } else {
                    let rest = &code.as_ref().trim_start()["wavedrom".len()..];
                    let rest = rest.trim();

                    if rest.starts_with('[') && rest.ends_with(']') {
                        let control_flags = rest[1..rest.len() - 1].trim();
                        for control_flag in control_flags.split(',') {
                            if control_flag.trim() == "with_source" {
                                keep_source_code_tag = Some(
                                    span.start
                                        ..span.start
                                            + content[span.start..]
                                                .find('\n')
                                                .unwrap_or(span.end),
                                )
                            }
                        }

                        wavedrom_block_start = Some(span.start);
                    }
                }
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
            assert!(
                code.as_ref().trim_start().starts_with("wavedrom"),
                "After an opening a wavedrom code block we expect it to close again"
            );
            let block_end = span.end;

            let mut diagram_content = String::with_capacity(block_end - block_start);
            for span in std::mem::take(&mut current_spans).into_iter() {
                diagram_content.push_str(&content[span]);
            }

            let mut wavedrom_code = Vec::new();

            let wavedrom_figure =
                Figure::from_json5(&diagram_content).map_err(|_| InsertionError::InvalidFigure)?;

            wavedrom_figure
                .assemble_with_options(assemble_options)
                .write_svg_with_options(&mut wavedrom_code, render_options)
                .map_err(|_| InsertionError::WriteSvg)?;

            let wavedrom_code =
                String::from_utf8(wavedrom_code).map_err(|_| InsertionError::InvalidUtf8)?;

            diagrams.push((
                block_start..block_end,
                wavedrom_code,
                keep_source_code_tag.take(),
            ));
        }
    }

    let mut out = String::with_capacity(content.len());
    let mut end_prev = 0;

    for (span, block, keep_source_code) in diagrams.into_iter() {
        out.push_str(&content[end_prev..span.start]);
        out.push_str(r#"<pre class="wavedrom">"#);
        out.push_str(&block);
        out.push_str("</pre>\n\n");

        if let Some(tag_span) = keep_source_code {
            out.push_str(&content[span.start..tag_span.start]);
            out.push_str("```json");
            out.push_str(&content[tag_span.end..span.end]);
        }

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

        let replaced_content = insert_wavedrom(
            content,
            PathAssembleOptions::default(),
            &RenderOptions::default(),
        )
        .unwrap();
        assert_ne!(content, &replaced_content);
        assert!(replaced_content.contains(r#"<pre class="wavedrom">"#));
        assert!(replaced_content.contains("</svg>"));
    }
}
