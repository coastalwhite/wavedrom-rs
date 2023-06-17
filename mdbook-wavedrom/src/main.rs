use crate::nop_lib::WavedromPreProcessor;
use clap::{Arg, ArgMatches, Command};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use semver::{Version, VersionReq};
use std::io;
use std::process;

pub fn make_app() -> Command {
    Command::new("mdbook-wavedrom")
        .about("A mdbook preprocessor that renders wavedrom-rs diagrams")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    // Users will want to construct their own preprocessor here
    let preprocessor = WavedromPreProcessor::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

mod nop_lib {
    use mdbook::BookItem;
    use mdbook_wavedrom::insert_wavedrom;
    use wavedrom::options::RenderOptions;
    use wavedrom::skin::Skin;
    use wavedrom::PathAssembleOptions;

    use super::*;

    pub struct WavedromPreProcessor;

    impl WavedromPreProcessor {
        pub fn new() -> WavedromPreProcessor {
            WavedromPreProcessor
        }
    }

    impl Preprocessor for WavedromPreProcessor {
        fn name(&self) -> &str {
            "wavedrom"
        }

        fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
            let mut assemble_options = PathAssembleOptions::default();
            let mut render_options = RenderOptions::default();

            if let Some(config) = ctx.config.get_preprocessor(self.name()) {
                if let Some(skin_path) = config.get("skin") {
                    let Some(skin_path) = skin_path.as_str() else {
                        eprintln!("[ERROR]: WaveDrom skin has invalid value type");
                        std::process::exit(1);
                    };

                    let skin = match std::fs::read_to_string(skin_path) {
                        Ok(content) => content,
                        Err(err) => {
                            eprintln!("[ERROR]: Failed to read content from WaveDrom skin file. Reason: {err}");
                            std::process::exit(1);
                        }
                    };

                    match Skin::from_json5(&skin) {
                        Ok(skin) => {
                            if let Some(assemble) = skin.assemble {
                                assemble_options = assemble;
                            }

                            if let Some(render) = skin.render {
                                render_options.merge_in(render);
                            }
                        }
                        Err(err) => {
                            eprintln!(
                                "[ERROR]: Failed to parse WaveDrom skin content. Reason: {err}"
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }

            book.for_each_mut(|item| match item {
                BookItem::Separator | BookItem::PartTitle(_) => {}
                BookItem::Chapter(chapter) => {
                    match insert_wavedrom(&chapter.content, assemble_options, &render_options) {
                        Ok(new_content) => chapter.content = new_content,
                        Err(..) => {}
                    }
                }
            });

            Ok(book)
        }

        fn supports_renderer(&self, renderer: &str) -> bool {
            renderer != "not-supported"
        }
    }
}
