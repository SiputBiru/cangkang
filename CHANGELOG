# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
- Fixed list parsing to support lists with blank lines between items, preventing numbering resets in ordered lists.
- Dropdown code blocks using `+++` fence, supporting optional language and title (e.g., `+++rust [My Title]`).
- Native HTML `<details>` and `<summary>` tags for dropdowns.
- Custom CSS styling for dropdown code blocks.

### Added
- Draft support for blog posts:
    - Posts are now considered drafts by default unless `"draft": false` is explicitly set in frontmatter.
    - Drafts are excluded from compilation and index listings.
    - Special handling for `index.md` and `404.md` to ensure they are always compiled.
- SEO and Discoverability features:
    - Automatic `sitemap.xml` generation during build.
    - RSS 2.0 feed (`index.xml`) generation for blog posts.
    - Support for `description` and `keywords` in frontmatter.
    - Meta tags for description and keywords in index and post templates.
- Modularized architecture:
    - Created `src/models.rs` to centralize data structures (`PageInfo`, `PageMetadata`).
    - Created `src/seo.rs` to encapsulate all discoverability and asset generation logic.
    - Decoupled `src/compiler.rs` and `src/frontmatter.rs` from data model definitions.
- Dedicated color-coded logger in `src/logger.rs` with `log_info`, `log_success`, `log_warn`, and `log_error` macros.
- Support for bold text in the logger and file path highlighting during compilation.
- Context-aware error handling in `src/error.rs` with `IoContext` trait to attach file paths to IO errors.
- Enhanced error reporting in `src/fs.rs`, `src/compiler.rs`, and `src/frontmatter.rs`.
- `GEMINI.md` for project context and standards.

## [0.1.0] - 2026-03-27

### Added
- MIT License.
- Docker and Nginx support for deployment.
- Pinned posts support and date-based sorting.
- Callout support (WARN, INFO) using enums.
- Table support in Markdown.
- Footnote and sidenote support.
- Image support (local and remote).
- Initial project structure with custom Markdown lexer/parser.

### Fixed
- Improved routing and link behavior (opening in new tabs).
- Optimized Docker builder image.
- Removed unused formatting and simplified internal compiler logic.
