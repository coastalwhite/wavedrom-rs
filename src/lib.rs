use std::str::FromStr;

mod path;
mod svg;

#[cfg(feature = "wavejson")]
pub mod wavejson;

use path::PathState;

pub use path::{RenderedWavePath, WaveDimension, WavePath, WavePathSegment};
pub use svg::ToSvg;

pub struct Wave {
    pub name: String,
    pub cycles: Cycles,
}

pub struct Figure(pub Vec<Wave>);
pub struct Cycles(pub Vec<CycleData>);

impl FromStr for Cycles {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cycles = Vec::with_capacity(s.len());

        let mut last_state = None;
        for (i, c) in s.char_indices() {
            let state = match c {
                '1' => CycleData::Top,
                '0' => CycleData::Bottom,
                '2' => CycleData::Box(0),
                '3' => CycleData::Box(1),
                '4' => CycleData::Box(2),
                '5' => CycleData::Box(3),
                '.' => last_state.ok_or(i)?,
                _ => return Err(i),
            };

            last_state = Some(state);
            cycles.push(state)
        }

        Ok(Self(cycles))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleData {
    Top,
    Bottom,
    Box(usize),
}

impl Default for FigurePadding {
    fn default() -> Self {
        Self {
            figure_top: 8,
            figure_bottom: 8,
            figure_left: 8,
            figure_right: 8,

            schema_top: 8,
            schema_bottom: 8,
        }
    }
}

impl Default for FigureSpacing {
    fn default() -> Self {
        Self {
            textbox_to_schema: 16,
            line_to_line: 16,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FigurePadding {
    pub figure_top: u32,
    pub figure_bottom: u32,
    pub figure_left: u32,
    pub figure_right: u32,

    pub schema_top: u32,
    pub schema_bottom: u32,
}

#[derive(Debug, Clone)]
pub struct FigureSpacing {
    pub textbox_to_schema: u32,
    pub line_to_line: u32,
}

impl From<&CycleData> for PathState {
    fn from(value: &CycleData) -> Self {
        match value {
            CycleData::Top => PathState::Top,
            CycleData::Bottom => PathState::Bottom,
            CycleData::Box(usize) => PathState::Box(*usize),
        }
    }
}

pub struct RenderedFigure<'a> {
    options: RenderOptions,

    pub textbox_width: u32,

    pub schema_height: u32,
    pub schema_width: u32,

    font_family: String,

    num_cycles: u32,

    lines: Vec<RenderedLine<'a>>,
}

pub struct RenderedLine<'a> {
    text: &'a str,
    text_width: u32,

    path: WavePath,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub font_size: u32,
    pub paddings: FigurePadding,
    pub spacings: FigureSpacing,
    pub wave_dimensions: WaveDimension,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            font_size: 10,
            paddings: FigurePadding::default(),
            spacings: FigureSpacing::default(),
            wave_dimensions: WaveDimension::default(),
        }
    }
}

impl<'a> RenderedFigure<'a> {
    pub fn width(&self) -> u32 {
        self.paddings().figure_left
            + self.paddings().figure_right
            + self.textbox_width
            + self.schema_width
            + self.spacings().textbox_to_schema
    }

    pub fn height(&self) -> u32 {
        self.paddings().figure_top + self.paddings().figure_bottom + self.schema_height
    }

    pub fn paddings(&self) -> &FigurePadding {
        &self.options.paddings
    }

    pub fn spacings(&self) -> &FigureSpacing {
        &self.options.spacings
    }

    pub fn wave_dimensions(&self) -> &WaveDimension {
        &self.options.wave_dimensions
    }
}

impl Figure {
    pub fn render_with_options(&self, options: &RenderOptions) -> Result<RenderedFigure, ()> {
        let RenderOptions {
            font_size,
            paddings,
            spacings,
            wave_dimensions,
        } = options;

        let num_lines = u32::try_from(self.0.len()).map_err(|_| ())?;

        let face =
            // ttf_parser::Face::parse(include_bytes!("../JetBrainsMono-Medium.ttf"), 0).unwrap();
            ttf_parser::Face::parse(include_bytes!("/usr/share/fonts/noto/NotoSansMono-Regular.ttf"), 0).unwrap();

        let font_family = get_font_family_name(&face)
            .map_or_else(|| "monospace".to_string(), |s| format!("{s}, monospace"));

        let lines = self
            .0
            .iter()
            .map(|wave| RenderedLine {
                text: &wave.name,
                text_width: wave.get_text_width(&face, *font_size),

                path: WavePath::new(wave.cycles.0.iter().map(PathState::from).collect()),
            })
            .collect::<Vec<RenderedLine>>();

        let num_cycles = lines.iter().map(|line| line.path.len()).max().unwrap_or(0);
        let num_cycles = u32::try_from(num_cycles).map_err(|_| ())?;

        let textbox_width = lines
            .iter()
            .map(|line| line.text_width)
            .max()
            .unwrap_or_default();

        // Error when there are too many cycles
        let schema_width = num_cycles * u32::from(wave_dimensions.cycle_width);

        let schema_height = if num_lines == 0 {
            0
        } else {
            paddings.schema_top
                + paddings.schema_bottom
                + spacings.line_to_line * (num_lines - 1)
                + u32::from(wave_dimensions.wave_height) * num_lines
        };

        // TODO: Does this clone need to be here?
        let options = options.clone();

        Ok(RenderedFigure {
            options,

            textbox_width,

            schema_width,
            schema_height,

            font_family,

            num_cycles,

            lines,
        })
    }

    #[inline]
    pub fn render(&self) -> Result<RenderedFigure, ()> {
        self.render_with_options(&RenderOptions::default())
    }
}

impl Wave {
    fn get_text_width(&self, face: &ttf_parser::Face, font_size: u32) -> u32 {
        let width = self.name
            .chars()
            .map(|c| {
                face.glyph_index(c).map_or_else(|| {
                        eprintln!("[WARNING]: Failed to get glyph for '{c}'");
                        0
                }, |g| {
                    u32::from(face.glyph_hor_advance(g).unwrap_or_else(|| {
                        eprintln!(
                            "[WARNING]: Failed to get length for glyph '{}' that represents character '{c}'",
                            face.glyph_name(g).unwrap_or(&c.to_string())
                        );
                        0
                    }))
                })
            })
            .sum::<u32>();

        let width = f64::from(width);

        // NOTE: Face::units_per_em guarantees the value to be non-zero. So this should never
        // generate a divide-by-zero error.
        let pts_per_em = f64::from(font_size) / f64::from(face.units_per_em());
        let width = width * pts_per_em;

        width.ceil() as u32
    }
}

fn name_to_string(name: ttf_parser::name::Name) -> Option<String> {
    if !name.is_unicode() {
        return None;
    }

    // Invalid UTF16 check
    if name.name.len() % 2 != 0 {
        return None;
    }

    let utf16_bytes = name
        .name
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<u16>>();

    String::from_utf16(&utf16_bytes).ok()
}

fn get_font_family_name(face: &ttf_parser::Face) -> Option<String> {
    for item in face.names() {
        if item.name_id == 1 {
            return name_to_string(item);
        }
    }

    None
}
