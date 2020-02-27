use std::path::Path;
use std::{io, process};

use clap::{App, Arg, SubCommand};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};

use mdbook_tera::{ContextSource, TeraPreprocessor};

const DEFAULT_CONTEXT_TOML_PATH: &str = "./src/context.toml";

fn app() -> App<'static, 'static> {
    App::new("mdbook-tera")
        .about("A mdbook preprocessor that renders tera")
        .arg(
            Arg::with_name("watch")
                .long("watch")
                .value_name("BOOL")
                .help("Reload the provided context file on change")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .value_name("FILE")
                .help("Sets JSON context file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("toml")
                .long("toml")
                .value_name("FILE")
                .help("Sets TOML context file")
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

    if matches.subcommand_matches("supports").is_some() {
        // We support every renderer.
        process::exit(0);
    }

    let ctx_src = match (
        matches.value_of("json"),
        matches.value_of("toml"),
        matches.is_present("watch"),
    ) {
        (Some(_), Some(_), _) => exit_with_error("cannot set both json and toml context".into()),
        (Some(json_path), None, watch) => ContextSource::from_json_file(json_path, watch),
        (None, Some(toml_path), watch) => ContextSource::from_toml_file(toml_path, watch),
        (None, None, watch) => {
            let default_path = Path::new(DEFAULT_CONTEXT_TOML_PATH);
            if default_path.exists() {
                ContextSource::from_toml_file(default_path, watch)
            } else {
                Ok(ContextSource::default())
            }
        }
    };

    let ctx_src = match ctx_src {
        Ok(ctx_src) => ctx_src,
        Err(err) => {
            exit_with_error(format!("failed to load context: {}", err));
        }
    };

    let preprocessor = TeraPreprocessor::new(ctx_src);

    if let Err(err) = handle_preprocessing(&preprocessor) {
        exit_with_error(err.to_string());
    }
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

fn exit_with_error(err: String) -> ! {
    eprintln!("{}", err);
    process::exit(1);
}
