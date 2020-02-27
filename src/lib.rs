mod context;

use std::path::{Path, PathBuf};

use error_chain::ChainedError;
use glob::glob;
use mdbook::book::{Book, BookItem};
use mdbook::errors::{Error as BookError, ErrorKind as BookErrorKind};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use tera::{Context, Tera};

pub use self::context::ContextSource;
pub use self::errors::{Error, ErrorKind};

mod errors {
    use error_chain::error_chain;

    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Tera(::tera::Error);
            Glob(::glob::PatternError);
            Toml(::toml::de::Error);
            Json(::serde_json::Error);
            Notify(::notify::Error);
        }

        errors {
            InvalidPath
        }

        skip_msg_variant
    }
}

/// A mdBook preprocessor that renders Tera.
#[derive(Clone)]
pub struct TeraPreprocessor {
    tera: Tera,
    context: ContextSource,
}

impl TeraPreprocessor {
    /// Construct a Tera preprocessor given a context source.
    pub fn new(context: ContextSource) -> Self {
        Self {
            context,
            tera: Tera::default(),
        }
    }

    /// Includes tera templates given a glob pattern and a root directory.
    pub fn include_templates<P>(&mut self, root: P, glob_str: &str) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let root = root.as_ref().canonicalize()?;
        let glob_with_root = root.join(glob_str);
        let glob_with_root = glob_with_root.to_string_lossy().to_owned();

        let paths: Vec<(PathBuf, String)> = glob(glob_with_root.as_ref())?
            .filter_map(|r| r.ok())
            .filter_map(|p| {
                if let Ok(name) = p.strip_prefix(root.as_path()) {
                    let name = name.to_string_lossy().into();
                    Some((p, name))
                } else {
                    None
                }
            })
            .collect();

        let path_refs = paths
            .iter()
            .map(|(p, n)| (p.as_path(), Some(n.as_str())))
            .collect();

        self.tera.add_template_files(path_refs)?;

        Ok(())
    }

    /// Returns a mutable reference to the internal Tera engine.
    pub fn get_tera_mut(&mut self) -> &mut Tera {
        &mut self.tera
    }
}

impl Default for TeraPreprocessor {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl Preprocessor for TeraPreprocessor {
    fn name(&self) -> &str {
        "tera"
    }

    fn run(&self, book_ctx: &PreprocessorContext, mut book: Book) -> Result<Book, BookError> {
        let mut tera = Tera::default();
        tera.extend(&self.tera).unwrap();

        let mut ctx = Context::new();
        ctx.insert("ctx", &book_ctx);
        ctx.extend(self.context.get_context());

        render_book_items(&mut book, &mut tera, &ctx)
            .map_err(|err| BookErrorKind::Msg(err.display_chain().to_string()))?;

        Ok(book)
    }
}

fn render_book_items(book: &mut Book, tera: &mut Tera, context: &Context) -> Result<(), Error> {
    let mut templates = Vec::new();
    // build the list of templates
    collect_item_chapters(&mut templates, book.sections.as_slice())?;
    // register them
    tera.add_raw_templates(templates)?;
    // render chapters
    render_item_chapters(tera, context, book.sections.as_mut_slice())
}

fn collect_item_chapters<'a>(
    templates: &mut Vec<(&'a str, &'a str)>,
    items: &'a [BookItem],
) -> Result<(), Error> {
    for item in items {
        match item {
            BookItem::Chapter(chapter) => {
                let path = chapter.path.to_str().ok_or(ErrorKind::InvalidPath)?;
                templates.push((path, chapter.content.as_str()));
                collect_item_chapters(templates, chapter.sub_items.as_slice())?;
            }
            BookItem::Separator => (),
        }
    }
    Ok(())
}

fn render_item_chapters(
    tera: &mut Tera,
    context: &Context,
    items: &mut [BookItem],
) -> Result<(), Error> {
    for item in items {
        match item {
            BookItem::Chapter(chapter) => {
                let path = chapter.path.to_str().ok_or(ErrorKind::InvalidPath)?;
                chapter.content = tera.render(path, context)?;
                render_item_chapters(tera, context, chapter.sub_items.as_mut_slice())?;
            }
            BookItem::Separator => (),
        }
    }
    Ok(())
}
