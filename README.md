[![Build Status](https://github.com/avitex/mdbook-tera/workflows/build/badge.svg)](https://github.com/avitex/mdbook-tera/actions?query=workflow:build)
[![Crate](https://img.shields.io/crates/v/mdbook-tera.svg)](https://crates.io/crates/mdbook-tera)
[![Docs](https://docs.rs/mdbook-tera/badge.svg)](https://docs.rs/mdbook-tera)

# mdbook-tera

**[Tera](https://github.com/Keats/tera) preprocessor for [mdBook](https://github.com/rust-lang/mdBook)**  
API documentation hosted on [docs.rs](https://docs.rs/mdbook-tera).

```text
$ mdbook-tera --help
Tera preprocessor for mdBook

Usage: mdbook-tera [OPTIONS] [COMMAND]

Commands:
  supports  Check whether a renderer is supported by this preprocessor
  help      Print this message or the help of the given subcommand(s)

Options:
      --json <FILE>              Sets context from JSON file
      --toml <FILE>              Sets context from TOML file
      --template-root <PATH>     Root directory to include templates from [default: ./src]
      --template-include <GLOB>  Include tera templates matching a glob expression [default: **/*.tera]
  -h, --help                     Print help
  -V, --version                  Print version
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
