mod context;

use std::path::Path;

use globwalk::GlobWalkerBuilder;
use mdbook::book::{Book, BookItem};
use mdbook::errors::{Error as BookError, ErrorKind as BookErrorKind};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use tera::{Context, Tera};
use thiserror::Error;

pub use self::context::{ContextSource, StaticContextSource};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] ::std::io::Error),
    #[error("{0}")]
    Tera(#[from] ::tera::Error),
    #[error("{0}")]
    Glob(#[from] ::globwalk::GlobError),
    #[error("{0}")]
    Toml(#[from] ::toml::de::Error),
    #[error("{0}")]
    Json(#[from] ::serde_json::Error),
    #[error("invalid path")]
    InvalidPath,
}

/// A mdBook preprocessor that renders Tera.
#[derive(Clone)]
pub struct TeraPreprocessor<C = StaticContextSource> {
    tera: Tera,
    context: C,
}

impl<C> TeraPreprocessor<C> {
    /// Construct a Tera preprocessor given a context source.
    pub fn new(context: C) -> Self {
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
        let root = &root.as_ref().canonicalize()?;

        let paths = GlobWalkerBuilder::from_patterns(root, &[glob_str])
            .build()?
            .filter_map(|r| r.ok())
            .filter_map(|p| {
                let path = p.into_path();
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                Some((path, Some(name)))
            });

        self.tera.add_template_files(paths)?;

        Ok(())
    }

    /// Returns a mutable reference to the internal Tera engine.
    pub fn tera_mut(&mut self) -> &mut Tera {
        &mut self.tera
    }
}

impl<C: Default> Default for TeraPreprocessor<C> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<C> Preprocessor for TeraPreprocessor<C>
where
    C: ContextSource,
{
    fn name(&self) -> &str {
        "tera"
    }

    fn run(&self, book_ctx: &PreprocessorContext, mut book: Book) -> Result<Book, BookError> {
        let mut tera = Tera::default();
        tera.extend(&self.tera).unwrap();

        let mut ctx = Context::new();
        ctx.insert("ctx", &book_ctx);
        ctx.extend(self.context.context());

        render_book_items(&mut book, &mut tera, &ctx)
            .map_err(|err| BookErrorKind::Msg(err.to_string()))?;

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
                let path = chapter.path.to_str().ok_or(Error::InvalidPath)?;
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
                let path = chapter.path.to_str().ok_or(Error::InvalidPath)?;
                chapter.content = tera.render(path, context)?;
                render_item_chapters(tera, context, chapter.sub_items.as_mut_slice())?;
            }
            BookItem::Separator => (),
        }
    }
    Ok(())
}
