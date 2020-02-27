[![Build Status](https://travis-ci.org/avitex/mdbook-tera.svg)](https://travis-ci.org/avitex/mdbook-tera)
[![Crate](https://img.shields.io/crates/v/mdbook-tera.svg)](https://crates.io/crates/mdbook-tera)
[![Docs](https://docs.rs/mdbook-tera/badge.svg)](https://docs.rs/mdbook-tera)

# mkbook-tera

**[Tera](https://github.com/Keats/tera) preprocessor for [mdBook](https://github.com/rust-lang/mdBook)**  
API documentation hosted on [docs.rs](https://docs.rs/mdbook-tera).

## Usage

First install the tera preprocessor.

```sh
cargo install mdbook-tera
```

Then in your `book.toml` file, add the tera preprocessor as below.

### Default Configuration

```toml
# Default options, load a TOML context file from ./src/context.toml
[preprocessor.tera]
```

### JSON Configuration

```toml
[preprocessor.tera]
command = "mdbook-tera --json ./src/context.json"
```

See the [example book](../example-book) for a basic usage.
