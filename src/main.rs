use std::path::Path;
use std::{fs, io, process};

use clap::{App, Arg, SubCommand};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};

use mdbook_tera::TeraPreprocessor;

const DEFAULT_CONTEXT_TOML_PATH: &str = "./context.toml";

fn app() -> App<'static, 'static> {
    App::new("mdbook-tera")
        .about("A mdbook preprocessor that renders tera")
        .arg(
            Arg::with_name("json")
                .long("json")
                .value_name("FILE")
                .help("Sets JSON context")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("toml")
                .long("toml")
                .value_name("FILE")
                .help("Sets TOML context")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = app().get_matches();

    if let Some(_) = matches.subcommand_matches("supports") {
        // We support every renderer.
        process::exit(0);
    }

    let preprocessor = match (matches.value_of("json"), matches.value_of("toml")) {
        (Some(_), Some(_)) => panic!("cannot set both json and toml context"),
        (Some(json_path), None) => {
            let json_str = load_context_file(json_path);
            TeraPreprocessor::from_json_str(json_str)
        }
        (None, Some(toml_path)) => {
            let toml_str = load_context_file(toml_path);
            TeraPreprocessor::from_toml_str(toml_str)
        }
        (None, None) => {
            if Path::new(DEFAULT_CONTEXT_TOML_PATH).exists() {
                let toml_str = load_context_file(DEFAULT_CONTEXT_TOML_PATH);
                TeraPreprocessor::from_toml_str(toml_str)
            } else {
                TeraPreprocessor::default()
            }
        }
    };

    if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn load_context_file(path: &str) -> String {
    fs::read_to_string(path).expect("failed to load context file")
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let ver = Version::parse(ctx.mdbook_version.as_str()).unwrap();
    let ver_req = VersionReq::parse(mdbook::MDBOOK_VERSION).unwrap();

    if !ver_req.matches(&ver) {
        eprintln!(
            "Warning: The {} plugin has the version requirement {} for mdbook, \
             but we're being called from version {}",
            pre.name(),
            ver_req,
            ver
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
