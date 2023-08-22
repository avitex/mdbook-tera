#![deny(
    warnings,
    rustdoc::all,
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic
)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(doc, deny(rustdoc::all))]
#![cfg_attr(doc, allow(rustdoc::missing_doc_code_examples))]

mod context;

use std::path::Path;

use anyhow::anyhow;
use globwalk::GlobWalkerBuilder;
use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use tera::{Context, Tera};

pub use self::context::{ContextSource, StaticContextSource};

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

    /// Includes Tera templates given a glob pattern and a root directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided path cannot be canonicalized or the
    /// inheritance chain can't be built, such as adding a child template
    /// without the parent one.
    #[allow(clippy::missing_panics_doc)]
    pub fn include_templates<P>(&mut self, root: P, glob_str: &str) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let root = &root.as_ref().canonicalize()?;

        let paths = GlobWalkerBuilder::from_patterns(root, &[glob_str])
            .build()?
            .filter_map(Result::ok)
            .map(|p| {
                let path = p.into_path();
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                (path, Some(name))
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

    fn run(&self, book_ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut tera = Tera::default();
        tera.extend(&self.tera).unwrap();

        let mut ctx = Context::new();
        ctx.insert("ctx", &book_ctx);
        ctx.extend(self.context.context());

        render_book_items(&mut book, &mut tera, &ctx)?;

        Ok(book)
    }
}

fn render_book_items(book: &mut Book, tera: &mut Tera, context: &Context) -> Result<(), Error> {
    let mut templates = Vec::new();
    // Build the list of templates
    collect_item_chapters(&mut templates, book.sections.as_slice())?;
    // Register them
    tera.add_raw_templates(templates)?;
    // Render chapters
    render_item_chapters(tera, context, book.sections.as_mut_slice())
}

fn collect_item_chapters<'a>(
    templates: &mut Vec<(&'a str, &'a str)>,
    items: &'a [BookItem],
) -> Result<(), Error> {
    for item in items {
        match item {
            BookItem::Chapter(chapter) => {
                if let Some(ref path) = chapter.path {
                    let path = path
                        .to_str()
                        .ok_or_else(|| anyhow!("invalid chapter path"))?;
                    templates.push((path, chapter.content.as_str()));
                }
                collect_item_chapters(templates, chapter.sub_items.as_slice())?;
            }
            BookItem::PartTitle(_) | BookItem::Separator => (),
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
                if let Some(ref path) = chapter.path {
                    let path = path
                        .to_str()
                        .ok_or_else(|| anyhow!("invalid chapter path"))?;
                    chapter.content = tera.render(path, context)?;
                }
                render_item_chapters(tera, context, chapter.sub_items.as_mut_slice())?;
            }
            BookItem::PartTitle(_) | BookItem::Separator => (),
        }
    }
    Ok(())
}
