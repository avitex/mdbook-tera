use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use notify::{Event, EventFn, RecommendedWatcher, RecursiveMode, Watcher};

pub use tera::Context;

use crate::errors::{Error, ErrorKind};

#[derive(Clone, Copy)]
enum Format {
    Json,
    Toml,
}

#[derive(Clone)]
pub struct ContextSource {
    context: Arc<Mutex<Context>>,
}

impl ContextSource {
    pub fn new(context: Context) -> Self {
        Self {
            context: Arc::new(Mutex::new(context)),
        }
    }

    pub fn get_context(&self) -> Context {
        self.lock_context().clone()
    }

    pub fn from_json_file<P>(path: P, watch: bool) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::from_file(path.as_ref(), Format::Json, watch)
    }

    pub fn from_toml_file<P>(path: P, watch: bool) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::from_file(path.as_ref(), Format::Toml, watch)
    }

    fn from_file(path: &Path, format: Format, watch: bool) -> Result<Self, Error> {
        let this = if watch {
            let this = Self::from_file(path, format, false)?;
            let this_clone = this.clone();
            let path_buf = path.to_owned();
            let parent_path = path.parent().ok_or(ErrorKind::InvalidPath)?;
            watch_path(parent_path, move |res: Result<Event, _>| {
                match res {
                    Ok(event) => {
                        if !event.paths.contains(&path_buf) {
                            return;
                        }
                        match load_context_file(path_buf.as_ref(), format) {
                            Ok(ctx) => {
                                *this_clone.lock_context() = ctx;
                                eprintln!("context reloaded");
                            }
                            Err(err) => eprintln!("failed to reload context: {:?}", err),
                        }
                    }
                    Err(e) => eprintln!("watch error: {:?}", e),
                };
            })?;
            this
        } else {
            let ctx = load_context_file(path, format)?;
            Self::new(ctx)
        };
        Ok(this)
    }

    fn lock_context(&self) -> MutexGuard<Context> {
        self.context.lock().expect("context lock poisoned")
    }
}

impl Default for ContextSource {
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

fn watch_path<F>(path: &Path, event_fn: F) -> Result<(), Error>
where
    F: EventFn,
{
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(event_fn)?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;
    Ok(())
}
