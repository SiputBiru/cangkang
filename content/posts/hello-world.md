---
{
    "title": "Welcome to Cangkang!",
    "date": "2026-03-25",
    "description": "My first test post."
}
---

# A New Beginning

Welcome to my brand new static site generator(SSG), generated entirely from scratch using Rust. No bloated dependencies, no heavy JavaScript frameworks, just pure, HTML generation.

It feels incredibly satisfying to own the entire stack from the parser to the final HTML output.

My plan for deployment is to create dockerfile with nginx as a http server.

## Things that work perfectly

* **Bold text** and *italic text* parsing.
* Custom JSON frontmatter extraction.
* Images routing from the public directory!

image from internet:
![cat-1](https://placecats.com/300/200)

image from local:
![cat-2]({{ root_dir }}images/cat.jpeg)

It even handles inline code like `let x = 10;` flawlessly without breaking the surrounding paragraph.

> [!NOTE]
> **Note stuff**
> simple Callout

### Numbered List

nested bullet item

1. Main item
    * Nested bullet item
    * Another nested bullet item

nested numbered item
2. Second main item
    1. Nested numbered item
    2. Another nested numbered item

### Foot note

Here is a normal paragraph, but it has a secret footnote attached to it[^1].

[^1]: This is the footnote text! When we compile this, Cangkang will grab this text and turn it into a beautiful, interactive margin note.
