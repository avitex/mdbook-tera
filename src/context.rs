use std::fs;
use std::path::Path;

pub use tera::Context;

use crate::Error;

pub trait ContextSource {
    /// Returns a context from the source.
    fn context(&self) -> Context;
}

#[derive(Clone, Copy)]
enum Format {
    Json,
    Toml,
}

/// A context source for the Tera preprocessor.
#[derive(Clone)]
pub struct StaticContextSource {
    context: Context,
}

impl ContextSource for StaticContextSource {
    fn context(&self) -> Context {
        self.context.clone()
    }
}

impl StaticContextSource {
    /// Construct a context source given a tera context.
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    /// Construct a context source given a JSON path.
    pub fn from_json_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::from_file(path.as_ref(), Format::Json)
    }

    /// Construct a context source given a TOML path.
    pub fn from_toml_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::from_file(path.as_ref(), Format::Toml)
    }

    fn from_file(path: &Path, format: Format) -> Result<Self, Error> {
        let ctx = load_context_file(path, format)?;
        Ok(Self::new(ctx))
    }
}

impl Default for StaticContextSource {
    fn default() -> Self {
        Self::new(Context::default())
    }
}

fn load_context_file(path: &Path, format: Format) -> Result<Context, Error> {
    let context_str = fs::read_to_string(path)?;

    match format {
        Format::Json => from_json_str(context_str.as_str()),
        Format::Toml => from_toml_str(context_str.as_str()),
    }
}

fn from_json_str(json_str: &str) -> Result<Context, Error> {
    let value = json_str.parse()?;
    Ok(Context::from_value(value)?)
}

fn from_toml_str(toml_str: &str) -> Result<Context, Error> {
    let value: toml::Value = toml_str.parse()?;
    Ok(Context::from_serialize(value)?)
}
