[![Build Status](https://travis-ci.org/avitex/mdbook-tera.svg)](https://travis-ci.org/avitex/mdbook-tera)
[![Crate](https://img.shields.io/crates/v/mdbook-tera.svg)](https://crates.io/crates/mdbook-tera)
[![Docs](https://docs.rs/mdbook-tera/badge.svg)](https://docs.rs/mdbook-tera)

# mdbook-tera

**[Tera](https://github.com/Keats/tera) preprocessor for [mdBook](https://github.com/rust-lang/mdBook)**  
API documentation hosted on [docs.rs](https://docs.rs/mdbook-tera).

```text
$ mdbook-tera --help
mdbook-tera 0.2.0
A mdBook preprocessor that renders Tera

USAGE:
    mdbook-tera [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --json <FILE>                Sets context from JSON file
        --template-include <GLOB>    Include tera templates matching a glob expression [default: **/*.tera]
        --template-root <PATH>       Root directory to include templates from [default: ./src]
        --toml <FILE>                Sets context from TOML file

SUBCOMMANDS:
    help        Prints this message or the help of the given subcommand(s)
    supports    Check whether a renderer is supported by this preprocessor
```

## Usage

First install the tera preprocessor.

```text
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

### Usage in Markdown files

See `example-book` for a basic usage.

Simply define your values in the `context.toml` file, and use them in tera statements.
You can access the book context with the key `ctx`.

```md
# My Heading

{{ my_value }}
```
