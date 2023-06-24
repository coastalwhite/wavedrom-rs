use std::fmt::Display;
use std::fs::File;
use std::io::{stdin, stdout, BufWriter, Read, self, StdoutLock};
use std::path::PathBuf;

use wavedrom::signal::options::{RenderOptions, PathAssembleOptions};
use wavedrom::skin::Skin;
use wavedrom::Figure;

#[derive(Default)]
struct Flags {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    skin: Option<PathBuf>,
}

enum OutputWriter<'a> {
    Stdio(StdoutLock<'a>),
    File(File),
}

impl<'a> io::Write for OutputWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdio(stdio) => stdio.write(buf),
            Self::File(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdio(stdio) => stdio.flush(),
            Self::File(file) => file.flush(),
        }
    }
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
-i/--input <path/to/input.json>: specify a path to a input wavejson file
-o/--output <path/to/output.svg>: specify a path to a output svg file
-s/--skin <path/to/skin.json>: specify a path to a skin file
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
        Some(input_path) => match std::fs::read_to_string(input_path) {
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

    let mut writer = BufWriter::new(match flags.output {
        None => {
            OutputWriter::Stdio(stdout().lock())
        }
        Some(output_path) => {
            let output_file = match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(output_path)
            {
                Ok(f) => f,
                Err(err) => {
                    eprintln!("[ERROR]: Failed to open output file. Reason: {err}");
                    std::process::exit(1);
                }
            };

            OutputWriter::File(output_file)
        }
    });


    let result = match figure {
        Figure::Signal(figure) => {
            let assembled = figure.assemble_with_options(assemble_options);
            assembled.write_svg_with_options(&mut writer, &render_options)
        }
        Figure::Register(register) => {
            register.write_svg(&mut writer)
        }
    };

    if let Err(err) = result {
        eprintln!("[ERROR]: Failed to write out svg. Reason: {err}");
        std::process::exit(1);
    }
}
