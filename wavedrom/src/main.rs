use std::io::{stdin, stdout, Read, BufWriter};
use std::path::PathBuf;

use clap::{value_parser, Arg, Command};
use wavedrom::skin::Skin;
use wavedrom::{Figure, PathAssembleOptions};
use wavedrom::options::RenderOptions;

static ABOUT: &str = r#"
A Signal Diagram Generator from WaveJson.

By default, this application attempts to read a file from STDIN and output to STDOUT. An input file can be given with the -i/--input flag and a output file can be passed with the -o/--output file.
"#;

pub fn make_app() -> Command {
    Command::new("wavedrom")
        .about(ABOUT.trim())
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT FILE")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT FILE")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("skin")
                .short('s')
                .long("skin")
                .value_name("SKIN FILE")
                .value_parser(value_parser!(PathBuf)),
        )
}

fn main() {
    let app = make_app().get_matches();

    let content = match app.get_one::<PathBuf>("input") {
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

    let (assemble_options, render_options) = match app.get_one::<PathBuf>("skin") {
        None => (PathAssembleOptions::default(), RenderOptions::default()),
        Some(skin_path) => {
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
        },
    };

    let figure = match Figure::from_json5(&content) {
        Ok(figure) => figure,
        Err(err) => {
            eprintln!("[ERROR]: Failed to parse content of file. Reason:");
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let assembled = figure.assemble_with_options(assemble_options);

    let result = match app.get_one::<PathBuf>("output") {
        None => {
            let mut writer = BufWriter::new(stdout().lock());
            assembled.write_svg(&mut writer)
        },
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

            let mut writer = BufWriter::new(output_file);
            assembled.write_svg_with_options(&mut writer, &render_options)
        }
    };

    if let Err(err) = result {
        eprintln!("[ERROR]: Failed to write out svg. Reason: {err}");
        std::process::exit(1);
    }
}