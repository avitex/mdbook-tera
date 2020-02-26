use mdbook::book::{Book, BookItem};
use mdbook::errors::Error as BookError;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use serde::Serialize;
use tera::{Error as TeraError, Tera};
use toml::Value as TomlValue;

pub use tera::Context;

#[derive(Debug)]
pub enum ProcessingError {
    Tera(TeraError),
    InvalidPath,
}

impl From<TeraError> for ProcessingError {
    fn from(err: TeraError) -> Self {
        Self::Tera(err)
    }
}

#[derive(Clone)]
pub struct TeraPreprocessor {
    tera: Tera,
    context: Context,
}

impl TeraPreprocessor {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            tera: Tera::default(),
        }
    }

    pub fn get_tera_mut(&mut self) -> &mut Tera {
        &mut self.tera
    }

    pub fn from_json_str(json_str: &str) -> Self {
        let value = json_str.parse().expect("json context malformed");
        let context = Context::from_value(value).expect("invalid tera context");
        Self::new(context)
    }

    pub fn from_toml_str(toml_str: &str) -> Self {
        let value: TomlValue = toml_str.parse().expect("toml context malformed");
        Self::from_serialize(value)
    }

    pub fn from_serialize(value: impl Serialize) -> Self {
        let context = Context::from_serialize(value).expect("invalid tera context");
        Self::new(context)
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
        ctx.extend(self.context.clone());

        // TODO: handle errors
        render_book_items(&mut book, &mut tera, &ctx).unwrap();

        Ok(book)
    }
}

fn render_book_items(
    book: &mut Book,
    tera: &mut Tera,
    context: &Context,
) -> Result<(), ProcessingError> {
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
) -> Result<(), ProcessingError> {
    for item in items {
        match item {
            BookItem::Chapter(chapter) => {
                let path = chapter.path.to_str().ok_or(ProcessingError::InvalidPath)?;
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
) -> Result<(), ProcessingError> {
    for item in items {
        match item {
            BookItem::Chapter(chapter) => {
                let path = chapter.path.to_str().ok_or(ProcessingError::InvalidPath)?;
                chapter.content = tera.render(path, context)?;
                render_item_chapters(tera, context, chapter.sub_items.as_mut_slice())?;
            }
            BookItem::Separator => (),
        }
    }
    Ok(())
}
