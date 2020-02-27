{% import "macros.tera" as macros %}

# Chapter 1

{{ macros::greeting(name=my_value) }}

This book is written by {{ ctx.config.book.authors | join() }}
