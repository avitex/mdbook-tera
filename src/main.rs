use std::path::Path;
use std::{io, process};

use anyhow::anyhow;
use clap::{App, Arg};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};

use mdbook_tera::{StaticContextSource, TeraPreprocessor};

const DEFAULT_CONTEXT_TOML_PATH: &str = "./src/context.toml";
const DEFAULT_TEMPLATE_ROOT: &str = "./src";

fn app() -> App<'static> {
    App::new("mdbook-tera")
        .about("A mdBook preprocessor that renders Tera")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("json")
                .long("json")
                .value_name("FILE")
                .help("Sets context from JSON file")
                .takes_value(true),
        )
        .arg(
            Arg::new("toml")
                .long("toml")
                .value_name("FILE")
                .help("Sets context from TOML file")
                .takes_value(true),
        )
        .arg(
            Arg::new("template-root")
                .long("template-root")
                .value_name("PATH")
                .help("Root directory to include templates from")
                .default_value(DEFAULT_TEMPLATE_ROOT)
                .takes_value(true),
        )
        .arg(
            Arg::new("template-include")
                .long("template-include")
                .value_name("GLOB")
                .help("Include tera templates matching a glob expression")
                .default_value("**/*.tera")
                .takes_value(true),
        )
        .subcommand(
            App::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = app().get_matches();

    if matches.subcommand_matches("supports").is_some() {
        // We support every renderer
        process::exit(0);
    }

    let ctx_src = match (matches.value_of("json"), matches.value_of("toml")) {
        (Some(_), Some(_)) => exit_with_error(anyhow!("cannot set both json and toml context")),
        (Some(json_path), None) => StaticContextSource::from_json_file(json_path),
        (None, Some(toml_path)) => StaticContextSource::from_toml_file(toml_path),
        (None, None) => {
            let default_path = Path::new(DEFAULT_CONTEXT_TOML_PATH);
            if default_path.exists() {
                StaticContextSource::from_toml_file(default_path)
            } else {
                Ok(StaticContextSource::default())
            }
        }
    };

    let ctx_src = match ctx_src {
        Ok(ctx_src) => ctx_src,
        Err(err) => {
            exit_with_error(anyhow!("failed to load context: {}", err));
        }
    };

    let mut preprocessor = TeraPreprocessor::new(ctx_src);

    if let Some(glob_str) = matches.value_of("template-include") {
        let root_dir = matches
            .value_of("template-root")
            .unwrap_or(DEFAULT_TEMPLATE_ROOT);

        if glob_str != "false" {
            if let Err(err) = preprocessor.include_templates(root_dir, glob_str) {
                exit_with_error(anyhow!(err));
            }
        }
    }

    if let Err(err) = handle_preprocessing(&preprocessor) {
        exit_with_error(anyhow!(err));
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

fn exit_with_error(err: Error) -> ! {
    eprintln!("{:?}", err);
    process::exit(1);
}
