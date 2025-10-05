use std::path::{Path, PathBuf};
use std::{io, process};

use anyhow::anyhow;
use clap::Parser;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};

use mdbook_tera::{StaticContextSource, TeraPreprocessor};

const DEFAULT_CONTEXT_TOML_PATH: &str = "./src/context.toml";
const DEFAULT_TEMPLATE_ROOT: &str = "./src";

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Sets context from JSON file
    #[clap(long, value_name = "FILE")]
    json: Option<PathBuf>,
    /// Sets context from TOML file
    #[clap(long, value_name = "FILE")]
    toml: Option<PathBuf>,
    /// Root directory to include templates from
    #[clap(long, value_name = "PATH", default_value = DEFAULT_TEMPLATE_ROOT)]
    template_root: PathBuf,
    /// Include tera templates matching a glob expression
    #[clap(long, value_name = "GLOB", default_value = "**/*.tera")]
    template_include: Option<String>,
    #[clap(subcommand)]
    cmd: Option<Subcommand>,
}

#[derive(Parser)]
enum Subcommand {
    /// Check whether a renderer is supported by this preprocessor
    Supports { renderer: PathBuf },
}

fn main() {
    let args = Args::parse();

    if let Some(Subcommand::Supports { .. }) = args.cmd {
        // We support every renderer
        process::exit(0);
    }

    let ctx_src = match (args.json, args.toml) {
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

    if let Some(glob_str) = args.template_include {
        if glob_str != "false" {
            if let Err(err) = preprocessor.include_templates(&args.template_root, &glob_str) {
                exit_with_error(err);
            }
        }
    }

    if let Err(err) = handle_preprocessing(&preprocessor) {
        exit_with_error(err);
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
