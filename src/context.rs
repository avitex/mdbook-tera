#[cfg(any(feature = "json", feature = "toml"))]
use std::fs;
#[cfg(any(feature = "json", feature = "toml"))]
use std::path::Path;

pub use tera::Context;

#[cfg(any(feature = "json", feature = "toml"))]
use crate::Error;

/// A context source for the Tera preprocessor.
pub trait ContextSource {
    /// Returns a context from the source.
    fn context(&self) -> Context;
}

/// Static context source for the Tera preprocessor.
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
    #[must_use]
    pub const fn new(context: Context) -> Self {
        Self { context }
    }

    /// Construct a context source given a JSON path.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided path or JSON read is invalid.
    #[cfg(feature = "json")]
    pub fn from_json_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let context_str = fs::read_to_string(path)?;
        let value = context_str.parse()?;
        let context = Context::from_value(value)?;
        Ok(Self::new(context))
    }

    /// Construct a context source given a TOML path.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided path or TOML read is invalid.
    #[cfg(feature = "toml")]
    pub fn from_toml_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let context_str = fs::read_to_string(path)?;
        let value: toml::Value = context_str.parse()?;
        let context = Context::from_serialize(value)?;
        Ok(Self::new(context))
    }
}

impl Default for StaticContextSource {
    fn default() -> Self {
        Self::new(Context::default())
    }
}
