use std::fmt::Display;
use std::io::{stdin, stdout, BufWriter, Read, Write};
use std::path::PathBuf;
use std::path::Path;

use usvg::TreeParsing;
use usvg::TreeTextToPath;

use wavedrom::signal::options::{RenderOptions, PathAssembleOptions};
use wavedrom::skin::Skin;
use wavedrom::Figure;

#[derive(Default)]
struct Flags {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    skin: Option<PathBuf>,
    png_scale: Option<String>,
}

enum ParsingError {
    MissingArgument(String),
    UnexpectedArgument(String),
    InvalidFlag(String),
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::MissingArgument(arg) => {
                write!(f, "An argument for flag '{arg}' is missing")
            }
            ParsingError::UnexpectedArgument(arg) => {
                write!(f, "The argument '{arg}' is unexpected")
            }
            ParsingError::InvalidFlag(arg) => write!(f, "Flag '{arg}' is not valid"),
        }
    }
}

impl Flags {
    fn print_metadata() {
        let pkg_name = env!("CARGO_PKG_NAME");
        let pkg_version = env!("CARGO_PKG_VERSION");
        let pkg_authors = env!("CARGO_PKG_AUTHORS");

        println!("{pkg_name} {pkg_version}");
        println!("{pkg_authors}");
    }

    fn about() -> &'static str {
        r#"
A Signal Diagram Generator from WaveJson.



By default, this application attempts to read a file from STDIN and output to
STDOUT. An input file can be given with the -i/--input flag and a output file
can be passed with the -o/--output file.
        "#
        .trim()
    }

    fn usage() -> &'static str {
        r"
Usage: wavedrom [FLAGS]

Takes a wavejson file from the STDIN and outputs a SVG to the STDOUT.

Flags:
-i/--input <path/to/input.json>: specify a path to a input wavejson file.
-o/--output <path/to/output.svg>: specify a path to a output svg or png file.
-s/--skin <path/to/skin.json>: specify a path to a skin file.
-p/--png_scale floating-point value to adjust the resolution of the output png file.
        "
        .trim()
    }

    fn get() -> Result<Self, ParsingError> {
        let mut args = std::env::args().skip(1);
        let mut flags = Flags::default();

        loop {
            let Some(arg) = args.next() else {
                break;
            };

            match &arg[..] {
                "-i" | "--input" => {
                    flags.input = Some(
                        args.next()
                            .ok_or(ParsingError::MissingArgument(arg))?
                            .into(),
                    );
                }
                "-o" | "--output" => {
                    flags.output = Some(
                        args.next()
                            .ok_or(ParsingError::MissingArgument(arg))?
                            .into(),
                    );
                }
                "-s" | "--skin" => {
                    flags.skin = Some(
                        args.next()
                            .ok_or(ParsingError::MissingArgument(arg))?
                            .into(),
                    );
                }
                "-p" | "--png_scale" => {
                    flags.png_scale = Some(
                        args.next()
                        .ok_or(ParsingError::MissingArgument(arg))?
                        .into(),
                    );
                }
                "-h" | "--help" => {
                    Self::print_metadata();
                    println!();
                    println!("{}", Self::about());
                    println!();
                    println!("{}", Self::usage());

                    std::process::exit(0);
                }
                s if s.starts_with("-") => return Err(ParsingError::InvalidFlag(arg)),
                _ => return Err(ParsingError::UnexpectedArgument(arg)),
            }
        }

        Ok(flags)
    }
}



fn get_file_extension(file_path: &str) -> Option<&str> {
    if let Some(extension) = Path::new(file_path).extension() {
        extension.to_str()
    } else {
        None
    }
}

fn export_svg_file(output_path:&PathBuf, svg_data:&[u8]) -> std::result::Result<(), std::io::Error>{
    let output_file = match std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&output_path)
                        {
                            Ok(f) => f,
                            Err(err) => {
                                eprintln!("[ERROR]: Failed to open output file. Reason: {err}");
                                std::process::exit(1);
                            }
                        };
                        let mut writer = BufWriter::new(output_file);
                        match writer.write(svg_data) {
                            Ok(_s) => {
                                Ok(())
                            },
                            Err(e) => {
                                eprintln!("[ERROR]: Error writing svg file: {e:?}.");
                                Err(e)
                            }
                        }
}

