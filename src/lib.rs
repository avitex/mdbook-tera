mod context;

use error_chain::ChainedError;
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

/// A mdbook preprocessor that renders tera.
#[derive(Clone)]
pub struct TeraPreprocessor {
    tera: Tera,
    context: ContextSource,
}

impl TeraPreprocessor {
    pub fn new(context: ContextSource) -> Self {
        Self {
            context,
            tera: Tera::default(),
        }
    }

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