fn export_png_file(flags: &Flags, output_path:&PathBuf, svg_data:&[u8]) -> std::result::Result<(), std::io::Error>{
    let opt = usvg::Options::default();
    let mut utree = usvg::Tree::from_data(&svg_data, &opt).unwrap();
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    utree.convert_text(&fontdb);
    let rtree = resvg::Tree::from_usvg(&utree);
    let size = rtree.size.to_int_size();

    let scale = match &flags.png_scale {
        None => 1.0,
        Some(scale) => {
            match scale.parse::<f32>() {
                Err(e) => {
                    eprintln!("[ERROR]: Error parsing png_scale: {e:?}.");
                    std::process::exit(1)
                }
                Ok(parsed_scale) => parsed_scale
            }
        }
    };

    let mut pixmap = resvg::tiny_skia::Pixmap::new((size.width() as f32 * scale) as u32, (size.height() as f32 * scale) as u32).unwrap();
    rtree.render(usvg::Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    match pixmap.save_png(&output_path) {
        Err(e) => { 
            eprintln!("[ERROR]: Error Encoding png: {e:?}.");
            std::process::exit(1)
        }
        Ok(()) => Ok(())
    }
}

fn main() {
    let flags = Flags::get().unwrap_or_else(|err| {
        eprintln!("[ERROR]: {err}");
        eprintln!();
        eprintln!("{}", Flags::usage());
        std::process::exit(1);
    });

    let content = match flags.input {
        None => {
            let mut buffer = Vec::new();
            let mut stdin = stdin().lock();
            match stdin.read_to_end(&mut buffer) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("[ERROR]: Failed to read stdin until end. Reason: {err}");
                    std::process::exit(1);
                }
            }

            match String::from_utf8(buffer) {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("[ERROR]: Stdin does not contain valid UTF-8. Reason: {err}");
                    std::process::exit(1);
                }
            }
        }
        Some(ref input_path) => match std::fs::read_to_string(input_path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("[ERROR]: Failed to read content from file. Reason: {err}");
                std::process::exit(1);
            }
        },
    };

    let (assemble_options, render_options) = match flags.skin {
        None => (PathAssembleOptions::default(), RenderOptions::default()),
        Some(ref skin_path) => {
            let skin = match std::fs::read_to_string(skin_path) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("[ERROR]: Failed to read content from skin file. Reason: {err}");
                    std::process::exit(1);
                }
            };

            match Skin::from_json5(&skin) {
                Ok(skin) => skin.options(),
                Err(err) => {
                    eprintln!("[ERROR]: Failed to parse skin content. Reason: {err}");
                    std::process::exit(1);
                }
            }
        }
    };

    let figure = match Figure::from_json5(&content) {
        Ok(figure) => figure,
        Err(err) => {
            eprintln!("[ERROR]: Failed to parse content of file. Reason:");
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let Figure::Signal(figure) = figure;
    let assembled = figure.assemble_with_options(assemble_options);

    let mut svg_data = BufWriter::new(Vec::new());
    match assembled.write_svg_with_options(&mut svg_data, &render_options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR]: Failed to assemble figure. Reason: {e}");
        }
    }
    
    let result = match flags.output {
        None => {
            let mut writer = BufWriter::new(stdout().lock());
            match writer.write(svg_data.buffer()) {
                Ok(_s) => {
                    Ok(())
                },
                Err(e) => {
                    eprintln!("[ERROR]: Error writing svg file: {e:?}.");
                    Err(e)
                }
            }
        }
        Some(ref output_path) => {
            if let Some(extension) = get_file_extension(&output_path.as_os_str().to_str().unwrap()) {
                match extension.to_lowercase().as_str() {
                    "svg" => {
                        export_svg_file(output_path, &svg_data.buffer())
                    },
                    "png" => {
                        export_png_file(&flags, output_path, &svg_data.buffer())
                    },
                    _ => {
                        eprintln!("[ERROR]: Unsupported file extension in output path.");
                        std::process::exit(1)
                    }
                }
            } else {
                eprintln!("[ERROR]: Output path has no file extension");
                std::process::exit(1)
            }
        }
    };

    if let Err(err) = result {
        eprintln!("[ERROR]: Failed to write output. Reason: {err}");
        std::process::exit(1);
    }
}
